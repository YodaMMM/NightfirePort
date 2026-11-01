//! The photograph targets and the breakable doors: placement 0xFD (`CamSubject_Create` 0x1bc5d8,
//! `CamSubject_Update` 0x1bc668) and 0xE1 (`Destroy_Create` 0x12c318, `Destroy_CollisionHandler`
//! 0x12bd48, `Destroy_Update` 0x12c4f8).
//!
//! The game's contact model: the collision pass registers every object overlapping the prop's glist
//! into its contact list (obj+0xD0); the per-tick update walks that list:
//! - `CamSubject_Update` 0x1bc668 writes `switch_channels[obj+0xF6] = 1` (obj+0xF6 = placement
//!   param 0 [M 0x1bc5d8:26]) for every contacting object of class 5 (a bullet, obj+0xFF == 5)
//!   whose weapon byte reads 0x54 — the camera gadget ([M 0x1bc668:14-24: the bullet's BU datum's
//!   weapon_data+2 == 0x54]). The 0700000c jetpack display (#414/#454) writes ch 0x1c and the
//!   0700000d laser display (#494/#495) ch 0x1e [D placements].
//! - `Destroy_CollisionHandler` 0x12bd48 subtracts each contacting bullet's damage from the tag's
//!   health (+0x04 = placement param 3 [M 0x12c318:36]); at health < 0 it writes
//!   `switch_channels[tag+0x12] = 1` (tag+0x12 = placement param 0 [M 0x12c318:35]) and smashes.
//!   The 0700000d "C5" doors (#93/#96 `bigdoor01/02`) carry ch 0x1f and health 1000/10000
//!   [D placements; Destroy_Update 0x12c4f8's p6 countdown arm is not ported — inert there, p6 = 0].
//!   The second arm IS ported (`Destroy::kill_channel`, `Props::channel_kills`): every tick a live
//!   destroyable whose +0x10 word (placement param 1 [M 0x12c318:36 — `piVar1+4 = tag+0x30`]) names
//!   an ON channel smashes (the word then clears [M 0x12c4f8:32]). It is inert on 0700000d (no
//!   writer for the tag+0x10 channel exists on that disc's placements [D]) but NOT on 07000008:
//!   the GuardRoom SmallExplosionBox #234 carries +0x10 = 137 [D the census p1 = 0x89] and the
//!   FUSEBOX_GuardRoomDoorLock #219 writes ch 137 [D the census] — cutting the fusebox smashes the
//!   box (996-level-08-decomp-audit; the old "out ch 137" reading of #234 in
//!   docs/levels/07000008.md had the direction reversed: +0x12/param 0 = 0 is its death write).
//!
//! The port checks the bullet's flight segment against each prop's bounds box (the glist bounds
//! the collision pass would register) in `Drones::update_player_bullets` — one check per tick per
//! bullet, the channel latching on the first overlap, as the game's contact list does.

use crate::triggers::Channels;
use crate::world::V3;
use nf_formats::level::Level;
use nf_formats::placement::{object_matrix_for_class, transform};

#[derive(Clone, Debug)]
pub struct CamSubject {
    pub placement: usize,
    pub lo: V3,
    pub hi: V3,
    /// `obj+0xF6` — the channel the photograph writes [M 0x1bc5d8:26].
    pub channel: u8,
    pub hit: bool,
}

#[derive(Clone, Debug)]
pub struct Destroy {
    pub placement: usize,
    /// The placement's translation — the object's obj+0x30, where `Destroy_Smash` 0x12bb60 sets
    /// off its blast. A leaf with no collision bounds (0700000d's bigdoor01) blasts here.
    pub pos: V3,
    /// The object's glist sphere radius (the level object's `radius`, the same value the doors'
    /// sphere fields carry) — the blast's size base. 0 when the object has none (the box
    /// half-diagonal stands in then).
    pub sphere: f32,
    pub lo: V3,
    pub hi: V3,
    /// `tag+0x12` — the channel the death writes [M 0x12c318:35].
    pub channel: u8,
    /// `tag+0x04` — the bullet-damage pool [M 0x12c318:36, 0x12bd48:44-48].
    pub health: f32,
    pub dead: bool,
    /// `tag+0x16` — the smash type byte (placement param 2's low byte [M 0x12c3c8 disasm
    /// `lbu v1,52(s0)` → `sb v1,22(a0)`]): `Destroy_Smash` 0x12bc50-0x12bd20 routes the blast
    /// script on it — 1 → 0x6000027 (radius ×4), 2 → 0x600004f (×2), other non-zero → 0x6000052
    /// (×3); a type-0 tag plays NO script (`if (cVar4 != 0)` [M 0x12bc50]).
    pub kind: u8,
    /// `tag+0x08` — the smash blast's damage float (placement param 5 [M 0x12c424 disasm
    /// `lw v0,64(s0)` → decoded → tag+8]; `Destroy_Smash` passes it as `Explode_Create`'s f13-slot
    /// damage [M 0x12bd04 `lwc1 f13,8(s1)`]). The disc's destroyables all carry small positive
    /// ints here (10…3000 [D census]), the only branch the port decodes. `Destroy_Create` stores
    /// them ×0.01 (`tag+8 = (float)param5 * 0.01` [M 0x12c318:48]), so the blasts deal 0.1…30.
    pub blast_damage: f32,
    /// `tag+0x10` — the external-kill channel (placement param 1 [M 0x12c318:36]): the second arm
    /// of `Destroy_Update` 0x12c4f8:30-35 smashes the tag when the channel is ON and clears the
    /// word. 0 = inert. #234 on 07000008 carries 137 (the fusebox's channel) [D the census].
    pub kill_channel: u16,
    /// `tag+0x14` — the bullet category mask [M 0x12c318:47-52 `*(param+0x3c) == 0 ? 0 : ~*(u16*)`]:
    /// `Collide_FilterBullets` 0x1ece30:26-33 drops every class-5 contact whose
    /// `weapon_data+0x10` word ANDs non-zero against it, and `Explode_Propagate` 0x17c518's case
    /// 0x21 smashes on a blast only when `& 0x20 == 0`. The 0700000d C5 doors carry p4 0x10 →
    /// 0xFFEF [D #93/#96]: handheld gunfire (0x08/0x0020 words) drops, the mounted guns 0x61/0x67/
    /// 0x70 (0x0010) pass [D weapon_data], and no blast reaches the smash.
    pub mask: u16,
    /// `obj+0x104`/`obj+0x105` — the hit-flash colour pair [M Destroy_CollisionHandler
    /// 0x12bd48:61-95]: a weapon-0x67 contact tick drops both by 5 (clamped at 0), every other
    /// handler tick recovers both by 1 (clamped at 255); the death return skips the step so the
    /// pair freezes. [L PCSX2 2026-10-27: 00ff0000 under the beam, ~3/frame recovery once it
    /// leaves — the port steps per handler call, one per tick.] The draw reads it for the hit tint.
    pub colour: [u8; 2],
    /// Tick scratch: a class-5 contact whose weapon is 0x67 reached this leaf this tick (the
    /// colour step's drop half [M 0x12bd48:32-34]). Cleared by `destroy_colour_tick`.
    pub hit67: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Props {
    pub subjects: Vec<CamSubject>,
    pub destroy: Vec<Destroy>,
    /// The mounted-gun implementations (`GunImp_Create` 0x131f20, placement 0x34).
    pub gunimps: Vec<crate::gunimp::GunImp>,

}

/// One placement's target object's bounds placed by the placement transform (the game's contact
/// is against the placed glist), in placement order. Objects whose gfx bounds are unset (anim-only
/// meshes: min/max stay at the parser's ±1e7 sentinel) fall back to their collision triangles.
/// The placement object's glist sphere radius (0 when it has no object).
fn object_radius(level: &Level, m: usize, p: &nf_formats::placement::Placement) -> f32 {
    let obj = if p.by_hash() { level.by_hash.get(&p.hashcode).map(|&(map, object)| &level.maps[map].objects[object]) }
        else { level.maps[m].objects.get(p.index as usize) };
    obj.map_or(0.0, |o| if o.radius.is_finite() && o.radius > 0.0 { o.radius } else { 0.0 })
}

pub fn bounds(level: &Level, m: usize, p: &nf_formats::placement::Placement) -> (V3, V3) {
    let obj = if p.by_hash() { level.by_hash.get(&p.hashcode).map(|&(map, object)| &level.maps[map].objects[object]) }
        else { level.maps[m].objects.get(p.index as usize) };
    let Some(o) = obj else { return ([0.0; 3], [0.0; 3]) };
    // Per-class builder: bounds() is reached from breakable placements (0x20, CCO(0) →
    // View_RotTransMatrix [M 0x12c230:12]); the class predicate keeps any other caller on the
    // game's own builder for its class [M placement::object_matrix_for_class].
    let mx = object_matrix_for_class(p.flags & 0xff, p.creates_cel(), p.rotation, p.translation, p.quaternion());
    let mut corners = Vec::new();
    let sane = [o.min, o.max].iter().all(|b| b.iter().all(|v| v.abs() < 1.0e6));
    if sane {
        for x in [o.min[0], o.max[0]] { for y in [o.min[1], o.max[1]] { for z in [o.min[2], o.max[2]] {
            corners.push([x, y, z]);
        } } }
    } else if let Some(c) = o.collision.as_ref() {
        for t in c.triangles() { for v in t { corners.push(v); } }
    }
    if corners.is_empty() { return ([0.0; 3], [0.0; 3]); }
    let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for v in corners {
        let v = transform(&mx, v);
        for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); }
    }
    (lo, hi)
}

impl Props {
    /// `GunImp_PlayerHasDied` 0x133008 (100 B, called from `Player_CheckForDeath` 0x190810:3-10
    /// right after the death marker): walk the objects for the class-0x47 ('G') gun implant whose
    /// linked player is the dead one and `GunImp_Deactivate` it [M 0x133008:4-17; the class byte
    /// at obj+0xff from `GunImp_Create` 0x131f20, the linked player at obj+0xe0 -> +4]. The port
    /// is single-player (the SP-only laser mount), so every active imp is Bond's: drop the mount —
    /// the same `active` flag the use-key dismount clears (drone.rs `gunimp_tick`).
    /// `GunImp_Deactivate`'s bond-side restore (cam mode, the saved substate, the velocity zeroes
    /// [M 0x132c18:33-47]) rides the death flow's own takeover of the view.
    pub fn gunimp_player_has_died(&mut self) {
        for g in self.gunimps.iter_mut() {
            g.active = false;
        }
    }

    /// `CamSubject_Create` / `Destroy_Create` over the main map's placements (placement order, like
    /// the game's `parsemap_create_dynamic_objects`).
    pub fn create(level: &Level) -> Props {
        let Some(m) = level.main_map() else { return Props::default() };
        let mut props = Props::default();
        for (i, p) in level.maps[m].placements.iter().enumerate() {
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
            let (lo, hi) = bounds(level, m, p);
            match p.flags & 0xff {
                // `CamSubject_Create` 0x1bc5d8: class 0x48, obj+0xF6 = placement +0x2C (param 0).
                0xfd => props.subjects.push(CamSubject { placement: i, lo, hi, channel: param(0) as u8, hit: false }),
                // `Destroy_Create` 0x12c318: tag+0x12 = param 0, tag+0x04 (health) = param 3
                // (as f32; the tag holds a float the handler subtracts into), tag+0x16 (kind) =
                // param 2's low byte, tag+0x08 (smash blast damage) = param 5, tag+0x14 (the
                // bullet mask) = ~param 4 with 0 staying 0 [M 0x12c318:47-52 disasm]. The colour
                // pair starts white (recovery clamps at 255 [M 0x12bd48:62-68]).
                0xe1 => props.destroy.push(Destroy { placement: i, pos: p.translation, sphere: object_radius(level, m, p), lo, hi, channel: param(0) as u8, health: param(3) as f32, dead: false, kind: (param(2) & 0xff) as u8, blast_damage: param(5) as f32 * 0.01,
                    kill_channel: param(1) as u16,
                    mask: (param(4) == 0).then_some(0).unwrap_or(!(param(4) as u16)),
                    colour: [0xff, 0xff], hit67: false }),
                // `GunImp_Create` 0x131f20 (placement 0x34, class 0x47): the SP arm mounts the
                // TurretGuns[0xb] hardware (the laser) [M 0x131f20:52-63]; the base transform is
                // `Quat_QuatToMat` on the placement's +0x24 quaternion with `Mat_SetTrans` writing
                // the position into the translation row [M 0x131f20:66-67] — that prebuilt matrix
                // then overwrites the created object's at `Control_CreateObjEx` 0x1343c8:18-23
                // (the class's matrix builder runs the same quat path, see
                // placement::object_matrix_for_class). The port's runtime keeps the yaw-only base
                // model (gunimp.rs): every 0x34 disc row is yaw-only with a consistent pure-yaw
                // quat (identity or a π yaw — max |quat − euler| 4.8e-7 over all 37 rows
                // [D placement_rot_census]), so `yaw_base = rotation[1]` equals the quat's yaw on
                // all disc data — pinned per row by the census test.
                0x34 => {
                    // The seat offset's source word: the glist's +0x28 float = the object record's
                    // min.z (the bounds block copies at the +8 shift; see gunimp.rs's seat doc)
                    // [M GunImp_Create 0x131f20:77-81; D #225 min.z = -1.4362069]. The by-hash
                    // map alone is not enough: the load's session mesh merge re-points shared
                    // hashes at foreign objects with sentinel-scale bounds, so the candidates are
                    // scanned for the sane-bounds art (the placement's own box [D]: |min.z| < 2).
                    let seat = {
                        let by_ref = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else { None };
                        let mut seat = 0.0f32;
                        if let Some(o) = if p.by_hash() { None } else { level.maps[m].objects.get(p.index as usize) } {
                            seat = o.min[2] * 0.75;
                        } else {
                            for mp in &level.maps {
                                for o in &mp.objects {
                                    if o.hashcode == p.hashcode && o.min.iter().all(|v| v.abs() < 100.0) {
                                        seat = o.min[2] * 0.75;
                                        break;
                                    }
                                }
                                if seat != 0.0 { break; }
                            }
                            if seat == 0.0 {
                                if let Some((mm, oo)) = by_ref {
                                    seat = level.maps[mm].objects[oo].min[2] * 0.75;
                                }
                            }
                        }
                        seat
                    };
                    props.gunimps.push(crate::gunimp::GunImp {
                    beam: None,
                    fire_handle: 0,
                        placement: i, pos: p.translation, yaw_base: p.rotation[1],
                        pitch: 0.0, yaw: 0.0, pitch_s: 0.0, yaw_s: 0.0,
                        active: false, heat: 0.0, cooldown: 0.0, hot: false,
                        turret: crate::gunimp::SP_TURRET as u8,
                        seat,
                        // The use-key pick's candidate radius: the placement object's gfx sphere
                        // (the same resolution the seat's sane-bounds scan uses) [M 0x1eae80:219-228].
                        radius: { let mut r = 0.0f32;
                            if let Some(o) = if p.by_hash() { None } else { level.maps[m].objects.get(p.index as usize) } { r = o.radius; }
                            else { for mp in &level.maps { for o in &mp.objects {
                                if o.hashcode == p.hashcode && o.min.iter().all(|v| v.abs() < 100.0) { r = o.radius; break; } }
                                if r != 0.0 { break; } } }
                            r },
                    });
                }
                _ => {}
            }
        }
        props
    }

    fn seg_box(a: V3, b: V3, lo: V3, hi: V3) -> bool {
        // The segment's AABB against the prop's bounds grown by 0.1 — the game's contact registers
        // on the bullet's sphere touching the glist bounds, and the world surface often coincides
        // with the box's near face (the flight dies there first).
        let (g, mut mn, mut mx) = (0.1, [0.0; 3], [0.0; 3]);
        for k in 0..3 {
            mn[k] = a[k].min(b[k]);
            mx[k] = a[k].max(b[k]);
            if mx[k] < lo[k] - g || mn[k] > hi[k] + g { return false; }
        }
        true
    }

    /// The bullet's flight this tick against the props ([M CamSubject_Update 0x1bc668 — class-5
    /// contacts with weapon 0x54 write the channel; Destroy_CollisionHandler 0x12bd48 — every
    /// class-5 contact subtracts its damage, death writes the channel]). Returns the tick's
    /// destroyable smashes that carry a blast (`Destroy_Smash` 0x12bb60's `Explode_Create` for a
    /// non-zero type byte): `(smash position, radius, damage, kind)` — the caller queues the blast
    /// (the game's blast runs inside the smash, on the bullet's owner as the attacker).

    /// The 0x54 camera-subject arm of `bullet_hit`, split out so the SP bullet pass can run it
    /// beside the merged nearest-solid contact rule (the subjects are a shot DETECTOR, not a
    /// solid contact — the round is not consumed by them).
    pub fn camera_shot_hit(&mut self, a: V3, b: V3, weapon: u16, ch: &mut Channels, frame: u32) {
        for s in self.subjects.iter_mut().filter(|s| !s.hit) {
            if weapon == 0x54 && Self::seg_box(a, b, s.lo, s.hi) {
                s.hit = true;
                ch.set(s.channel as u32, 1, frame);
            }
        }
    }

    /// `Destroy_Smash` 0x12bb60's death half for leaf `i`: the one-shot latch, the dead mark
    /// (obj+0xff = 1 [M :33/:41]), the channel write `switch_channels[+0x12] = 1` [M :40 — the
    /// same write `Destroy_CollisionHandler` 0x12bd48:42 reaches through the bullet-contact path]
    /// and the type-script blast [M :58-76: kind 1 → 0x6000027 ×4, kind 2 → 0x600004f ×2, other
    /// non-zero → 0x6000052 ×3; the port keeps its established radius/damage stand-ins, see the
    /// bullet_hit note]. Returns the blast tuple the caller queues.
    pub fn smash(&mut self, i: usize, ch: &mut Channels, frame: u32) -> Option<(V3, f32, f32, u8)> {
        let d = self.destroy.get_mut(i)?;
        if d.dead { return None; }
        d.dead = true;
        // The channel-0 arm [M 0x12bb60:34-37]: a tag whose channel word is 0 only flips
        // obj+0xff (the class byte) — no switch_channels write. The write is the else arm's
        // (0x12bb60:40), for a non-zero channel.
        if d.channel != 0 { ch.set(d.channel as u32, 1, frame); }
        if d.kind == 0 { return None; }
        // The box centre stands in for the object's position; a leaf without bounds (the C5
        // pair's bigdoor01) has none, so it blasts at its placement — at the box's (0,0,0) its
        // fireball was set off at the world origin.
        let c = if d.lo == d.hi { d.pos } else { [(d.lo[0] + d.hi[0]) / 2.0, (d.lo[1] + d.hi[1]) / 2.0, (d.lo[2] + d.hi[2]) / 2.0] };
        let ext = [(d.hi[0] - d.lo[0]) / 2.0, (d.hi[1] - d.lo[1]) / 2.0, (d.hi[2] - d.lo[2]) / 2.0];
        let sphere = if d.sphere > 0.0 { d.sphere } else { (ext[0] * ext[0] + ext[1] * ext[1] + ext[2] * ext[2]).sqrt() };
        // A (near-)zero sphere makes a zero blast: radius ~0 reaches nothing and the script plays at
        // scale 0 [M 0x12bb60: radius and scale both from tag+0x8c]. Not spawned — the draw shows
        // the newest blast of a script, so a same-tick empty one hid the C5 fireball.
        if sphere < 0.05 { return None; }
        let mult = match d.kind { 1 => 4.0f32, 2 => 2.0, _ => 3.0 };
        Some((c, sphere * mult, d.blast_damage, d.kind))
    }

    /// `weapon_word` is the round's `weapon_data+0x10` category word [M 0x1ece30:26] — the
    /// caller reads it off the same record the game's hitdata carries. Per leaf the game first
    /// filters (`Collide_FilterBullets(list, tag+0x14)`: a class-5 contact whose word ANDs
    /// non-zero is dropped — set to the 1e8 dead distance and removed from the list [M
    /// 0x1ece30:30, the 0x4cbebc20 store]), so a dropped contact deals NOTHING and never
    /// reaches the handler's walk or the colour arms.
    pub fn bullet_hit(&mut self, a: V3, b: V3, weapon: u16, weapon_word: u16, damage: f32, ch: &mut Channels, frame: u32) -> Vec<(V3, f32, f32, u8)> {
        let mut smashes = Vec::new();
        self.camera_shot_hit(a, b, weapon, ch, frame);
        for i in 0..self.destroy.len() {
            if self.destroy[i].dead { continue; }
            // The filter [M 0x1ece30:26-33]: the contact dies before the handler's walk.
            if weapon_word & self.destroy[i].mask != 0 { continue; }
            let (hit, health) = {
                let d = &self.destroy[i];
                (Self::seg_box(a, b, d.lo, d.hi), d.health - damage)
            };
            if hit {
                // The colour arm's weapon read [M 0x12bd48:32 — the contact's weapon id word ==
                // 0x67 sets the drop step]: this tick's flash decision, applied once per tick.
                if weapon == 0x67 { self.destroy[i].hit67 = true; }
                self.destroy[i].health = health;
                if health < 0.0 {
                    // `Destroy_CollisionHandler` 0x12bd48:31-49 drains HITDATA+8 per class-5
                    // contact and reaches the death through `Destroy_Smash` — the channel write
                    // and the blast are the smash's [M 0x12bb60:40,:58-76]. The handler RETURNS
                    // here, so the death tick skips the colour step (the freeze at death).
                    if let Some(s) = self.smash(i, ch, frame) { smashes.push(s); }
                }
            }
        }
        smashes
    }

    /// The colour pair's once-per-tick step, the tail of every non-death
    /// `Destroy_CollisionHandler` call [M 0x12bd48:61-95]: a weapon-0x67 contact this tick
    /// (kind-5, past the filter) drops both bytes 5 (clamped 0), any other tick recovers both
    /// 1 (clamped 255 — the run's `if (iVar6 == 0)` recover arm covers the no-contact ticks
    /// too, and kind-5 contacts of other weapons damage WITHOUT stopping the recovery). Dead
    /// leaves freeze: the death return skips the step and the smashed object's handler never
    /// runs again (class byte 0x01 [M 0x12bb60] has no collision handler).
    pub fn destroy_colour_tick(&mut self) {
        for d in self.destroy.iter_mut().filter(|d| !d.dead) {
            for k in 0..2 {
                d.colour[k] = if d.hit67 { d.colour[k].saturating_sub(5) } else { d.colour[k].saturating_add(1) };
            }
            d.hit67 = false;
        }
    }

    /// `Destroy_Update` 0x12c4f8's second arm (30-35): every tick a live destroyable whose +0x10
    /// word names an ON channel smashes and the word clears. The smash is the game's full
    /// `Destroy_Smash` 0x12bb60 [M 0x12c4f8:35], so it WRITES the tag's death channel
    /// `switch_channels[+0x12] = 1` for a non-zero channel [M 0x12bb60:40] — the same write the
    /// bullet path's smash makes. On 07000014 the generator-hall cascade runs through exactly
    /// those writes: box #726 (kill ch 0x51, death 0x52 [D the census]) → GENERATORS_Intact #1113
    /// (kill 0x52, death 0x4e) → box #727 (kill 0x4e, death 0x53) → generator #1152 (0x5c) → box
    /// #728 (0xaf) → generator #1154 (0x5a) → box #730 (0xc9) → generator #1153 (0x5b) → box #729
    /// (kill 0x5b, death **0x85** — Door_Slide_11A/B's lock channel [D placements; M 0x1363c8
    /// reads +0x68]). The walk order is the control walk's newest-first object order [M
    /// 0x1336f8:54-68, the order drone.rs' dispatch sorts by], so one cascade link propagates
    /// per tick. Returns the same blast tuples as `bullet_hit`.
    pub fn channel_kills(&mut self, ch: &mut Channels, frame: u32) -> Vec<(V3, f32, f32, u8)> {
        let mut smashes = Vec::new();
        for i in (0..self.destroy.len()).rev() {
            if self.destroy[i].dead || self.destroy[i].kill_channel == 0 { continue; }
            if !ch.on(self.destroy[i].kill_channel as u32) { continue; }
            self.destroy[i].kill_channel = 0; // the game clears the word [M 0x12c4f8:32]
            if let Some(s) = self.smash(i, ch, frame) { smashes.push(s); }
        }
        smashes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn box_props() -> Props {
        Props {
            subjects: vec![CamSubject { placement: 0, lo: [4.0, 7.0, -111.0], hi: [6.0, 9.0, -109.0], channel: 0x1c, hit: false }],
            destroy: vec![Destroy { placement: 1, pos: [0.0; 3], sphere: 0.0, lo: [133.0, 4.0, -270.0], hi: [137.0, 7.0, -268.0], channel: 0x1f, health: 100.0, dead: false,
            kind: 1, blast_damage: 100.0, kill_channel: 0, mask: 0xffef, colour: [0xff, 0xff], hit67: false }],
            gunimps: Vec::new(),
        }
    }

    #[test]
    fn gunimp_player_has_died_drops_the_mount() {
        // `GunImp_PlayerHasDied` 0x133008: the dead player's active mount deactivates.
        let mut p = box_props();
        p.gunimps.push(crate::gunimp::GunImp {
            placement: 0, pos: [0.0; 3], yaw_base: 0.0, pitch: 0.0, yaw: 0.0, pitch_s: 0.0, yaw_s: 0.0,
            active: true, heat: 0.0, cooldown: 0.0, hot: false, turret: 0xb, seat: 0.0, radius: 0.0,
            beam: None,
            fire_handle: 0,
        });
        p.gunimp_player_has_died();
        assert!(!p.gunimps[0].active);
        // Sister: an idle mount stays idle and an empty list does not panic.
        p.gunimp_player_has_died();
        assert!(!p.gunimps[0].active);
        let mut empty = box_props();
        empty.gunimp_player_has_died();
    }

    #[test]
    fn camera_bullet_writes_the_subject_channel() {
        let mut p = box_props();
        let mut ch = Channels::default();
        // The camera's (weapon 0x54) shot through the 0700000c jetpack display's box.
        p.bullet_hit([5.0, 8.0, -112.0], [5.0, 8.0, -108.0], 0x54, 0x0100, 1.0, &mut ch, 10);
        assert!(ch.on(0x1c), "the photograph writes ch 0x1c [M 0x1bc668:20-22]");
        // A sister: the same shot with a gun does nothing (the weapon byte gates the subject).
        let mut p = box_props();
        let mut ch = Channels::default();
        p.bullet_hit([5.0, 8.0, -112.0], [5.0, 8.0, -108.0], 0x1e, 0x0020, 1.0, &mut ch, 10);
        assert!(!ch.on(0x1c), "a non-camera bullet never photographs the subject");
    }

    #[test]
    fn bullets_whittle_the_door_until_it_writes_its_channel() {
        let mut p = box_props();
        p.destroy[0].mask = 0; // the generic p4-0 door: the filter passes everything [M 0x12c318:49-51]
        let mut ch = Channels::default();
        // 9 hits of 10 damage leave the 100-health door standing; the 10th lands exactly on 0.0,
        // which the game's `fVar9 < 0.0` does NOT count [M 0x12bd48:46].
        for f in 0..10 {
            p.bullet_hit([135.0, 5.5, -270.5], [135.0, 5.5, -267.5], 0x1e, 0x0020, 10.0, &mut ch, f);
            assert!(!ch.on(0x1f), "hit {f}: the door stands");
            assert!(!p.destroy[0].dead);
        }
        // The 11th crosses below zero → the death channel.
        p.bullet_hit([135.0, 5.5, -270.5], [135.0, 5.5, -267.5], 0x1e, 0x0020, 10.0, &mut ch, 10);
        assert!(ch.on(0x1f), "the death writes ch 0x1f [M 0x12bd48:51-57]");
        assert!(p.destroy[0].dead);
        // A sister: a shot that misses the box does nothing.
        let mut p = box_props();
        let mut ch = Channels::default();
        p.bullet_hit([20.0, 5.5, -270.5], [20.0, 5.5, -267.5], 0x1e, 0x0020, 10.0, &mut ch, 1);
        assert!(!ch.on(0x1f) && !p.destroy[0].dead, "a miss never damages the door");
    }

    /// `Collide_FilterBullets` 0x1ece30:26-33 with the C5 doors' tag+0x14 = 0xFFEF [D #93/#96
    /// p4 0x10; M 0x12c318:47-52]: handheld gunfire's category word (0x0008 [D weapon_data
    /// 0x08], 0x0020 [D 0x2e]) ANDs non-zero → the contact is dropped and the door takes
    /// nothing; the mounted guns 0x61/0x67/0x70 carry 0x0010 [D] → they pass.
    #[test]
    fn the_c5_mask_drops_handheld_rounds_and_passes_the_mounted_guns() {
        let mut ch = Channels::default();
        // A handheld gun's round straight through the door: no drain, no colour mark.
        let mut p = box_props();
        p.bullet_hit([135.0, 5.5, -270.5], [135.0, 5.5, -267.5], 0x08, 0x0008, 7.0, &mut ch, 1);
        assert_eq!(p.destroy[0].health, 100.0, "the 0x08 word meets 0xFFEF — the contact is dropped [M 0x1ece30:26-33]");
        p.destroy_colour_tick();
        assert_eq!(p.destroy[0].colour, [0xff, 0xff], "a dropped contact is no contact — the recovery arm runs [M 0x12bd48:61-68]");
        // The mounted gun's word passes: the same round drains.
        p.bullet_hit([135.0, 5.5, -270.5], [135.0, 5.5, -267.5], 0x67, 0x0010, 50.0, &mut ch, 2);
        assert_eq!(p.destroy[0].health, 50.0, "the 0x0010 word passes the 0xFFEF mask [D]");
        assert!(p.destroy[0].hit67, "a 0x67 contact sets the flash drop");
        // An unmasked door (p4 0 → tag+0x14 = 0 [M 0x12c318:49-51]) takes everything.
        let mut p = box_props();
        p.destroy[0].mask = 0;
        p.bullet_hit([135.0, 5.5, -270.5], [135.0, 5.5, -267.5], 0x08, 0x0008, 7.0, &mut ch, 3);
        assert_eq!(p.destroy[0].health, 93.0, "mask 0 filters nothing [M 0x1ece30 with param_2 = 0]");
    }

    /// The colour pair's per-tick step [M Destroy_CollisionHandler 0x12bd48:61-95]: a 0x67
    /// contact tick drops both bytes 5, other ticks recover 1 (clamped both ends), and the
    /// death freeze leaves the pair where the smash caught it. [L PCSX2 2026-10-27: 00ff0000
    /// under the beam, ~3/frame recovery measured on display frames.]
    #[test]
    fn the_colour_pair_drops_under_the_beam_and_recovers_after() {
        let mut ch = Channels::default();
        let mut p = box_props();
        // 51 contact ticks take the pair 0xff → 0 (255 = 51·5 − 0: saturating at 0).
        for f in 0..51 {
            p.bullet_hit([135.0, 5.5, -270.5], [135.0, 5.5, -267.5], 0x67, 0x0010, 0.5, &mut ch, f);
            p.destroy_colour_tick();
        }
        assert_eq!(p.destroy[0].colour, [0x00, 0x00], "51 drops of 5 saturate the pair at 0 [M 0x12bd48:71-95]");
        // The beam leaves: recovery +1 a tick.
        for _ in 0..59 { p.destroy_colour_tick(); }
        assert_eq!(p.destroy[0].colour, [0x3b, 0x3b], "59 recoveries of 1 [L 00ff3b3b]");
        for _ in 0..196 { p.destroy_colour_tick(); }
        assert_eq!(p.destroy[0].colour, [0xff, 0xff], "recovery clamps at 255 [M 0x12bd48:62-68]");
        // Death freezes the pair: the smash tick skips the step.
        p.bullet_hit([135.0, 5.5, -270.5], [135.0, 5.5, -267.5], 0x67, 0x0010, 200.0, &mut ch, 52);
        assert!(p.destroy[0].dead);
        p.destroy_colour_tick();
        assert_eq!(p.destroy[0].colour, [0xff, 0xff], "a dead leaf never steps again [M 0x12bd48:49 return]");
        // A non-0x67 contact damages WITHOUT stopping the recovery (iVar6 stays 0).
        let mut p = box_props();
        p.bullet_hit([135.0, 5.5, -270.5], [135.0, 5.5, -267.5], 0x33, 0x0010, 5.0, &mut ch, 1);
        p.destroy_colour_tick();
        assert_eq!(p.destroy[0].health, 95.0, "the 0x33 round (word 0x0010, [D]) drains");
        assert_eq!(p.destroy[0].colour, [0xff, 0xff], "no 0x67 contact — the recover arm [M 0x12bd48:61]");
    }

    /// `Destroy_Update` 0x12c4f8's second arm (30-35): a live destroyable whose +0x10 word
    /// (placement param 1 [M 0x12c318:36 `piVar1+4 = tag+0x30`]) names an ON channel smashes and
    /// the word clears. The smash is `Destroy_Smash` [M 0x12c4f8:35], which writes the death
    /// channel for a non-zero +0x12 [M 0x12bb60:40]. 07000008's GuardRoom SmallExplosionBox #234
    /// carries +0x10 = 137 [D the census p1 = 0x89] — the FUSEBOX_GuardRoomDoorLock's channel
    /// [D the census] — and a zero death channel (nothing to write); 07000014's cascade boxes
    /// carry non-zero deaths, so the write is pinned here too.
    #[test]
    fn the_external_kill_channel_smashes_the_tag_and_clears() {
        let mut p = box_props();
        p.destroy[0].kill_channel = 137;
        let mut ch = Channels::default();
        // The channel off: the tag stands (the arm reads the channel every tick).
        let s = p.channel_kills(&mut ch, 10);
        assert!(s.is_empty() && !p.destroy[0].dead, "the channel off — the tag stands [M 0x12c4f8:30-31's ON read]");
        // The channel on: the smash, with the type-1 blast (the same routing the bullet smash uses).
        ch.set(137, 1, 10);
        let s = p.channel_kills(&mut ch, 10);
        assert_eq!(s.len(), 1, "the channel kill smashes [M 0x12c4f8:32-35 → Destroy_Smash]");
        assert_eq!(s[0].3, 1, "the kind rides the blast tuple (the type-1 script 0x06000027 ×4 radius [M 0x12bc70-0x12bc98])");
        assert!(p.destroy[0].dead, "the tag is dead");
        assert!(ch.on(0x1f), "the death channel rides the remote smash [M 0x12c4f8:35 → 0x12bb60:40]");
        // The word cleared: a second pass never re-smashes [M 0x12c4f8:32's `+0x10 = 0`].
        let s = p.channel_kills(&mut ch, 11);
        assert!(s.is_empty(), "the word cleared — no re-smash [M 0x12c4f8:32]");
        // A zero word is inert (every disc box without the listener).
        let mut p = box_props();
        let s = p.channel_kills(&mut ch, 12);
        assert!(s.is_empty() && !p.destroy[0].dead, "kill_channel 0 is inert");
    }

    /// The remote kills run in the control walk's newest-first order [M 0x1336f8:54-68], so a
    /// chain of kill→death links propagates one link per tick even when the next tag's kill
    /// channel is written by an earlier-in-the-list tag this same tick.
    #[test]
    fn the_remote_kill_cascade_propagates_one_link_per_tick_newest_first() {
        let mut p = Props::default();
        // Two tags: placement 0 kills on ch 5 and dies writing ch 6; placement 1 (newer) kills on
        // ch 6 and dies writing ch 7. A single pass after ch 5 rises smashes only the OLDER tag.
        p.destroy.push(Destroy { placement: 0, pos: [0.0; 3], sphere: 0.0, lo: [0.0; 3], hi: [1.0; 3], channel: 6, health: 1.0, dead: false, kind: 0, blast_damage: 0.0, kill_channel: 5, mask: 0, colour: [0xff, 0xff], hit67: false });
        p.destroy.push(Destroy { placement: 1, pos: [0.0; 3], sphere: 0.0, lo: [2.0; 3], hi: [3.0; 3], channel: 7, health: 1.0, dead: false, kind: 0, blast_damage: 0.0, kill_channel: 6, mask: 0, colour: [0xff, 0xff], hit67: false });
        let mut ch = Channels::default();
        ch.set(5, 1, 1);
        let s = p.channel_kills(&mut ch, 1);
        assert!(s.is_empty(), "a kind-0 smash returns no blast [M 0x12bc50's kind gate]");
        assert!(p.destroy[0].dead && ch.on(6), "the older tag dies and writes its death ch [M 0x12bb60:40]");
        assert!(!p.destroy[1].dead, "the newer tag (checked first, newest-first) waits a tick [M 0x1336f8:54-68]");
        p.channel_kills(&mut ch, 2);
        assert!(p.destroy[1].dead && ch.on(7), "the cascade links one tick later");
    }
}
