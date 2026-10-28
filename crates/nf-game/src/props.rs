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
                // param 2's low byte, tag+0x08 (smash blast damage) = param 5 [M disasm].
                0xe1 => props.destroy.push(Destroy { placement: i, lo, hi, channel: param(0) as u8, health: param(3) as f32, dead: false, kind: (param(2) & 0xff) as u8, blast_damage: param(5) as f32 * 0.01,
                    kill_channel: param(1) as u16 }),
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
        ch.set(d.channel as u32, 1, frame);
        if d.kind == 0 { return None; }
        let c = [(d.lo[0] + d.hi[0]) / 2.0, (d.lo[1] + d.hi[1]) / 2.0, (d.lo[2] + d.hi[2]) / 2.0];
        let ext = [(d.hi[0] - d.lo[0]) / 2.0, (d.hi[1] - d.lo[1]) / 2.0, (d.hi[2] - d.lo[2]) / 2.0];
        let sphere = (ext[0] * ext[0] + ext[1] * ext[1] + ext[2] * ext[2]).sqrt();
        let mult = match d.kind { 1 => 4.0f32, 2 => 2.0, _ => 3.0 };
        Some((c, sphere * mult, d.blast_damage, d.kind))
    }

    pub fn bullet_hit(&mut self, a: V3, b: V3, weapon: u16, damage: f32, ch: &mut Channels, frame: u32) -> Vec<(V3, f32, f32, u8)> {
        let mut smashes = Vec::new();
        self.camera_shot_hit(a, b, weapon, ch, frame);
        for i in 0..self.destroy.len() {
            if self.destroy[i].dead { continue; }
            let (hit, health) = {
                let d = &self.destroy[i];
                (Self::seg_box(a, b, d.lo, d.hi), d.health - damage)
            };
            if hit {
                self.destroy[i].health = health;
                if health < 0.0 {
                    // `Destroy_CollisionHandler` 0x12bd48:31-49 drains HITDATA+8 per class-5
                    // contact and reaches the death through `Destroy_Smash` — the channel write
                    // and the blast are the smash's [M 0x12bb60:40,:58-76].
                    if let Some(s) = self.smash(i, ch, frame) { smashes.push(s); }
                }
            }
        }
        smashes
    }

    /// `Destroy_Update` 0x12c4f8's second arm (30-35): every tick a live destroyable whose +0x10
    /// word names an ON channel smashes and the word clears. Returns the same blast tuples as
    /// `bullet_hit` (the smash is `Destroy_Smash` 0x12bb60 — the same blast routing).
    pub fn channel_kills(&mut self, ch: &Channels) -> Vec<(V3, f32, f32, u8)> {
        let mut smashes = Vec::new();
        for d in self.destroy.iter_mut().filter(|d| !d.dead && d.kill_channel != 0) {
            if !ch.on(d.kill_channel as u32) { continue; }
            d.kill_channel = 0; // the game clears the word [M 0x12c4f8:32]
            d.dead = true;
            if d.kind != 0 {
                let c = [(d.lo[0] + d.hi[0]) / 2.0, (d.lo[1] + d.hi[1]) / 2.0, (d.lo[2] + d.hi[2]) / 2.0];
                let ext = [(d.hi[0] - d.lo[0]) / 2.0, (d.hi[1] - d.lo[1]) / 2.0, (d.hi[2] - d.lo[2]) / 2.0];
                let sphere = (ext[0] * ext[0] + ext[1] * ext[1] + ext[2] * ext[2]).sqrt();
                let mult = match d.kind { 1 => 4.0f32, 2 => 2.0, _ => 3.0 };
                smashes.push((c, sphere * mult, d.blast_damage, d.kind));
            }
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
            destroy: vec![Destroy { placement: 1, lo: [133.0, 4.0, -270.0], hi: [137.0, 7.0, -268.0], channel: 0x1f, health: 100.0, dead: false,
            kind: 1, blast_damage: 100.0, kill_channel: 0 }],
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
        p.bullet_hit([5.0, 8.0, -112.0], [5.0, 8.0, -108.0], 0x54, 1.0, &mut ch, 10);
        assert!(ch.on(0x1c), "the photograph writes ch 0x1c [M 0x1bc668:20-22]");
        // A sister: the same shot with a gun does nothing (the weapon byte gates the subject).
        let mut p = box_props();
        let mut ch = Channels::default();
        p.bullet_hit([5.0, 8.0, -112.0], [5.0, 8.0, -108.0], 0x1e, 1.0, &mut ch, 10);
        assert!(!ch.on(0x1c), "a non-camera bullet never photographs the subject");
    }

    #[test]
    fn bullets_whittle_the_door_until_it_writes_its_channel() {
        let mut p = box_props();
        let mut ch = Channels::default();
        // 9 hits of 10 damage leave the 100-health door standing; the 10th lands exactly on 0.0,
        // which the game's `fVar9 < 0.0` does NOT count [M 0x12bd48:46].
        for f in 0..10 {
            p.bullet_hit([135.0, 5.5, -270.5], [135.0, 5.5, -267.5], 0x1e, 10.0, &mut ch, f);
            assert!(!ch.on(0x1f), "hit {f}: the door stands");
            assert!(!p.destroy[0].dead);
        }
        // The 11th crosses below zero → the death channel.
        p.bullet_hit([135.0, 5.5, -270.5], [135.0, 5.5, -267.5], 0x1e, 10.0, &mut ch, 10);
        assert!(ch.on(0x1f), "the death writes ch 0x1f [M 0x12bd48:51-57]");
        assert!(p.destroy[0].dead);
        // A sister: a shot that misses the box does nothing.
        let mut p = box_props();
        let mut ch = Channels::default();
        p.bullet_hit([20.0, 5.5, -270.5], [20.0, 5.5, -267.5], 0x1e, 10.0, &mut ch, 1);
        assert!(!ch.on(0x1f) && !p.destroy[0].dead, "a miss never damages the door");
    }

    /// `Destroy_Update` 0x12c4f8's second arm (30-35): a live destroyable whose +0x10 word
    /// (placement param 1 [M 0x12c318:36 `piVar1+4 = tag+0x30`]) names an ON channel smashes and
    /// the word clears. 07000008's GuardRoom SmallExplosionBox #234 carries +0x10 = 137 [D the
    /// census p1 = 0x89] — the FUSEBOX_GuardRoomDoorLock's channel [D the census].
    #[test]
    fn the_external_kill_channel_smashes_the_tag_and_clears() {
        let mut p = box_props();
        p.destroy[0].kill_channel = 137;
        let mut ch = Channels::default();
        // The channel off: the tag stands (the arm reads the channel every tick).
        let s = p.channel_kills(&ch);
        assert!(s.is_empty() && !p.destroy[0].dead, "the channel off — the tag stands [M 0x12c4f8:30-31's ON read]");
        // The channel on: the smash, with the type-1 blast (the same routing the bullet smash uses).
        ch.set(137, 1, 10);
        let s = p.channel_kills(&ch);
        assert_eq!(s.len(), 1, "the channel kill smashes [M 0x12c4f8:32-35 → Destroy_Smash]");
        assert_eq!(s[0].3, 1, "the kind rides the blast tuple (the type-1 script 0x06000027 ×4 radius [M 0x12bc70-0x12bc98])");
        assert!(p.destroy[0].dead, "the tag is dead");
        // The word cleared: a second pass never re-smashes [M 0x12c4f8:32's `+0x10 = 0`].
        let s = p.channel_kills(&ch);
        assert!(s.is_empty(), "the word cleared — no re-smash [M 0x12c4f8:32]");
        // A zero word is inert (every disc box without the listener).
        let mut p = box_props();
        let s = p.channel_kills(&ch);
        assert!(s.is_empty() && !p.destroy[0].dead, "kill_channel 0 is inert");
    }
}
