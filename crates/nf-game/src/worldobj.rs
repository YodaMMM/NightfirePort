//! The gameplay placement types the port drew as static geometry: the channel inverter (0x41), the
//! turret shooter (0xd2), the gated dynamic object (0x3b), the third-person camera spline (0xe7),
//! the one-sided wall (0xd5) and the music trigger (0xfb).
//! All are created by `parsemap_create_dynamic_objects` 0x1d04f0 from the placements' params
//! (the level_tag record, docs/fields/level-tag.md) and updated per tick by `control_funcs[class]`
//! through `control_movement_object_handler` 0x1336f8's update pass. None of the four creators
//! draws from the shared RNG (checked on each create's decompile), so creating them here does not
//! move the stream.
//!
//! Per-object layout follows the game's `obj_tag` (docs/atlas/object.md): the placement params the
//! creators copy into obj+0xF4/+0xF6 or the instance at obj+0xE0, the draw-flags word obj+0xF0 from
//! the glist's +0x44 (docs/fields/glist.md), the control flags obj+0xFE (bit 0 = delete me, applied
//! by the delete pass 0x1336f8:86-87 the same tick) and the tint/alpha byte obj+0x106.

use crate::world::V3;

fn dist3(a: V3, b: V3) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// InverterTrigger (placement 0x41 → `InverterTrigger_Create` 0x1b3040, class 0x53).
/// `InverterTrigger_Update` 0x1b30a8: while the IN channel is set, the OUT channel is forced to 0 —
/// the inverter that lets a script's 1 mean "off" (07000006/12/14).
#[derive(Clone, Debug)]
pub struct Inverter {
    /// obj+0xF4 = param 0 (u16): the channel watched.
    pub ch_in: u16,
    /// obj+0xF6 = param 1 (u16): the channel held low (`Update` refuses 0).
    pub ch_out: u16,
}

impl Inverter {
    /// `InverterTrigger_Update` 0x1b30a8: `if (switch_channels[in] && out) switch_channels[out] = 0;`
    pub fn tick(&self, channels: &mut crate::triggers::Channels, frame: u32) {
        if channels.on(self.ch_in as u32) && self.ch_out != 0 {
            channels.set(self.ch_out as u32, 0, frame);
        }
    }
}

/// Shooter (placement 0xd2 → `Shooter_Create` 0x17d030, class 0x4e): a one-shot fixed gun.
/// `Shooter_Update` 0x17d0c0: while the arm channel (param 0, obj+0xF4) is set it fires one
/// `Bullet_init` 0x12c5a0 bullet with the weapon at param 1 (obj+0xF6), along the object matrix's
/// forward (`Mat_GetDir` on obj+0x90) — or straight down (0, −1, 0) when the weapon record's class
/// halfword at +0x02 is 0x3b — then sets obj+0xFE bit 0, and the movement handler's delete pass
/// (0x1336f8:86-87) removes the object that same tick: one bullet per placement, on the arm edge.
#[derive(Clone, Debug)]
pub struct Shooter {
    /// The object position (obj+0x30 = the placement translation).
    pub pos: V3,
    /// The matrix forward (`Mat_GetDir(obj+0x90)`): the placement's RotZYX forward, column 2.
    pub dir: V3,
    /// obj+0xF6 = param 1 (u16): the `weapon_data` record index.
    pub weapon: u16,
    /// obj+0xF4 = param 0 (u16): the arm channel.
    pub arm: u16,
    /// obj+0xFE bit 0: set when fired; the game's delete pass removes the object, so never again.
    pub fired: bool,
}

impl Shooter {
    /// `Shooter_Update` 0x17d0c0. Spawns at most one bullet (the game deletes the object right
    /// after); `weapon_type_3b` is the record's +0x02 halfword (the game re-checks per fire).
    pub fn tick(
        &mut self,
        channels: &crate::triggers::Channels,
        weapons: &nf_formats::weapon::WeaponData,
        rng: &mut crate::drone::Rng,
        out: &mut Vec<crate::bullet::Bullet>,
    ) {
        if self.fired || !channels.on(self.arm as u32) {
            return;
        }
        self.fired = true;
        let Some(w) = weapons.records.get(self.weapon as usize) else { return };
        let f = |o: usize| f32::from_le_bytes(w[o..o + 4].try_into().unwrap());
        // Mat_GetDir(obj+0x90); the 0x3b-class weapons fire straight down (0x17d0c0:20-24).
        let dir: V3 = if u16::from_le_bytes([w[2], w[3]]) == 0x3b {
            [0.0, -1.0, 0.0]
        } else {
            self.dir
        };
        // `Bullet_init` 0x12c5a0: the spread sphere (weapon +0x24, flags +0x68) around the direction,
        // then the record's damage/range/speed (+0x0C/+0x1C/+0x20). The fuse block runs too — the
        // satchel rounds (family +0x02 == 0x3b) carry `((kind − 0x3b)·5 + 5)·60` ticks and the 100
        // burn counter [M 0x12c5a0:199-211].
        let flags68 = u32::from_le_bytes(w[0x68..0x6c].try_into().unwrap());
        let dir = crate::bullet::spread(dir, f(0x24), flags68, rng);
        let mut b = crate::bullet::Bullet {
            pos: self.pos,
            dir,
            speed: f(0x20),
            travelled: 0.0,
            range: f(0x1c),
            damage: f(0x0c),
            owner: usize::MAX,
            state: 0,
            weapon: self.weapon,
            flyby: flags68 & 0x40_0000 != 0,
            bounces: 0,
            glass_pass: crate::bullet::glass_pass_of(w),
            ..crate::bullet::Bullet::TAIL
        };
        b.init_fuse(w);
        out.push(b);
    }
}

/// DynamicObject (placement 0x3b → `DynamicObject_Create` 0x18dbf8, class 0x52): a placement-gated
/// prop with a distance fade. `DynamicObject_Update` 0x18d3b8 drives three effects:
/// - the gate channel A (param 0): when present and off the object is inert (early out);
/// - channel B (param 1) with the mode byte (param 3): mode 0 — armed B deletes the object
///   (obj+0xFE bit 0, the delete pass 0x1336f8 removes it that tick); mode 1 — armed B ORs the
///   draw-flags 0x30 on (obj+0xF0), otherwise restores the glist's base word;
/// - the distance fade (param 2, instance+8): inside the radius obj+0x106 (the alpha byte) rises
///   linearly from 0 at the radius to 255 at half of it.
#[derive(Clone, Debug)]
pub struct DynObject {
    /// The object position (obj+0x80 world sphere centre = the placement translation).
    pub pos: V3,
    /// instance[0] = param 0 (u16): the gate channel.
    pub ch_a: u16,
    /// instance[2] = param 1 (u16): the action channel.
    pub ch_b: u16,
    /// instance[8] = param 2 (u16): the fade radius (`Update` casts it to int; 0 = no fade).
    pub dist: u16,
    /// instance[10] = param 3 (u8): the mode byte.
    pub mode: u8,
    /// obj+0xF0 base = the glist's +0x44 draw-flags word (`DynamicObject_Create` 0x18dbf8:15-16).
    pub base_flags: u32,
    /// obj+0xF0 (the live word: base restored, or base | 0x30 in mode 1 with B armed).
    pub flags: u32,
    /// obj+0x106, the alpha byte written by the distance fade (init 0xFF, object.md +0x103..0x106).
    pub alpha: u8,
    /// obj+0xFE bit 0 (mode 0 + B armed → deleted by the movement pass that tick).
    pub dead: bool,
    /// The create order among the level's 0x3b placements (the draw's posed slot; the delete pass
    /// removes dead objects from the list, so the list index is not stable).
    pub slot: u16,
    /// The collision world's key for the glist; 0 when it has no shape.
    pub shape: u32,
    /// obj+0x90.
    pub matrix: [[f32; 4]; 4],
}

impl DynObject {
    /// `DynamicObject_Update` 0x18d3b8. `viewer` is `glb_viewer+0xC0` (the camera position).
    pub fn tick(&mut self, viewer: V3, channels: &crate::triggers::Channels) {
        if self.ch_a != 0 {
            if !channels.on(self.ch_a as u32) {
                return;
            }
            self.flags = self.base_flags; // *(u32*)(obj+0xF0) = instance[4..6]
        }
        // The mode byte splits B's effect: 0 = delete on armed B, else flags 0x30 while armed.
        if self.mode == 0 {
            if self.ch_b != 0 && channels.on(self.ch_b as u32) {
                self.dead = true; // obj+0xFE |= 1 [M 0x18d3b8:26]
            }
        } else if self.ch_b == 0 || !channels.on(self.ch_b as u32) {
            self.flags = self.base_flags; // 0x18d3b8:32-33, 38-39
        } else {
            self.flags |= 0x30; // 0x18d3b8:34-36
        }
        // The distance fade: alpha = clamp(d/radius − 0.5, 0, …) · 2 · 255 [M 0x18d3b8:40-49].
        if self.dist != 0 {
            let d = dist3(self.pos, viewer);
            if d < self.dist as f32 {
                let f = (d / self.dist as f32 - 0.5).max(0.0);
                self.alpha = ((f + f) * 255.0) as u8;
            }
        }
    }
}

/// ThirdCam (placement 0xe7 → `ThirdCam_Create` 0x1bc510, class 0xC): a third-person camera riding
/// a 0x19 key-list spline. The object registers itself in `ThirdPersonCameras[cam_id]`; the
/// camera/View side picks it by id and rides `ThirdCam_GetPos` 0x1bc590's `Spline_Interp`.
#[derive(Clone, Debug)]
pub struct ThirdCam {
    /// instance[0] = param 0 (u16): the `ThirdPersonCameras` slot.
    pub cam_id: u16,
    /// instance[4] = the level_tag +0x08 pointer: the 0x19 key list (`m_paths[record+0x44]`).
    pub keys: Vec<crate::door::Key>,
    /// instance[8] = the level_tag +0x02 key count.
    pub key_count: u16,
    /// instance[10] = param 1 (u8).
    pub flag: u8,
}

impl ThirdCam {
    /// `ThirdCam_GetPos` 0x1bc590: `Spline_Interp(keys, count·t, 0, count, m, 0, 2)` — the spline
    /// position at parameter t ∈ [0, 1]. The game clamps every neighbour index into [0, count], so
    /// time past the last key rests on the final key; the port rides `door::interp` with the last
    /// key index (its `last` is an inclusive index) and lets the clamp do the same. Render-side
    /// today (no camera consumer in the Sim).
    pub fn get_pos(&self, t: f32) -> Option<crate::door::Mat4> {
        if self.keys.len() < 2 {
            return None;
        }
        let last = (self.key_count.max(1) - 1).min(self.keys.len() as u16 - 1).max(1);
        crate::door::interp(&self.keys, 0, last, self.key_count as f32 * t, true)
    }
}

/// The four families, created from the main map's placements in placement order (the game creates
/// in `parsemap_block_map_data_dynamic` 0x1d1b68's walk order; no RNG in any of the four creates).
#[derive(Clone, Debug, Default)]
/// MusicTrigger (placement 0xfb → `MusicTrigger_Create` 0x1b30e0, class 0xc): while switch
/// channel `ch_on` (p0) is on it re-arms a 30-tick countdown; every tick the countdown runs fires
/// `Music_Event(event_id, 2)` [M 0x1b31a0:19-27] — so the event keeps firing for up to 30 ticks
/// after the channel clears. `ch_flag` (p1) is a kill channel and `flag_byte` (p3) a one-shot byte:
/// both set obj+0xfe bit 0, the delete flag [M 0x1b31a0:15,26-27], and the movement handler's delete
/// pass removes the object that tick [M 0x1336f8:86-87]. So a p3 = 1 trigger posts its event for ONE
/// tick, however long the channel stays on (07000001's event 2, the alarm slot, then decays).
pub struct MusicTrigger {
    /// p2 — the `Music_Event` board slot ([D] 07000001: 2, 07000002: 31/32, 07000007: 13 …).
    pub event_id: u32,
    /// p0 — the switch channel that re-arms the countdown (create skips the object when 0).
    pub ch_on: u16,
    /// p1 — optional kill channel (deletes the object, after this tick's update has run).
    pub ch_flag: u16,
    /// p3.
    pub flag_byte: u8,
    /// The game's countdown short at inst+8 (`0x1e` armed, drained per tick).
    pub countdown: i16,
    /// The game's obj+0xfe bit 0: deleted by the delete pass the same tick.
    pub dead: bool,
}

/// Rotor (placement 0x71 → `rotor_init` 0x18f4b8, `rotor_update` 0x18f568, class 7): a spinning
/// fan. inst+0x00..0x0B holds the placement's ROT vector (spin per tick per axis — 07000001's
/// `comroom_fanblades` carry rot y = 0.0349 rad [D placements]); inst+0x10 = param 1 (u8) is the
/// gate channel. `rotor_update` 0x18f568: when the gate is 0 or the channel reads 0, the object
/// matrix rotation (obj+0x50) += spin·FRAME_RATE_MUL (1.0 at the port's fixed 60 Hz) and
/// obj+0xFA |= 0x20 (redraw). Param 0's slot is decompiler-dropped at the 0x1d04f0 call [S].
#[derive(Clone, Debug)]
pub struct Rotor {
    /// The placement ROT vector (radians per tick per axis).
    pub spin: V3,
    /// inst+0x10 — param 1: the gate channel (0 = always spins).
    pub gate: u16,
    /// The accumulated obj+0x50 rotation — the getter the draw code applies to the fan object.
    pub angle: V3,
}

impl Rotor {
    /// `rotor_update` 0x18f568.
    pub fn tick(&mut self, channels: &crate::triggers::Channels) {
        if self.gate != 0 && channels.on(self.gate as u32) { return; }
        for k in 0..3 { self.angle[k] += self.spin[k]; }
    }
}

/// Mine (placement 0xFE → `Mine_Create` 0x17ce28, class 0x4B): the floating laser tripbomb —
/// 07000015's nine `Tripbomb_3rd_*` cels. `Mine_Update` 0x17c930:
/// - the body bobs: `obj+0x34 = pos.y + sin(GameState+0x52·(param3·0.0002))·(param4·0.1)`
///   [M 0x17c930 disasm 0x17c9a8-0x17c9f4 — the sin's rate argument reads inst+0x14, which
///   `Mine_Create` 0x17ce28:46 fills with param3·0.0002; inst+0x18 = param4·0.1 the amplitude
///   [M 0x17ce28:47]; the clock is the game's global `GameState+0x52` tick — the port's fixed
///   60 Hz frame, no RNG]. The game also accumulates `obj+0x54 += param2·0.0002·FRAME_RATE_MUL`
///   [M 0x17c930 disasm 0x17c978-0x17c988] — the spin angle, wrapped into ±π by
///   `control_movement_object_handler` 0x1336f8 and drawn through the same pose matrix
///   (`View_RotTransMatrix` 0x1e5168 reads obj+0x50..0x58) — which the port does not model
///   [S unverified scope: the bob is this task; the spin stays out].
/// - the explode arm: when the object's HITDATA list (`obj+0xD0`, the collide pass's contact list
///   [M docs/atlas/object.md +0xD0]) holds an entry whose +0x58 references another object, the
///   mine explodes [M 0x17c930:31-45's walk]: `Camera_Shake(param1)` when param1 ≥ 0.5, then an
///   Explode object (class 0xF) built from the mine's instance: damage = inst+0x1C = param 0
///   (float), radius/shake/blast-script-scale = inst+0x20 = param 1, blast script = inst+0x24 =
///   param 5 (07000015: 0x06000052, the standard blast), lifetime = the script's +0xAB0 u16, the
///   script scale `SCRIPTINFO+0x94 = param1`, `obj+0xE8 = param1·0.0667` [M disasm 0x17ca74-0x17cb10],
///   and `obj+0xFE |= 1` deletes the mine that tick [M 0x17cb7c-0x17cb88].
///
/// `Mine_Create` 0x17ce28 reads no RNG. The contact surface is the mine cel's own collision —
/// 07000015's long mine carries 1240 tris spanning ±0.23 x, ±0.10 y, 0..−4.34 local z (the beam)
/// [D probe census_07000015::probe_07000015_mine_objects].
#[derive(Clone, Debug)]
pub struct Mine {
    /// The main-map placement index.
    pub placement: usize,
    /// obj+0x30: the placement translation (the bob wobbles obj+0x34, the y, off this).
    pub pos: V3,
    /// inst+0x1C = param 0 (float): the blast damage.
    pub damage: f32,
    /// inst+0x20 = param 1 (float): the blast radius, the camera shake and the blast-script scale.
    pub radius: f32,
    /// inst+0x14 = param 3 · 0.0002: the bob clock rate (rad per tick) [M 0x17c930 disasm
    /// 0x17c9a8/0x17c9c4 load inst+0x14 into the sin's argument; 0x17ce28:46 fills inst+0x14 from
    /// placement param 3]. (param 2 is the separate obj+0x54 spin rate, not modelled.)
    pub bob_rate: f32,
    /// inst+0x18 = param 4 · 0.1: the bob amplitude.
    pub bob_amp: f32,
    /// inst+0x24 = param 5: the blast script hash.
    pub script: u32,
    /// The mine cel's collision triangles in world space (the contact test's surface — the game's
    /// hit list fills from the collide pass over the same cel).
    pub tris: Vec<[V3; 3]>,
    pub lo: [f32; 3],
    pub hi: [f32; 3],
    /// obj+0xFE bit 0: exploded; the delete pass removes the object that tick.
    pub exploded: bool,
    /// The live posed y (obj+0x34) — the draw-side getter for the bob.
    pub posed_y: f32,
    /// The world sphere (`obj+0x80` centre = the matrix · the glist centre, `obj+0x8C` = the glist
    /// radius · |obj+0xE8| = 1.0 [M Control_BuildWorldSph 0x133070]) — the pick sphere of
    /// `Collide_PickObj` 0x1eae80 and the blast sweep of `Explode_Propagate` 0x17c518 read it.
    pub centre: V3,
    pub sph_r: f32,
    /// A referenced HITDATA entry waiting on the mine's list for the next `Mine_Update` pass —
    /// set by a blast's case 0x4B (`Coll_AddHitToList(…, obj+0xD0, 0, attacker)` [M 0x17c518
    /// case 0x4b]) and by a bullet's mirrored contact (`Collide_Update` 0x1ea398:160-233).
    /// 0 = none, else the `MINE_*` cause.
    pub pending: u8,
    /// What set the mine off (0 = not exploded, else a `MINE_*` cause) — the port's observability
    /// for the tests; the game keeps no such field.
    pub cause: u8,
}

/// `Mine::cause` / `Mine::pending` values: a body contact (the player's / a drone's collide-pass
/// capsule; drones are `MINE_DRONE`), a bullet's mirrored contact, a blast's case-0x4B entry.
pub const MINE_BODY: u8 = 1;
pub const MINE_BULLET: u8 = 2;
pub const MINE_BLAST: u8 = 3;
/// A drone's body contact (the 0.4 body cylinder stand-in).
pub const MINE_DRONE: u8 = 4;

impl Mine {
    /// `Mine_Create` 0x17ce28 over the main map's placements (placement order). No RNG [M].
    /// `tris_of` resolves the placement's object collision (the same closure shape the other
    /// creators take).
    pub fn create(
        placements: &[nf_formats::placement::Placement],
        tris_of: impl Fn(&nf_formats::placement::Placement) -> (Vec<[V3; 3]>, V3, f32),
    ) -> Vec<Mine> {
        let mut out = Vec::new();
        for (i, p) in placements.iter().enumerate() {
            if p.flags & 0xff != 0xfe { continue; }
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
            let (tris, centre, sph_r) = tris_of(p);
            let (mut lo, mut hi) = ([f32::MAX; 3], [-f32::MAX; 3]);
            for t in &tris { for v in t { for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); } } }
            out.push(Mine {
                placement: i,
                pos: p.translation,
                damage: param(0) as i32 as f32,
                radius: param(1) as i32 as f32,
                // The sin's rate is inst+0x14 = param3·0.0002 [M 0x17c930 disasm 0x17c9c8 +
                // 0x17ce28:46] — NOT param2 (that accumulates the obj+0x54 spin angle,
                // [M 0x17c978-0x17c988]).
                bob_rate: param(3) as i32 as f32 * 0.0002,
                bob_amp: param(4) as i32 as f32 * 0.1,
                script: param(5),
                lo, hi,
                tris,
                exploded: false,
                posed_y: p.translation[1],
                centre, sph_r,
                pending: 0,
                cause: 0,
            });
        }
        out
    }

    /// The bob half of `Mine_Update` 0x17c930:9-30 — `posed_y = pos.y + sin(t·rate)·amp` over the
    /// fixed 60 Hz frame counter (the game's GameState+0x52 clock; deterministic, no RNG).
    pub fn bob(&mut self, frame: u32) {
        self.posed_y = self.pos[1] + (frame as f32 * self.bob_rate).sin() * self.bob_amp;
    }

    /// The explode arm's contact test: the collide pass gives the mine a HITDATA entry when a
    /// touching object's capsule (the player body, feet `a` → head `b`, radius `r`; the drones'
    /// 0.4 spheres the same way) meets the mine cel's collision tris [M 0x17c930's hit-list walk +
    /// docs/atlas/object.md +0xD0]. The port's static world already holds the same tris, so a
    /// walked-in Bond rests AT contact distance — the game's own sphere-vs-cel query radius is the
    /// capsule radius, matched here (+0.03 graze epsilon [S]).
    pub fn touching(&self, a: V3, b: V3, r: f32) -> bool {
        if self.tris.is_empty() { return false; }
        if (0..3).any(|k| (a[k] < self.lo[k] - r && b[k] < self.lo[k] - r) || (a[k] > self.hi[k] + r && b[k] > self.hi[k] + r)) { return false; }
        crate::triggers::capsule_hits_tris(a, b, r + 0.03, &self.tris)
    }

    /// A bullet's contact with the mine. The round's per-tick HITTEST is mode 0x201 (the flight
    /// segment) with mask `+0x98 = 0` [M Bullet_update 0x12e7b0 disasm 0x12f040, 0x12f134;
    /// docs/systems/searchlight-bullet.md §1], so `Collide_Pick` 0x1eb320 walks the straddled cels'
    /// object lists (only mask bit 0x10 skips them [M 0x1eb320 LAB_001eb8e8]) and `Collide_PickObj`
    /// 0x1eae80 admits the mine: `obj+0xF0` is 0 (`Mine_Create` 0x17ce28 never writes it), no class
    /// mask bit is set, and the glist carries collision (`obj+0xD8` and `glist+8` non-zero); the
    /// 0x201 arm keeps it when the segment's closest point to the world sphere centre lies inside
    /// `obj+0x8C`. `Collide_Intersect` 0x1ebaa8's 0x201 arm then refines against the glist mesh (the
    /// front-face ray test, `Intersect_RayGeom` 0x10a8c8:168). Returns the nearest hit's segment
    /// fraction; `Collide_Update` 0x1ea398:160-233 mirrors the hit onto the mine's own list with
    /// +0x58 = the bullet — the referenced entry `Mine_Update` 0x17c930 explodes on.
    pub fn segment_t(&self, a: V3, b: V3) -> Option<f32> {
        if self.exploded || self.tris.is_empty() { return None; }
        let sub = |x: V3, y: V3| [x[0] - y[0], x[1] - y[1], x[2] - y[2]];
        let cross = |x: V3, y: V3| [x[1] * y[2] - x[2] * y[1], x[2] * y[0] - x[0] * y[2], x[0] * y[1] - x[1] * y[0]];
        let dot = |x: V3, y: V3| x[0] * y[0] + x[1] * y[1] + x[2] * y[2];
        let d = sub(b, a);
        let dd = dot(d, d);
        let ac = sub(self.centre, a);
        let t = if dd < 1e-12 { 0.0 } else { (dot(ac, d) / dd).clamp(0.0, 1.0) };
        let q = sub([a[0] + d[0] * t, a[1] + d[1] * t, a[2] + d[2] * t], self.centre);
        if dot(q, q) >= self.sph_r * self.sph_r { return None; }
        let mut best: Option<f32> = None;
        for tri in &self.tris {
            let (e1, e2) = (sub(tri[1], tri[0]), sub(tri[2], tri[0]));
            if dot(cross(e1, e2), d) >= 0.0 { continue; } // back face [M 0x10a8c8:168]
            let p = cross(d, e2);
            let det = dot(e1, p);
            if det.abs() < 1e-12 { continue; }
            let inv = 1.0 / det;
            let s = sub(a, tri[0]);
            let u = dot(s, p) * inv;
            if !(0.0..=1.0).contains(&u) { continue; }
            let qv = cross(s, e1);
            let v = dot(d, qv) * inv;
            if v < 0.0 || u + v > 1.0 { continue; }
            let tt = dot(e2, qv) * inv;
            if (0.0..=1.0).contains(&tt) && best.is_none_or(|bt| tt < bt) { best = Some(tt); }
        }
        best
    }

    /// `Explode_Propagate` 0x17c518's reach for one mine: `explode::falloff` over the distance from
    /// the blast to the world sphere's edge must reach 0.0002 [M 0x17c518, the gate before the class switch]; the
    /// case-0x4B arm then hangs a
    /// HITDATA on the mine whose +0x58 is the blast's attacker (`Coll_AddHitToList(…, obj+0xD0, 0,
    /// param_5)` [M 0x17c518 case 0x4b]). A mine's own blast passes `EXPLODEINFO+0x24` = the mine
    /// (`Mine_Update` 0x17c930 stores it, `Explode_CollisionHandler` 0x17cde0 passes it as param_5),
    /// so the entry is referenced and the next `Mine_Update` explodes the neighbour too.
    pub fn blast_reaches(&self, pos: V3, radius: f32, damage: f32) -> bool {
        if self.exploded || !(damage >= 0.0002) || !(radius >= 0.0002) { return false; }
        let d = [self.centre[0] - pos[0], self.centre[1] - pos[1], self.centre[2] - pos[2]];
        let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() - self.sph_r;
        crate::explode::falloff(damage, radius, dist) >= 0.0002
    }
}

/// OneSided wall (placement 0xd5 → `OneSided_Create` 0x1e7ce8, class 0x4C). The creator makes a
/// plain control object (`control_create_object(0, pos, rot, 0)` 0x1e7ce8:9), stores the
/// placement's celglist at obj+0xD8 (0x1e7ce8:14) and the class byte 0x4C at obj+0xff
/// (0x1e7ce8:15), then links it into the room (0x1e7ce8:16). Its params are all unused.
/// `OneSided_Update` 0x1e7d58 (control_funcs[0x4c·0xC] @0x26fc20, run by the update pass
/// 0x1336f8:54-69): dot the object matrix's dir (`Mat_GetDir` 0x1e3398 reads obj+0x90's third
/// column) with (obj pos − viewer) — `glb_viewer+0xC0`, the active camera's world position —
/// and set obj+0xF0 bit 0x10 (hide) when the dot < 0, clear it otherwise (0x1e7d58 C decomp:
/// the `_vsub`/`_vmul` pair + the 3-float sum; the disassembly confirms the same three floats
/// feed the `< 0.0` branch). One whole-object plane flip per tick; no RNG, no channels.
#[derive(Clone, Debug)]
pub struct OneSided {
    /// The object position (obj+0x30 = the placement translation).
    pub pos: V3,
    /// The matrix dir (`Mat_GetDir(obj+0x90)`, the placement's RotZYX forward, column 2).
    pub dir: V3,
    /// The placement index (the join key the draw side needs when it honours the hidden flag).
    pub placement: usize,
    /// obj+0xF0 bit 0x10, refreshed every tick by [`OneSided::tick`] (hidden when facing away).
    pub hidden: bool,
}

impl OneSided {
    /// `OneSided_Update` 0x1e7d58: `dot(dir, pos − viewer) < 0` → obj+0xF0 |= 0x10, else clear.
    /// The viewer is the active camera's world position (`glb_viewer+0xC0`) — the same stand-in
    /// [`DynObject::tick`]'s fade uses (on-foot first-person = the eye).
    pub fn tick(&mut self, viewer: V3) {
        let d = [self.pos[0] - viewer[0], self.pos[1] - viewer[1], self.pos[2] - viewer[2]];
        self.hidden = self.dir[0] * d[0] + self.dir[1] * d[1] + self.dir[2] * d[2] < 0.0;
    }
}

#[derive(Clone, Default)]
pub struct WorldObjects {
    pub rotors: Vec<Rotor>,
    pub inverters: Vec<Inverter>,
    pub shooters: Vec<Shooter>,
    pub dynobjs: Vec<DynObject>,
    pub thirdcams: Vec<ThirdCam>,
    pub music_triggers: Vec<MusicTrigger>,
    pub onesided: Vec<OneSided>,
}

impl WorldObjects {
    /// The placement scan (`p.flags & 0xff` kinds 0x41 / 0xd2 / 0x3b / 0xe7 / 0x71 / 0xfb / 0xd5). `path_blocks` is the
    /// main map's 0x19 key-list payloads (`m_paths`), indexed by the placement's +0x44 word.
    /// `glist` resolves the placement's object to its glist's +0x44 draw-flags word (the word
    /// `DynamicObject_Create` copies to obj+0xF0), its sphere centre (`celglist+0x20..+0x2C`,
    /// docs/fields/glist.md) and its collision shape key — the world sphere obj+0x80 the fade reads starts at the matrix
    /// translation + the glist centre (0x1336f8 pass 6 rebuilds it from the same two).
    pub fn create(
        placements: &[nf_formats::placement::Placement],
        path_blocks: &[&[u8]],
        glist: impl Fn(&nf_formats::placement::Placement) -> (u32, V3, u32),
    ) -> WorldObjects {
        let mut w = WorldObjects::default();
        for p in placements {
            let param = |k: u32| -> u32 { p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0) };
            match p.flags & 0xff {
                0x41 => {
                    // InverterTrigger_Create 0x1b3040: p0 → obj+0xF4, p1 → obj+0xF6 (u16 each).
                    w.inverters.push(Inverter { ch_in: param(0) as u16, ch_out: param(1) as u16 });
                }
                0xd2 => {
                    // Shooter_Create 0x17d030: p0 → obj+0xF4, p1 → obj+0xF6. Its obj+0x90 comes
                    // from CCO(0) → View_RotTransMatrix [M 0x17d030:10, 0x1344e0:26-29].
                    let m = nf_formats::placement::view_rot_trans_matrix(p.rotation, p.translation);
                    w.shooters.push(Shooter {
                        pos: p.translation,
                        // Mat_GetDir(obj+0x90): the matrix's forward column (identity → +z).
                        dir: [m[0][2], m[1][2], m[2][2]],
                        weapon: param(1) as u16,
                        arm: param(0) as u16,
                        fired: false,
                    });
                }
                0xd5 => {
                    // OneSided_Create 0x1e7ce8: a plain control object (CCO(0) → the placement
                    // pos/rot [M 0x1e7ce8:9, 0x1344e0:26-29]), the glist at obj+0xD8, class 0x4C.
                    // The placement's object still draws (the static misc path scene.rs keeps —
                    // the hidden flag's consumer is the draw side, a hand-off stub).
                    let m = nf_formats::placement::view_rot_trans_matrix(p.rotation, p.translation);
                    w.onesided.push(OneSided {
                        pos: p.translation,
                        // Mat_GetDir(obj+0x90): the matrix's dir column (identity → +z).
                        dir: [m[0][2], m[1][2], m[2][2]],
                        placement: p.index as usize,
                        hidden: false,
                    });
                }
                0x3b => {
                    // DynamicObject_Create 0x18dbf8: p0/p1 (u16), p2 (u16), p3 (u8); obj+0xF0 = the
                    // glist's +0x44 word (0x18dbf8:15-16), |0x30 when the p0 SHORT != 0 (0x18dbf8:24,30-32 —
                    // the level_tag slot is written u32 but read `*(short *)(param_3 + 0x2c)`, so only the
                    // low 16 bits gate the OR; the census found no disc placement where the two differ [D
                    // dynobj_p0_census], the unit test pins the short with p0 = 0x10000).
                    let (base, centre, shape) = glist(p);
                    // obj+0x90: CCO(0) → View_RotTransMatrix [M 0x18dbf8:13, 0x1344e0:26-29].
                    let m = nf_formats::placement::view_rot_trans_matrix(p.rotation, p.translation);
                    let slot = w.dynobjs.len() as u16;
                    w.dynobjs.push(DynObject {
                        slot,
                        // obj+0x80 (the world sphere the fade distance reads): the matrix translation
                        // plus the rotated glist centre (Control_BuildWorldSph's steady state).
                        pos: nf_formats::placement::transform(&m, centre),
                        ch_a: param(0) as u16,
                        ch_b: param(1) as u16,
                        dist: param(2) as u16,
                        mode: param(3) as u8,
                        base_flags: base,
                        // p0 as a SHORT [M 0x18dbf8:24] — the OR fires on the low 16 bits only.
                        flags: if param(0) as u16 != 0 { base | 0x30 } else { base },
                        alpha: 0xff,
                        dead: false,
                        shape,
                        matrix: m,
                    });
                }
                0xe7 => {
                    // ThirdCam_Create 0x1bc510: cam id = p0 (u16), key count = the level_tag +0x02
                    // half (placement +0x40), the key list = m_paths[placement+0x44], flag = p1 (u8).
                    let count = (p.extra[7] & 0xffff) as u16;
                    let index = (p.extra[8] & 0xffff) as usize;
                    let keys = path_blocks.get(index).map(|b| crate::door::parse_keys(b)).unwrap_or_default();
                    w.thirdcams.push(ThirdCam { cam_id: param(0) as u16, keys, key_count: count, flag: param(1) as u8 });
                }
                0x71 => {
                    // rotor_init 0x18f4b8: the ROT vector is the spin, param 1 (u8) the gate.
                    w.rotors.push(Rotor { spin: p.rotation, gate: param(1) as u16, angle: [0.0; 3] });
                }
                0xfb => {
                    // MusicTrigger_Create 0x1b30e0:12-16: p2 == 0xff or p0 == 0 → no object
                    // (07000011's placement, params (18, 0, 255, 0), is inert by this gate
                    // [D placement bytes]).
                    let (id, ch) = (param(2), param(0));
                    if id != 0xff && ch != 0 {
                        w.music_triggers.push(MusicTrigger {
                            event_id: id,
                            ch_on: ch as u16,
                            ch_flag: param(1) as u16,
                            flag_byte: param(3) as u8,
                            countdown: 0,
                            dead: false,
                        });
                    }
                }
                _ => {}
            }
        }
        w
    }

    /// One tick, in the game's update-pass order (the DynamicObjList walk 0x1336f8 pass 4): the
    /// inverters, shooters and dynamic objects are plain `control_funcs` updates. `viewer` is the
    /// camera position for the fade; spawned shooter bullets join the drone-bullet list.
    pub fn tick(
        &mut self,
        channels: &mut crate::triggers::Channels,
        viewer: V3,
        weapons: &nf_formats::weapon::WeaponData,
        rng: &mut crate::drone::Rng,
        bullets: &mut Vec<crate::bullet::Bullet>,
        frame: u32,
        music: &mut crate::music::Music,
    ) {
        for inv in &self.inverters {
            inv.tick(channels, frame);
        }
        for r in &mut self.rotors {
            r.tick(channels);
        }
        for sh in &mut self.shooters {
            sh.tick(channels, weapons, rng, bullets);
        }
        for ob in &mut self.dynobjs {
            ob.tick(viewer, channels);
        }
        self.dynobjs.retain(|ob| !ob.dead);
        // OneSided_Update 0x1e7d58, per object (the same update pass): the facing flip only —
        // no channels, no RNG, so its walk order against the other classes is unobservable.
        for os in &mut self.onesided {
            os.tick(viewer);
        }
        // MusicTrigger_Update 0x1b31a0, in its order: the p1 flag arm, the p0 re-arm
        // (`countdown = 0x1e`), then the drain (`countdown != 0` → decrement, `Music_Event(id, 2)`,
        // the p3 flag) — the event board slot reads 2 while the countdown runs and stops 30 ticks
        // after the channel clears.
        for t in &mut self.music_triggers {
            if t.ch_flag != 0 && channels.on(t.ch_flag as u32) { t.dead = true; }
            if t.ch_on != 0 && channels.on(t.ch_on as u32) { t.countdown = 30; }
            if t.countdown != 0 {
                t.countdown -= 1;
                music.event(t.event_id as usize, 2);
                if t.flag_byte != 0 { t.dead = true; }
            }
        }
        self.music_triggers.retain(|t| !t.dead);
    }
}

#[cfg(test)]
mod music_trigger_tests {
    use super::*;
    use crate::triggers::Channels;

    fn place(flags: u32, params: &[(u32, u32)]) -> nf_formats::placement::Placement {
        nf_formats::placement::Placement { index: 0, hashcode: u32::MAX, flags, translation: [0.0; 3], rotation: [0.0; 3], params: params.to_vec(), extra: [0; 9] }
    }

    /// The create gates [M 0x1b30e0:12-16]: p2 == 0xff or p0 == 0 → no object. 07000001's real
    /// record is params (13, 0, 2, 1) and 07000011's is (18, 0, 255, 0) [D placement bytes].
    #[test]
    fn create_gates_and_param_map() {
        let w = WorldObjects::create(&[place(0xfb, &[(0, 13), (1, 0), (2, 2), (3, 1)])], &[], |_| (0, [0.0; 3], 0));
        assert_eq!(w.music_triggers.len(), 1);
        let t = &w.music_triggers[0];
        assert_eq!((t.event_id, t.ch_on, t.ch_flag, t.flag_byte), (2, 13, 0, 1));
        assert_eq!(t.countdown, 0);
        assert!(WorldObjects::create(&[place(0xfb, &[(0, 18), (1, 0), (2, 255), (3, 0)])], &[], |_| (0, [0.0; 3], 0)).music_triggers.is_empty(), "p2 == 0xff → no object");
        assert!(WorldObjects::create(&[place(0xfb, &[(0, 0), (1, 0), (2, 5), (3, 0)])], &[], |_| (0, [0.0; 3], 0)).music_triggers.is_empty(), "p0 == 0 → no object");
    }

    /// `MusicTrigger_Update` 0x1b31a0: the channel re-arms 0x1e; each tick the countdown runs fires
    /// `Music_Event(id, 2)`; after the channel clears the event keeps for the remaining countdown
    /// (≤ 30 ticks) and then stops.
    #[test]
    fn countdown_rearms_fires_and_expires() {
        let mut w = WorldObjects::create(&[place(0xfb, &[(0, 13), (1, 0), (2, 2), (3, 0)])], &[], |_| (0, [0.0; 3], 0));
        let mut ch = Channels::default();
        let mut music = crate::music::Music::default();
        let mut rng = crate::drone::Rng::new(1);
        let (mut weapons, mut bullets) = (nf_formats::weapon::WeaponData::default(), Vec::new());
        // One frame: the control-object pass writes the board, then `UpdateMusicalEvents` consumes
        // and clears it (music.rs's update — the game's EventList is per-frame).
        let mut tick = |w: &mut WorldObjects, ch: &mut Channels, music: &mut crate::music::Music, rng: &mut crate::drone::Rng, bullets: &mut Vec<crate::bullet::Bullet>, f: u32| {
            w.tick(ch, [0.0; 3], &weapons, rng, bullets, f, music);
        };
        let mut consume = |music: &mut crate::music::Music| music.update(0);
        // Channel off: no countdown, no event.
        tick(&mut w, &mut ch, &mut music, &mut rng, &mut bullets, 1);
        assert_eq!(music.events[2], 0);
        consume(&mut music);
        // Channel on: the countdown re-arms every tick; the event fires.
        ch.set(13, 1, 2);
        for f in 2..6 {
            tick(&mut w, &mut ch, &mut music, &mut rng, &mut bullets, f);
            assert_eq!(music.events[2], 2, "the event fires while the channel holds");
            consume(&mut music);
        }
        assert!(w.music_triggers[0].countdown > 0 && w.music_triggers[0].countdown < 30);
        assert!(!w.music_triggers[0].dead, "p3 = 0 keeps the object");
        // Channel off: the countdown drains and the event fires until it hits 0 (the armed 0x1e
        // [M 0x1b31a0:20] — the last fire is the tick the countdown reaches 0).
        ch.set(13, 0, 6);
        let mut last_fire = 0u32;
        for f in 6..50 {
            tick(&mut w, &mut ch, &mut music, &mut rng, &mut bullets, f);
            if music.events[2] == 2 { last_fire = f; }
            consume(&mut music);
        }
        assert_eq!(w.music_triggers[0].countdown, 0, "the countdown expired at the game's tick");
        assert_eq!(last_fire, 6 + 28, "the event stops 0x1e ticks after the re-arm stops");
    }
}

#[cfg(test)]
mod music_trigger_delete_tests {
    use super::*;
    use crate::triggers::Channels;

    fn run(params: &[(u32, u32)], ch: &mut Channels, ticks: u32) -> (WorldObjects, u32) {
        let p = nf_formats::placement::Placement { index: 0, hashcode: u32::MAX, flags: 0xfb, translation: [0.0; 3], rotation: [0.0; 3], params: params.to_vec(), extra: [0; 9] };
        let mut w = WorldObjects::create(&[p], &[], |_| (0, [0.0; 3], 0));
        let (mut music, mut rng) = (crate::music::Music::default(), crate::drone::Rng::new(1));
        let (weapons, mut bullets) = (nf_formats::weapon::WeaponData::default(), Vec::new());
        let mut fires = 0;
        for f in 0..ticks {
            w.tick(ch, [0.0; 3], &weapons, &mut rng, &mut bullets, f, &mut music);
            if music.events[2] == 2 { fires += 1; }
            music.update(0);
        }
        (w, fires)
    }

    /// p3 = 1 [M 0x1b31a0:26-27]: the object posts once and is deleted [M 0x1336f8:86-87].
    #[test]
    fn a_one_shot_trigger_posts_for_one_tick_and_goes() {
        let mut ch = Channels::default();
        ch.set(13, 1, 0);
        let (w, fires) = run(&[(0, 13), (1, 0), (2, 2), (3, 1)], &mut ch, 100);
        assert_eq!(fires, 1, "07000001's record posts once with the channel held 100 ticks");
        assert!(w.music_triggers.is_empty());
    }

    /// p1 [M 0x1b31a0:15]: the kill channel deletes the object, but the update still finishes:
    /// a re-arm on the same tick posts once. Sister: no kill, no delete.
    #[test]
    fn the_kill_channel_lets_the_tick_finish() {
        let mut ch = Channels::default();
        ch.set(13, 1, 0);
        ch.set(0x96, 1, 0);
        let (w, fires) = run(&[(0, 13), (1, 0x96), (2, 2), (3, 0)], &mut ch, 10);
        assert_eq!(fires, 1);
        assert!(w.music_triggers.is_empty());
        let mut ch = Channels::default();
        ch.set(13, 1, 0);
        let (w, fires) = run(&[(0, 13), (1, 0x96), (2, 2), (3, 0)], &mut ch, 10);
        assert_eq!(fires, 10, "the channel holds, the kill channel is off");
        assert_eq!(w.music_triggers.len(), 1);
    }
}

/// `rotor_update` 0x18f568: the object matrix rotation (obj+0x50) accumulates spin·FRAME_RATE_MUL
/// every tick while the gate channel reads 0 (gate 0 = always spins), and freezes while it reads 1.
/// The angle is the draw-side getter (07000011's three fans: #199/obj#128, #205/#206/obj#134).
#[cfg(test)]
mod rotor_tests {
    use super::*;
    use crate::triggers::Channels;

    #[test]
    fn rotor_tick_accumulates_and_gate_stops() {
        let mut r = Rotor { spin: [0.25, 0.0, 0.0], gate: 7, angle: [0.0; 3] };
        let mut ch = Channels::default();
        r.tick(&ch);
        r.tick(&ch);
        assert!((r.angle[0] - 0.5).abs() < 1e-6, "two ticks add 2·spin: {}", r.angle[0]);
        ch.set(7, 1, 1);
        r.tick(&ch);
        assert!((r.angle[0] - 0.5).abs() < 1e-6, "gate on: frozen at {}", r.angle[0]);
        // Sister: gate 0 never freezes, even with a channel set.
        let mut always = Rotor { spin: [0.0, 0.1, 0.0], gate: 0, angle: [0.0; 3] };
        always.tick(&ch);
        assert!((always.angle[1] - 0.1).abs() < 1e-6, "gate-0 rotor ignores the channels");
    }
}

#[cfg(test)]
mod mine_tests {
    use super::*;

    /// A closed axis box (outward-wound) as the mine's mesh: the beam x 0..4, y −0.1..0.1, z ±0.2.
    fn beam_mine() -> Mine {
        let (lo, hi) = ([0.0f32, -0.1, -0.2], [4.0f32, 0.1, 0.2]);
        let c = |i: usize| [if i & 1 != 0 { hi[0] } else { lo[0] }, if i & 2 != 0 { hi[1] } else { lo[1] }, if i & 4 != 0 { hi[2] } else { lo[2] }];
        // Six faces, two tris each, wound so the normal points out.
        let quads = [[0, 2, 3, 1], [4, 5, 7, 6], [0, 1, 5, 4], [2, 6, 7, 3], [0, 4, 6, 2], [1, 3, 7, 5]];
        let mut tris = Vec::new();
        for q in quads { tris.push([c(q[0]), c(q[1]), c(q[2])]); tris.push([c(q[0]), c(q[2]), c(q[3])]); }
        Mine { placement: 0, pos: [0.0; 3], damage: 80.0, radius: 15.0, bob_rate: 0.0, bob_amp: 0.0, script: 0,
            tris, lo, hi, exploded: false, posed_y: 0.0, centre: [2.0, 0.0, 0.0], sph_r: 2.02, pending: 0, cause: 0 }
    }

    #[test]
    fn a_round_through_the_beam_hits_it_and_a_miss_does_not() {
        let m = beam_mine();
        // Down onto the beam's middle from above: the top face (front-facing) at y 0.1.
        let t = m.segment_t([2.0, 5.0, 0.0], [2.0, -5.0, 0.0]).expect("the round crosses the beam");
        assert!((t - 0.49).abs() < 1e-4, "the top face at y 0.1 is t 0.49, got {t}");
        // Sister: 0.3 beside the beam — inside the pick sphere, but the mesh refine misses.
        assert!(m.segment_t([2.0, 5.0, 0.5], [2.0, -5.0, 0.5]).is_none(), "a miss beside the beam");
        // Sister: a segment that stops short of the beam.
        assert!(m.segment_t([2.0, 5.0, 0.0], [2.0, 1.0, 0.0]).is_none(), "the flight ends short");
        // An exploded mine takes nothing.
        let mut gone = beam_mine();
        gone.exploded = true;
        assert!(gone.segment_t([2.0, 5.0, 0.0], [2.0, -5.0, 0.0]).is_none());
    }

    #[test]
    fn the_blast_reach_is_explode_propagates_falloff() {
        let m = beam_mine();
        // dist − sph_r: 16.02 − 2.02 = 14.0 → 80·(1 − 14/15) > 0: reached [M 0x17c518].
        assert!(m.blast_reaches([2.0, 16.02, 0.0], 15.0, 80.0));
        // 17.03 − 2.02 = 15.01 → below 0: not reached (sister).
        assert!(!m.blast_reaches([2.0, 17.03, 0.0], 15.0, 80.0));
        // Inside the sphere: full reach; a zero radius reaches nothing.
        assert!(m.blast_reaches([2.0, 0.0, 0.0], 10.0, 80.0));
        assert!(!m.blast_reaches([2.0, 0.0, 0.0], 0.0, 80.0));
    }
}

/// The hidden-dynobj carry (dynobj-hidden-skip): `DynamicObject_Create` 0x18dbf8 ORs 0x30 into
/// obj+0xF0 when the p0 SHORT is non-zero, and `DynamicObject_Update` 0x18d3b8 restores the raw
/// glist word / re-ORs 0x30 with the mode-1 channel. The 0x20 bit of that word is in every
/// `Collide_PickObj` skip set [M 0x1eae80:72-82], so a 0x30 object is never picked for any ray.
#[cfg(test)]
mod dynobj_hidden_tests {
    use super::*;
    use crate::triggers::Channels;

    fn place(params: &[(u32, u32)]) -> nf_formats::placement::Placement {
        nf_formats::placement::Placement { index: 0, hashcode: u32::MAX, flags: 0x3b, translation: [0.0; 3], rotation: [0.0; 3], params: params.to_vec(), extra: [0; 9] }
    }

    /// The real 07000007 rope barrier's word is 0x0 and the Dynamic Blocker's is 0x10 [D
    /// dynobj_p0_census]; 0x81 is the probe's solar-panel word. None carries 0x10/0x20 bits the
    /// OR would fold, so the expected words are exact.
    fn create(params: &[(u32, u32)]) -> DynObject {
        WorldObjects::create(&[place(params)], &[], |_| (0x81, [0.0; 3], 7)).dynobjs.remove(0)
    }

    /// `DynamicObject_Create` 0x18dbf8:15-16 copies the glist's +0x44 word; :24,30-32 OR 0x30 when
    /// the p0 SHORT differs from zero — a p0 whose low 16 bits are 0 (0x10000) must NOT set them.
    /// (The old port tested the full u32: red there, green since the short read.)
    #[test]
    fn create_or_the_hide_word_only_for_a_nonzero_p0_short() {
        let base = 0x81u32;
        assert_eq!(create(&[]).flags, base, "p0 absent: the raw glist word [M 0x18dbf8:15-16]");
        assert_eq!(create(&[(0, 0)]).flags, base, "p0 == 0: the raw glist word");
        assert_eq!(create(&[(0, 21)]).flags, base | 0x30, "p0 = 21 (07000007 pl 495 [D]): +0x30");
        assert_eq!(create(&[(0, 1)]).flags, base | 0x30, "p0 = 1 (07000041 DynoRod [D]): +0x30");
        assert_eq!(create(&[(0, 0x10000)]).flags, base, "p0 = 0x10000: the SHORT read is 0 [M 0x18dbf8:24]");
    }

    /// The create-time OR never adds bits beyond 0x30: a p0 = 9 'Dynamic Blocker' whose glist word
    /// already carries 0x10 (07000007 pl 496 [D]) ends at word | 0x30 (0x10 folds into it).
    #[test]
    fn the_or_folds_into_a_sight_through_base_word() {
        let w = WorldObjects::create(&[place(&[(0, 9)])], &[], |_| (0x10, [0.0; 3], 7)).dynobjs.remove(0);
        assert_eq!(w.base_flags, 0x10);
        assert_eq!(w.flags, 0x10 | 0x30);
        assert_eq!(w.ch_a, 9, "the gate channel is the same p0 [M 0x18d3b8:9]");
    }

    /// `DynamicObject_Update` 0x18d3b8:9-15: a gated object frozen while its channel reads off, and
    /// restored to the RAW word (the create-time OR gone) once it reads on — the record's word was
    /// stored before the OR [M 0x18dbf8:21-23]. So a channel-on swap console becomes ray-pickable.
    #[test]
    fn gate_on_restores_the_raw_word_and_gate_off_freezes_the_create_word() {
        // ch_a = 120, base 0 (the real 07000014 CONSOLE_Camera_Inactive record [D pl 801-813]).
        let mut w = WorldObjects::create(&[place(&[(0, 120)])], &[], |_| (0, [0.0; 3], 7)).dynobjs.remove(0);
        assert_eq!(w.flags, 0x30, "hidden at create");
        let mut ch = Channels::default();
        w.tick([0.0; 3], &ch);
        assert_eq!(w.flags, 0x30, "channel off: the update returns, the create word stays [M 0x18d3b8:11-13]");
        ch.set(120, 1, 2);
        w.tick([0.0; 3], &ch);
        assert_eq!(w.flags, 0, "channel on: obj+0xF0 = the record's raw word [M 0x18d3b8:14-15]");
    }

    /// Mode 1 (0x18d3b8:36-43): armed B ORs 0x30 back onto the live word; B off restores the raw
    /// record word — so the channel toggles the object between ray-hidden and ray-pickable.
    #[test]
    fn mode_one_toggles_the_hide_word_with_channel_b() {
        // 07000005's WallHug blockers are mode-1 records (p3 = 1) but ungated (p0 = 0) [D pl 324/325].
        let mut w = WorldObjects::create(&[place(&[(1, 5), (3, 1)])], &[], |_| (0x10, [0.0; 3], 7)).dynobjs.remove(0);
        let mut ch = Channels::default();
        w.tick([0.0; 3], &ch);
        assert_eq!(w.flags, 0x10, "B off: the raw word [M 0x18d3b8:40-43]");
        ch.set(5, 1, 2);
        w.tick([0.0; 3], &ch);
        assert_eq!(w.flags, 0x10 | 0x30, "B armed: +0x30 [M 0x18d3b8:36-38]");
        ch.set(5, 0, 4);
        w.tick([0.0; 3], &ch);
        assert_eq!(w.flags, 0x10, "B disarmed: the raw word again");
    }

    /// The word 0x30 the create ORs is what `Collide_PickObj` 0x1eae80:72-82 never picks: bit 0x20
    /// sits in EVERY skip set. The game-side consequence (no part in the collision set) is pinned
    /// end-to-end by dynobj_hidden_live; here the skip-word math itself.
    #[test]
    fn the_hidden_word_is_admitted_by_no_mask() {
        for mask in [0u32, 0x1, 0x2, 0x4, 0x8, 0xc, 0xe, 0x20, 0xa27, 0x72f] {
            assert!(!crate::world::pick_admits(mask, 0x30), "mask {mask:#x} must skip a 0x30 word");
            assert!(!crate::world::pick_admits(mask, 0x20), "mask {mask:#x} must skip a bare 0x20 word");
        }
        // Sister: a plain word (0x81, the probe's solar panel) is picked by every mask; the bare
        // sight-through 0x10 only by masks without bit 4 (the game's own hidden-flag [M :79-81]).
        for mask in [0u32, 0xa27, 0xc, 0xe, 0x72f] {
            assert!(crate::world::pick_admits(mask, 0x81), "mask {mask:#x} admits a plain word");
        }
        assert!(!crate::world::pick_admits(0xa27, 0x10), "mask bit 4 skips the 0x10 word");
        assert!(crate::world::pick_admits(0x2, 0x10), "a mask without bit 4 admits the 0x10 word");
    }
}
