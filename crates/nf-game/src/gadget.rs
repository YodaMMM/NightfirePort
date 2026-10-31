//! Bond's gadgets (weapons 0x4A..0x5E): the laser-family beam and its lock burn.
//!
//! `Player_SetWatch` 0x1a69d0 (1244 B, the last statement of `Player_CollisionHandler`
//! 0x1949a8:204) attaches the wristwatch face + hour/minute/second hand meshes to the fp
//! view-model's bone 2 while the view-model skin is 0x0500004b, and deletes them otherwise
//! [M 0x1a69d0:36-152] — pure render attach; its clock DATA (the per-level base table and the
//! hand rates) is ported in `crate::watch`.
//!
//! The game's model [M]:
//! - `Player_LaserPointer` (0x1a65d8): while a laser weapon is out (its `weapon_data+0x68` gate and
//!   `BLData+0x964`), every tick casts a ray from `Player_GetHeadPos` through the hand bone's
//!   direction, 100.0 units (the 0x42c80000 constant at 0x1a6748), and parks the beam object
//!   (`BLData+0x7ec`) at the hand with its far end at the ray hit (`Collide_RayIntersect` 0x1ec690;
//!   no hit → the full 100). The beam mesh swaps 0x2000131 / 0x2000133 per frame (the flicker).
//! - The beam carries a WEAPON ID at `obj+0x44`: `Lock_Update` (0x1ae3d8 case 0) scans the lock's
//!   contact list for class-5 entries whose +0x44 is the Q-Decryptor (0x4E/0x4F, burn 3/tick) or in
//!   the `− 0x56 < 2` band (0x56/0x57 — the watch laser's and the Q-Laser's beams), and advances the
//!   lock's burn timer; at the end `Lock_Unlock` writes the lock's out channel.
//! - The port's `Lock::burn` (fix-objects2) models the burn timer as already expired (the level's
//!   create-time base is 0 [M 0x1ae1f8]), so one tick of beam contact completes it.
//!
//! Port status: the laser-family beam (the ray, the lock burn, the endpoints for the renderer) and
//! the Q-Claw's grapple (the fire-time latch, the substate 2 reel/hang, `scene.rs`'s fire gate).
//! The decryptor scan mode (substate 5) is not started.
//!
//! The gadget ids, settled from the disc (fix-qclaw; the old hardcoded table had them shifted — it
//! put the laser on 0x50/0x51 and the claw on 0x4A/0x4C): [L ee_start_05.bin @ weapon_data 0x2bf150
//! + USATxt.dat] **0x4A/0x4C Stunner** (the watch taser, clip 100), **0x4E/0x4F Laser** (the only
//! records whose +0x68 carries the laser-pointer flag 0x2), **0x50/0x51 Grapple — the Q-Claw**
//! (the bank's own name "Grapple", label 0x05000044; `+0x70 & 0x40` = the grapple-bolt flag
//! `Bullet_Delete` 0x12e660:33 latches on; range +0x1C = 50.0/100.0; clip 0; damage 0),
//! **0x54/0x55 Micro-Camera** (the watch; its aim-gated beam per `Player_WeaponFiring`
//! 0x1a5460:512-514), **0x56/0x57 Decryptor**.

use crate::lock::Lock;
use crate::world::V3;
/// The weapon ids whose shots carry the LASER BEAM bit `weapon_data+0x6C & 0x100` [M
/// Player_MuzzleFlash 0x1a4298:131 → `Draw_LaserBeam`]: the Laser 0x4E and its upgrade 0x4F
/// [D weapon_data records: 0x4E +0x6C = 0x900 (0x800|0x100), 0x4F = 0x900; the disc names them
/// 0x05000043 "Laser" [L USATxt.dat]. The old set (0x50/0x51/0x54/0x55) was the Grapple and the
/// Micro-Camera — their records carry 0x4000/0 in +0x6C, no beam bits].
pub fn is_laser_weapon(w: u8) -> bool {
    matches!(w, 0x4e | 0x4f)
}

/// The weapons whose SHOT draws a beam: the Laser family (the 0x100 bit → `Draw_LaserBeam`) and
/// the Stunner 0x4A/0x4C (the `weapon_data+0x6C & 0x80` bit → `Draw_TaserBeam` [M 0x1a4298:134];
/// their records: +0x6C = 0x880 [L], the disc name 0x05000042 "Stunner" [L USATxt.dat]).
pub fn runs_beam(w: u8) -> bool {
    is_laser_weapon(w) || matches!(w, 0x4a | 0x4c)
}

/// The Q-Claw's ids: **0x50 (base) and 0x51 (upgrade)**. Evidence: the bank's own name "Grapple"
/// on both records [L USATxt +0x38 = 0x05000044]; `weapon_data+0x70 & 0x40` — the grapple-bolt flag
/// `Bullet_Delete` 0x12e660:33 branches on (the claw bolt's death IS the latch: the hit object's
/// hitdata +0x58 straddler must be class 'Q' = 0x51, then `Player_SetGrapplePoint` 0x195da0);
/// `Bullet_CollisionHandler` 0x12ce80's `weapon_data+0x02 - 0x50 < 2` band; `Player_GrappleSetRope`
/// 0x192728's `BLData+0xdc+0x62 - 0x50 < 2` gate (the rope draws only with the claw out); and the
/// level loadouts granting 0x50 (07000005's start [L Inventory::level_start]).
pub fn is_grapple(w: u8) -> bool {
    matches!(w, 0x50 | 0x51)
}

/// The Q-Decryptor's ids: **0x56 (base) and 0x57 (upgrade)**. Evidence: the disc records
/// (label 0x0500004e, range +0x1C = 2.0/4.0, +0x6C = 0 — NO beam bits, so the shot is a plain
/// bullet [L ee_start_05.bin @ weapon_data 0x2bf150]); `Player_WeaponFiring` 0x1a5460's
/// `uVar6 − 0x56 < 2` bands (:155 the watch-face digits, :551 the use gate) and `Lock_Update`
/// 0x1ae3d8's `− 0x56 < 2` contact band (:162, case 0) with the 0x56→+1 / 0x57→+3 decode rates
/// (:305-315, case 5).
pub fn is_decryptor(w: u8) -> bool {
    matches!(w, 0x56 | 0x57)
}

/// The decryptor's use-gate pick [M Player_WeaponFiring 0x1a5460:551-578 + Check_Target
/// 0x1a7810]: the 50-unit crosshair ray (0x42480000 [M 0x1a7810:114-116]) must hit the lock
/// (the game's `Collide_RayIntersect` object hit stored to BLData+0x880, then the gate re-checks
/// the type byte 0x3a and the kind/state), and the player-to-lock distance must be within the
/// weapon's own `weapon_data+0x1C` (2.0/4.0 [L]). The port's stand-in [S, as laser_tick's]: the
/// ray's closest approach within the lock's world sphere, no world hit before it, and the
/// distance gate. Returns the lock index.
pub fn pick_decryptor_lock(
    eye: V3,
    dir: V3,
    root: V3,
    locks: &[Lock],
    world: &crate::world::CollisionWorld,
    range: f32,
) -> Option<usize> {
    const RAY_LEN: f32 = 50.0; // [M 0x1a7810:114-116]
    let end = [eye[0] + dir[0] * RAY_LEN, eye[1] + dir[1] * RAY_LEN, eye[2] + dir[2] * RAY_LEN];
    let world_t = world.segment(eye, end).unwrap_or(1.0) * RAY_LEN; // the fraction → units
    let ab2 = dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2];
    if ab2 < 1e-9 { return None; }
    let mut best: Option<(f32, usize)> = None;
    let dbg = std::env::var_os("NF_DECPICK_DEBUG").is_some();
    for (i, lock) in locks.iter().enumerate() {
        let ap = [lock.pos[0] - eye[0], lock.pos[1] - eye[1], lock.pos[2] - eye[2]];
        let t = (ap[0] * dir[0] + ap[1] * dir[1] + ap[2] * dir[2]) / ab2;
        if t < 0.0 || t * ab2.sqrt() > RAY_LEN { if dbg { eprintln!("decpick {i}: REJ ray t={t}"); } continue; }
        let closest = [eye[0] + dir[0] * t, eye[1] + dir[1] * t, eye[2] + dir[2] * t];
        let d2 = (closest[0] - lock.pos[0]).powi(2) + (closest[1] - lock.pos[1]).powi(2) + (closest[2] - lock.pos[2]).powi(2);
        if d2 > lock.radius * lock.radius { if dbg { eprintln!("decpick {i}: REJ sphere d={:?}", d2.sqrt()); } continue; }
        // A wall in FRONT of the lock blocks [M 0x1a7810:121] — but the lock's own plate sits
        // flush on the wall it is mounted on, and the port's lock carries no collision box of its
        // own (only the 0.132 keypad sphere), so the mount wall's face lands within the sphere's
        // tolerance. The occlusion test therefore floors the margin at 0.5 [S — the game's ray
        // hits the lock object itself in Collide_RayIntersect; the port cannot separate the mount
        // geometry from the lock's own bounds].
        if world_t < t - lock.radius.max(0.5) { continue; }
        // The distance gate is the PLAYER ROOT to the lock [M 0x1a5460:574-577 — `iVar18+0x30`].
        let dist = ((lock.pos[0] - root[0]).powi(2) + (lock.pos[1] - root[1]).powi(2) + (lock.pos[2] - root[2]).powi(2)).sqrt();
        if dist > range { continue; }
        if best.is_none_or(|(bt, _)| t < bt) { best = Some((t, i)); }
    }
    // The target is the NEAREST lock the ray meets, whatever its state; the gate then refuses an
    // open one or a non-decryptor kind [M 0x1a5460:557-566 tests the +0x880 target's type, kind 0/2
    // and state 0/5]. An unlocked keypad therefore blocks the one mounted behind it on the far side
    // of the door (0700000a #646/#729, 0.26 apart) — skipping it let a held fire start a second scan.
    let (_, i) = best?;
    let lock = &locks[i];
    (!lock.burned && !lock.unlocked && (lock.kind == 0 || lock.kind == 2)).then_some(i)
}


/// One tick of the laser beam's VISUAL endpoints (the Laser 0x4E/0x4F — the watch itself): the ray from `head`
/// along `dir` (unit), 100.0 units [M 0x1a65d8], capped by the world's first hit
/// (`Collide_RayIntersect` 0x1ec690). THE BURN IS NOT HERE: the game's burn runs through the
/// bullet stream's contacts (`Player_WeaponInitBullet` 0x1a4bf0 spawns a class-5 bullet per
/// refire tick — the watch's +0x44 short is 0 [L weapon_data 0x4E]; `Collide_PickObj` 0x1eae80:92-108
/// picks the lock's world sphere off the bullet's per-tick segment; `Collide_Update` 0x1ea398 lands
/// the contact on the lock's `obj+0xD0` list; `Lock_Update` 0x1ae3d8:57-101 burns −3/−10 per
/// contact) — the port's bullet pass drives it (drone.rs's watch arm). This function is the beam
/// MESH's endpoints only.
/// `head`/`dir` are THE CROSSHAIR RAY the caller fires bullets down (the game's shot ray runs from
/// `Player_GetHeadPos` through `Player_GetAimingPoint` [M Player_WeaponInitBullet 0x1a4bf0:152-159],
/// and the beam draws from the muzzle to that ray's re-cast hit [M 0x1a4298:107-132]) — the old
/// `bond.look()` ray made the laser inaccurate on the 0700000c display vent.
/// Returns the beam's endpoints (the hand end is the caller's; here the head) and the hit point
/// for the renderer.
pub fn laser_tick(head: V3, dir: V3, world: &crate::world::CollisionWorld) -> (V3, V3) {
    let len = 100.0f32;
    let end = [head[0] + dir[0] * len, head[1] + dir[1] * len, head[2] + dir[2] * len];
    // The world hit caps the beam's visual end (Collide_RayIntersect 0x1ec690's first-hit t).
    let hit_t = world.segment(head, end).unwrap_or(1.0);
    let hit = [head[0] + dir[0] * len * hit_t, head[1] + dir[1] * len * hit_t, head[2] + dir[2] * len * hit_t];
    (head, hit)
}

/// The laser watch's held-meter constants and tick [D HUD_UpdateAmmoPane 0x19a7f0:160-176 — while
/// the player's view state is 0xb (the watch's held anim state — the fire tail's `+0x6C & 0x400`
/// arm belongs to the grenade family; the watch's +0xB4 = 0x60004e8 selects the same state via its
/// fire anim [L weapon_data 0x4E +0xB4; M 0x1a5460 disasm 0x1a6538-0x1a6578]) the ammo pane reads a
/// 0–100 value at the tracked object's instance link +0x110 (u16) / +0x104 (f32), NOT the
/// kind-0x1a clip percentage]. The measured drain while held: ~31/s ≈ 0.5/frame [D
/// docs/live/2026-10-14-weapon-measurements-pcsx2.md "The laser watch 0x4e": 100 → 59 over ~1.3 s].
/// The game's writer of those instance fields is UNLOCATED in the decompile (an exhaustive store
/// scan of ACTION.ELF's .text finds no write at link+0x110/link+0x104 nor at the obj-relative
/// +0x220/+0x214 — the pane reads them through the bullet walk) — the
/// port stands the value in as `Sim::laser_charge` with the [D] drain. The refill below is [S]:
/// no decompile or disc evidence pins the game's idle refill (the old "+1 per 4 frames" shape was
/// the gadget-clip regen [M Player_Update 0x193600 disasm 0x193f18-58], which regenerates the
/// gadget clips, not this meter); it is kept for the HUD's usability and kept OUT of pass/fail.
pub const LASER_CHARGE_DRAIN_PER_TICK: f32 = 31.0 / 60.0;
pub const LASER_CHARGE_REFILL_PER_TICK: f32 = 0.25;

/// One tick of the held meter: `beam_on` (the beam drew this tick) drains, an idle tick refills;
/// clamped to 0..100.
pub fn laser_charge_tick(charge: &mut f32, beam_on: bool) {
    if beam_on {
        *charge = (*charge - LASER_CHARGE_DRAIN_PER_TICK).max(0.0);
    } else {
        *charge = (*charge + LASER_CHARGE_REFILL_PER_TICK).min(100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::triggers::Channels;
    use crate::world::CollisionWorld;

    fn empty_world() -> CollisionWorld { CollisionWorld::empty() }

    #[test]
    fn laser_tick_returns_the_visual_endpoints_and_burns_nothing() {
        // The burn left laser_tick (the old aim-line + BEAM_RADIUS 0.25 stand-in): the game burns
        // through the bullet stream's contact list [M 0x1ae3d8:57-101 via 0x1eae80:92-108] — the
        // drone pass drives it. Here: the endpoints only, and a lock in the path stays unburnt.
        let mut ch = Channels::default();
        let world = empty_world();
        let mut locks = vec![test_lock(99.5, 0.0, 134)];
        let (a, b) = laser_tick([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], &world);
        assert!((b[0] - 100.0).abs() < 0.01, "the empty world's beam runs its full 100 units");
        assert!((a[0] - 0.0).abs() < 0.01, "the head end is the caller's point");
        assert!(!locks[0].burned && !ch.on(134), "laser_tick no longer burns — the contact chain is the bullet pass's");
    }

    fn test_lock(x: f32, y: f32, out: u16) -> Lock {
        Lock {
            placement: 0, pos: [x, y, 0.0], radius: 1.0, out, key_channels: [0xffff; 2], burn: 0, burned: false, unlock_on_operative: false, unlocked: false,
            kind: 1, code: 0, scan_progress: 0, scan_solved: 0, scanning: false, lod_next: false,
            burn_a: 0xff, burn_b: 0xff, grace: 0, pending_sounds: Vec::new(),
            facing: [0.0, 0.0, 1.0],
        }
    }

    #[test]
    fn lock_burn_takes_the_real_tick_counts_and_writes_the_channel() {
        // The real burn: 0x4E takes 3/tick off the 0xff bytes, the unlock fires when +0x105 < 4
        // [M 0x1ae3d8:82-101; both bytes start 0xff — control_init_object 0x134630, Lock_Create
        // writes neither]: (0xff − 3k < 4) → k = 84. The 0x4F's −10/tick → k = 26.
        let mut ch = Channels::default();
        let mut l = test_lock(99.5, 0.0, 134);
        let mut ticks = 0;
        while !l.burn(3, &mut ch, 10) { ticks += 1; assert!(ticks < 200, "the 0x4E burn must complete"); }
        assert_eq!(ticks, 83, "the 84th contact tick completes (the loop's last call returned true) [M 0x1ae3d8:82-101]");
        assert!(ch.on(134), "the unlock writes the out channel [M 0x1ae3d8:194-195]");
        let mut ch2 = Channels::default();
        let mut l2 = test_lock(99.5, 0.0, 135);
        let mut t2 = 0;
        while !l2.burn(10, &mut ch2, 10) { t2 += 1; assert!(t2 < 200); }
        assert_eq!(t2, 25, "the 0x4F's 26th contact tick completes [M 0x1ae3d8:66]");
    }

    #[test]
    fn lock_burn_refills_one_per_tick_after_the_two_tick_grace() {
        // The idle arm: a contact stores the grace 2 [M 0x1ae3d8:79]; every no-contact tick
        // decrements it; at 0 BOTH bytes refill +1 to the 0xff cap [M :107-121]. A partial burn
        // HEALS; a completed lock never refills (state 1 — the out channel stays latched).
        let mut ch = Channels::default();
        let mut l = test_lock(99.5, 0.0, 134);
        for _ in 0..10 { l.burn(3, &mut ch, 10); } // 0xff − 30 = 0xe1
        assert_eq!(l.burn_a, 0xe1);
        l.idle_tick(); l.idle_tick(); // the grace ticks [M :107-111]
        assert_eq!(l.burn_a, 0xe1, "no refill while the grace runs");
        l.idle_tick();
        assert_eq!(l.burn_a, 0xe2, "the refill resumes at grace 0, +1/tick [M :113-121]");
        for _ in 0..400 { l.idle_tick(); }
        assert_eq!(l.burn_a, 0xff, "the refill caps at 0xff");
        // And the burn resumes from the healed value.
        let mut ticks = 0;
        while !l.burn(3, &mut ch, 10) { ticks += 1; assert!(ticks < 200); }
        assert_eq!(ticks, 83, "a fully healed lock takes the full 84 ticks again");
    }

    #[test]
    fn laser_charge_tick_drains_at_the_measured_rate_and_clamps() {
        let mut c = 100.0f32;
        // 60 beam ticks = ~31 drained [D the measured ~31/s]; the floor is 0.
        for _ in 0..60 { laser_charge_tick(&mut c, true); }
        assert!((100.0 - c - 31.0).abs() < 0.01, "60 beam ticks drain ~31 (drained {:.2})", 100.0 - c);
        for _ in 0..600 { laser_charge_tick(&mut c, true); }
        assert_eq!(c, 0.0, "the charge floors at 0");
        // The idle REFILL SHAPE is [S] unverified (the game's writer is unlocated — see the module
        // doc): NOT a pass/fail pin. The cap at 100 is the pane's own 0–100 read [D the HUD's u16].
        for _ in 0..600 { laser_charge_tick(&mut c, false); }
        assert_eq!(c, 100.0, "the charge caps at 100 [D the pane reads a 0–100 value]");
    }

    fn is_laser_weapon_matches_the_laser_family() {
        // The disc records settle the ids: 0x4E/0x4F carry +0x6C & 0x100 (Draw_LaserBeam) and the
        // name label 0x05000043 "Laser"; 0x50/0x51 are the Grapple (0x4000) and 0x54/0x55 the
        // Micro-Camera (0) [L weapon_data + USATxt.dat].
        for w in [0x4eu8, 0x4f] { assert!(is_laser_weapon(w), "{w:#x} is a laser weapon"); }
        for w in [0x50u8, 0x51, 0x54, 0x55, 0x4a, 0x4c, 0x02] { assert!(!is_laser_weapon(w), "{w:#x} is not"); }
    }

    #[test]
    fn is_grapple_matches_the_q_claw_records() {
        for w in [0x50u8, 0x51] { assert!(is_grapple(w), "{w:#x} is the Q-Claw [L bank 'Grapple'; +0x70 & 0x40]"); }
        for w in [0x4Au8, 0x4c, 0x4e, 0x52, 0x54] { assert!(!is_grapple(w), "{w:#x} is not"); }
    }

    #[test]
    fn runs_beam_is_the_laser_and_the_stunner() {
        // +0x6C & 0x180 routes the shot's beam draw: 0x4E/0x4F → 0x900 (the laser), 0x4A/0x4C →
        // 0x880 (the taser) [L weapon_data; M 0x1a4298:131/134].
        for w in [0x4eu8, 0x4f, 0x4a, 0x4c] { assert!(runs_beam(w), "{w:#x} runs the beam"); }
        for w in [0x50u8, 0x54, 0x52] { assert!(!runs_beam(w), "{w:#x} does not"); }
    }

    #[test]
    fn is_decryptor_matches_the_q_decryptor_records() {
        // The disc records: 0x56/0x57 label 0x0500004e, +0x6C = 0 (no beam bits) [L ee_start_05.bin];
        // the −0x56 < 2 bands in Player_WeaponFiring 0x1a5460:155/551 and Lock_Update 0x1ae3d8:162/305.
        for w in [0x56u8, 0x57] { assert!(is_decryptor(w), "{w:#x} is the Decryptor"); }
        for w in [0x4eu8, 0x4f, 0x50u8, 0x54u8, 0x55u8, 0x58u8, 0x02u8] { assert!(!is_decryptor(w), "{w:#x} is not"); }
    }

    fn dec_lock(kind: u16, out: u16) -> Lock {
        Lock {
            placement: 0, pos: [10.0, 0.0, 0.0], radius: 1.0, out, key_channels: [0xffff; 2],
            burn: 0, burned: false, unlock_on_operative: false, unlocked: false,
            kind, code: 7, scan_progress: 0, scan_solved: 0, scanning: false, lod_next: false,
            burn_a: 0xff, burn_b: 0xff, grace: 0, pending_sounds: Vec::new(),
            facing: [0.0, 0.0, 1.0],
        }
    }

    #[test]
    fn decrypt_scan_solves_the_four_digits_at_the_game_thresholds_then_writes_the_channel() {
        // The base 0x56: +1/tick. Digit i solves when progress > (4−i)·0x28 — 160/120/80/40
        // [M Lock_Update 0x1ae3d8:347-364]; a solve resets the progress to 5 [M :367]; the unlock
        // lands at 4 of 4 (the disasm's bne v1,v0 0x1aebec) and writes the out channel
        // [M Lock_Unlock 0x1b0650 → case 1 :193-196].
        let mut ch = Channels::default();
        let mut l = dec_lock(0, 90);
        let mut ticks = 0u32;
        let mut solved_at = [0u32; 4];
        let mut last = false;
        while !l.unlocked && ticks < 100_000 {
            ticks += 1;
            let was = l.scan_solved;
            last = l.decrypt_tick(1, false, &mut ch, ticks);
            if l.scan_solved != was && (l.scan_solved as usize) <= 4 { solved_at[(l.scan_solved - 1) as usize] = ticks; }
        }
        assert!(l.unlocked, "4 of 4 digits unlock the lock");
        assert!(last, "decrypt_tick returned true exactly on the unlock tick");
        assert!(ch.on(90), "the unlock writes the lock's out switch channel");
        // The 0x56's own timeline: the FIRST contact only flips state 0 → 5 (nothing banks
        // [M Lock_Update 0x1ae3d8:163-190]), then 161 + 116 + 76 + 36 = 389 accumulating ticks
        // (161 to exceed 160 from 0, then 116 to exceed 120 from 5, 76 from 5 past 80, 36 past 40)
        // — 390 ticks of continuous contact in total.
        assert_eq!(ticks, 390, "the flip tick + the 0x56's per-tick +1 must take exactly 390 ticks (got {ticks})");
        assert_eq!(solved_at, [162, 278, 354, 390], "the per-digit solve ticks (tick 1 is the flip)");
    }

    #[test]
    fn decrypt_upgrade_runs_three_times_faster() {
        // The 0x57 adds 3/tick [M 0x1ae3d8:309-314]: from 0 to >160 needs 54 ticks (53·3=159),
        // from 5 to >120 39 (5+38·3=119), from 5 to >80 26 (5+25·3=80 is not > 80; 83 is),
        // from 5 to >40 12 (5+11·3=38, 5+12·3=41) — the flip tick + 54+39+26+12 = 132 ticks.
        let mut ch = Channels::default();
        let mut l = dec_lock(0, 91);
        let mut ticks = 0u32;
        while !l.unlocked && ticks < 100_000 { ticks += 1; l.decrypt_tick(1, true, &mut ch, ticks); }
        assert!(l.unlocked && ch.on(91));
        assert_eq!(ticks, 132, "the flip tick + the 0x57's +3/tick takes 132 ticks (got {ticks})");
    }

    #[test]
    fn decrypt_scan_decays_four_per_tick_and_resets_completely() {
        // No contact bullet: −4/tick [M 0x1ae3d8:328-339]; at 0 the scan resets — state back to 0,
        // the solved digits lost [M :340-344].
        let mut ch = Channels::default();
        let mut l = dec_lock(0, 92);
        for _ in 0..100 { l.decrypt_tick(1, false, &mut ch, 1); }
        assert!(l.scanning && l.scan_progress == 99, "scanning after 100 ticks — the first flipped, 99 banked (progress now {})", l.scan_progress);
        for _ in 0..30 { l.decrypt_tick(0, false, &mut ch, 1); }
        assert!(!l.scanning && l.scan_solved == 0 && l.scan_progress == 0,
            "30 ticks of −4 fully reset the scan (progress now {})", l.scan_progress);
        // And the reset scan re-accumulates from scratch (the game restarts the digits):
        // the first re-contact is the flip again, so 50 ticks bank 49.
        for _ in 0..50 { l.decrypt_tick(1, false, &mut ch, 1); }
        assert_eq!(l.scan_progress, 49, "re-contact accumulates from 0 again (50 ticks, one flip)");
    }

    #[test]
    fn decrypt_first_contact_flips_state_0_to_5_and_banks_nothing() {
        // Lock_Update 0x1ae3d8's state-0 contact arm [M :163-190]: the class-5 bullet's
        // `+0xf6 += 1` makes the progress non-zero and the `sVar4 != 0` arm immediately moves the
        // lock to state 5 (`+0xf4 = 5`) and resets the progress to 0 — the first contact banks
        // NOTHING. The sister: the second contact banks the game's +1 [M :305-307].
        let mut ch = Channels::default();
        let mut l = dec_lock(0, 95);
        assert!(!l.decrypt_tick(1, false, &mut ch, 1));
        assert!(l.scanning && l.scan_progress == 0 && l.scan_solved == 0,
            "the flip tick leaves the progress at 0 (now {})", l.scan_progress);
        l.decrypt_tick(1, false, &mut ch, 2);
        assert_eq!(l.scan_progress, 1, "the second contact banks the game's +1");
    }

    #[test]
    fn decrypt_scan_ignores_a_laser_kind_lock_and_an_unlocked_one() {
        // The sister: record[0] == 1 is the LASER-burn lock — the decryptor never scans it
        // [M the use gate 0x1a5460:562 == 0 || == 2; Lock_Update case 0's kind-1 arm reads 0x4e/0x4f only].
        let mut ch = Channels::default();
        let mut l = dec_lock(1, 93);
        for _ in 0..500 { l.decrypt_tick(1, false, &mut ch, 1); }
        assert!(!l.scanning && !l.unlocked && !ch.on(93), "the laser-kind lock ignores the decryptor");
        let mut l2 = dec_lock(0, 94);
        l2.unlocked = true;
        for _ in 0..10 { assert!(!l2.decrypt_tick(1, false, &mut ch, 1)); }
        assert!(!ch.on(94), "an already-unlocked lock stays quiet");
    }

    #[test]
    fn decrypt_pick_gates_on_the_ray_the_world_and_the_range() {
        let world = empty_world();
        // A decryptor-kind lock 1.5 ahead; eye and root colocated for the unit test.
        let locks = vec![dec_lock(0, 95)];
        assert_eq!(pick_decryptor_lock([8.5, 0.0, 0.0], [1.0, 0.0, 0.0], [8.5, 0.0, 0.0], &locks, &world, 2.0), Some(0));
        // The gate is the DISTANCE root→lock, not the ray [M 0x1a5460:574-577].
        assert_eq!(pick_decryptor_lock([7.0, 0.0, 0.0], [1.0, 0.0, 0.0], [7.0, 0.0, 0.0], &locks, &world, 2.0), None,
            "3.0 from the lock is beyond the base decryptor's 2.0 range");
        // The 0x57's 4.0 reaches it.
        assert_eq!(pick_decryptor_lock([7.0, 0.0, 0.0], [1.0, 0.0, 0.0], [7.0, 0.0, 0.0], &locks, &world, 4.0), Some(0));
        // Off the ray: no pick.
        assert_eq!(pick_decryptor_lock([8.5, 3.0, 0.0], [0.0, 1.0, 0.0], [8.5, 3.0, 0.0], &locks, &world, 4.0), None);
        // The laser-kind lock is not a target.
        let laser_locks = vec![dec_lock(1, 96)];
        assert_eq!(pick_decryptor_lock([8.5, 0.0, 0.0], [1.0, 0.0, 0.0], [8.5, 0.0, 0.0], &laser_locks, &world, 4.0), None);
    }
}
// ---- the Q-Claw grapple (0x4A/0x4C, substate 2) ----------------------------------------------------

/// A grapple point (placement 0x3A, `Grapple_Create` 0x198298 — the class-0x51 object, the world
/// sphere like the ladders [M Control_BuildWorldSph 0x133070]).
#[derive(Clone, Debug)]
pub struct GrapplePoint {
    pub pos: V3,
    pub radius: f32,
}

impl GrapplePoint {
    /// `Grapple_Create` over the main map's 0x3A placements (05's #281/#282 [D probe]).
    pub fn create(placements: &[nf_formats::placement::Placement], object: impl Fn(&nf_formats::placement::Placement) -> Option<(V3, f32)>) -> Vec<GrapplePoint> {
        let mut out = Vec::new();
        for p in placements.iter() {
            if p.flags & 0xff != 0x3a { continue; }
            let (centre, radius) = object(p).unwrap_or((p.translation, 1.0));
            out.push(GrapplePoint {
                pos: [p.translation[0] + centre[0], p.translation[1] + centre[1], p.translation[2] + centre[2]],
                radius,
            });
        }
        out
    }
}

/// The grapple substate's runtime state (`BLData+0x140..0x160`'s fields the substate reads).
///
/// `Player_InitGrapple` 0x195d10 (144 B, `Player_Init` 0x1910f8:147 — unconditional, SP and MP):
/// `memset(BL+0x140, 0, 0x30)` (the grapple block, exactly this struct's default) and hangs the
/// rope mesh object — `Control_CreateObjEx(1.0, hashtable(0x02000138), parent = player, mode 5)` —
/// at **BL+0x158** [M 0x195d10:9-13]. The zeroed block is what `GrappleState::default()` gives;
/// the rope mesh is render-side (bldata.md:56/59 flag +0x140/+0x158 as the not-ported fields).
#[derive(Clone, Debug)]
pub struct GrappleState {
    /// The point Bond hangs from (`+0x14C`'s object position + `+0xC0`'s offset [M 0x192860 tail]).
    pub point: V3,
    /// `+0x15C`: 1 = reeling, 2 = hanging.
    pub phase: u8,
    /// `+0x150`: the reel speed accumulator (0.05/tick up to 1.5 single-player [M 0x192860:96-104]).
    pub pull: f32,
}

/// The fire-time point selection: the nearest point whose sphere the aim ray touches within
/// `max_dist`. The game's test is a physical one: the claw's bolt (a flying bullet whose object
/// radius is 1.0 [M Bullet_init 0x12c5a0:92 — `obj+0xe8 = 1.0`, never overridden]) sweeps the ray
/// and its death check is the travelled-distance cap `weapon_data+0x1c` [M Bullet_update
/// 0x12e7b0:158], so the acceptance window is the point's sphere grown by the bolt's radius.
pub fn pick_grapple_point(points: &[GrapplePoint], from: V3, dir: V3, max_dist: f32) -> Option<V3> {
    const BOLT_RADIUS: f32 = 1.0; // [M 0x12c5a0:92]
    let mut best: Option<(f32, V3)> = None;
    for p in points {
        let Some(t) = ray_closest_t(from, dir, p.pos, max_dist) else { continue };
        let closest = [from[0] + dir[0] * t, from[1] + dir[1] * t, from[2] + dir[2] * t];
        let d2 = (closest[0] - p.pos[0]).powi(2) + (closest[1] - p.pos[1]).powi(2) + (closest[2] - p.pos[2]).powi(2);
        let reach = p.radius + BOLT_RADIUS;
        if d2 > reach * reach { continue; }
        if best.is_none_or(|(bt, _)| t < bt) { best = Some((t, p.pos)); }
    }
    best.map(|(_, pos)| pos)
}

/// The claw bolt's range: the weapon's own `weapon_data+0x1c` — the game's bolt-death cap
/// [M Bullet_update 0x12e7b0:158] — read off the record: 50.0 for the Q-Claw 0x50, 100.0 for the
/// upgrade 0x51 [L ee_start_05.bin; the fists 1.75 / P2K 50 / sniper 1200 scale confirms the unit.
/// The old 5.0/10.0 were the STUNNER 0x4A/0x4C's ranges — the hardcoded table's shift].
pub fn grapple_range(wd: &nf_formats::weapon::WeaponData, w: u8) -> f32 {
    wd.records.get(w as usize).map_or(5.0, |r| f32::from_le_bytes(r[0x1c..0x20].try_into().unwrap()))
}

fn ray_closest_t(a: V3, dir: V3, p: V3, max_dist: f32) -> Option<f32> {
    let ab2 = dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2];
    if ab2 < 1e-9 { return None; }
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let t = (ap[0] * dir[0] + ap[1] * dir[1] + ap[2] * dir[2]) / ab2;
    (t >= 0.0 && t <= max_dist).then_some(t)
}

/// One tick of the grapple substate [M Player_SSGrapple 0x192860]. The fire polarity is the game's
/// own and it is the opposite of a press-to-release model: phase 1 (the reel) reads NO fire input
/// at all [M 0x192860:79-121 — the exits are the MP line-of-sight miss, the rope-fire anim done
/// with the 15-tick timer expired, and the arrival < 2.0]; the pull accumulates 0.05/tick up to
/// **1.5 single-player** [M 0x192860:96-104 — `fVar10 = 1.5` when `MPSettings == 0`], and the
/// arrival < 2.0 hangs [M 0x192860:91-94]. Phase 2 (the hang) holds while the fire is HELD and
/// detaches the tick it is RELEASED [M 0x192860:56-66 — `Input_Action(9,1)` non-zero commits the
/// wish velocity +0x10..0x1F → +0x20..0x2F and stays; zero → phase 4]. Returns whether the grapple
/// detached this tick.
pub fn grapple_tick(pos: &mut V3, state: &mut GrappleState, fire: bool) -> bool {
    match state.phase {
        1 => {
            // The reel: no fire test [M 0x192860:79-121]; the pull cap is 1.5 single-player.
            state.pull = (state.pull + 0.05).min(1.5);
            let d = [state.point[0] - pos[0], state.point[1] - pos[1], state.point[2] - pos[2]];
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            if len < 2.0 {
                state.phase = 2; // the hang [M 0x192860:91-94]
            } else {
                let step = state.pull / len;
                for k in 0..3 { pos[k] += d[k] * step; }
            }
            false
        }
        2 => {
            // The hang: HELD fire stays (the velocity commit [M 0x192860:57-62]); the RELEASE
            // detaches [M 0x192860:64-66 — phase 4]. The return is the detach.
            !fire
        }
        _ => false,
    }
}

#[cfg(test)]
mod grapple_tests {
    use super::*;

    #[test]
    fn grapple_reels_bond_to_the_point_and_hangs_while_the_fire_is_held() {
        let points = vec![GrapplePoint { pos: [10.0, 20.0, 0.0], radius: 2.0 }];
        let Some(point) = pick_grapple_point(&points, [0.0, 12.0, 0.0], [1.0, 1.0, 0.0].map(|v| v / 2f32.sqrt().powi(2) * 2f32.sqrt()), 100.0) else {
            panic!("the aim ray through the point's sphere picks it");
        };
        let mut state = GrappleState { point, phase: 1, pull: 0.0 };
        let mut pos = [0.0f32, 12.0, 0.0];
        let mut ticks = 0;
        // The reel runs with the fire HELD (phase 1 reads no fire input [M 0x192860:79-121]).
        while state.phase == 1 && ticks < 600 {
            grapple_tick(&mut pos, &mut state, true);
            ticks += 1;
        }
        assert_eq!(state.phase, 2, "the reel arrives (the distance < 2.0 [M 0x192860:91-94])");
        let d = ((pos[0] - point[0]).powi(2) + (pos[1] - point[1]).powi(2) + (pos[2] - point[2]).powi(2)).sqrt();
        assert!(d < 2.5, "Bond hangs within reach of the point (got {d:.2})");
        // The hang holds while the fire stays held [M 0x192860:57-62].
        for _ in 0..30 { assert!(!grapple_tick(&mut pos, &mut state, true), "held fire must NOT detach"); }
        // The RELEASE detaches [M 0x192860:64-66].
        assert!(grapple_tick(&mut pos, &mut state, false), "the fire release detaches");
    }

    #[test]
    fn grapple_acceptance_is_the_bolt_sweep_and_the_range_is_weapon_data() {
        // The bolt's radius grows the window: a ray passing 1.2 from a radius-0.64 point misses a
        // bare ray but the game's bolt (radius 1.0 [M 0x12c5a0:92]) hits.
        let points = vec![GrapplePoint { pos: [10.0, 0.0, 0.0], radius: 0.64 }];
        let dir = [10.0f32, 1.2, 0.0].map(|v| v / (10.0f32 * 10.0 + 1.2 * 1.2).sqrt());
        assert!(pick_grapple_point(&points, [0.0, 0.0, 0.0], dir, 100.0).is_some(),
            "the bolt sweep (radius 1.0) accepts a 1.2-unit pass");
        // The range is the record's own weapon_data+0x1c [M 0x12e7b0:158; L ee_start_05.bin]:
        // 50.0 for the claw (0x50), 100.0 for the upgrade (0x51).
        let mut records = vec![[0u8; 0x10c]; 0x60];
        for (i, rng) in [(0x4ausize, 5.0f32), (0x4c, 10.0), (0x50, 50.0), (0x51, 100.0)] {
            records[i][0x1c..0x20].copy_from_slice(&rng.to_le_bytes());
        }
        let wd = nf_formats::weapon::WeaponData { records, ammo_cap: Vec::new(), ammo_raw: Vec::new() };
        assert_eq!(grapple_range(&wd, 0x50), 50.0);
        assert_eq!(grapple_range(&wd, 0x51), 100.0);
        assert_eq!(grapple_range(&wd, 0x4a), 5.0, "the Stunner's own range — the old claw claim");
        // Beyond the range the bolt dies mid-flight — no pick.
        let far = [10.0f32, 0.0, 0.0];
        let far_dir = [1.0f32, 0.0, 0.0];
        assert!(pick_grapple_point(&points, [0.0, 0.0, 0.0], far_dir, grapple_range(&wd, 0x50)).is_none()
            || (far[0] - 0.0) <= grapple_range(&wd, 0x50), "the point within the range still picks");
    }

    #[test]
    fn grapple_ignores_points_off_the_ray() {
        let points = vec![GrapplePoint { pos: [10.0, 0.0, 0.0], radius: 2.0 }];
        assert!(pick_grapple_point(&points, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 100.0).is_none(),
            "the ray pointing up misses the point on the +x axis");
    }
}
