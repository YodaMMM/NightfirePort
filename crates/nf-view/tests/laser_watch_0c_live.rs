//! 991-laser-watch-0c: the laser watch on 0700000c behaves like the PS2 from the reported repro
//! spot. Evidence base:
//! - The repro: `--bond 7.61,15.71,-113.39,5.783,-0.363` — Bond stands on the inner balcony (the
//!   floor below the spot is 14.606 [D coll-floor]; `stand_at` rests him at floor + 1.1 = 15.706,
//!   the reported 15.71), yaw 5.783, pitch −0.363 (the `--bond` view: the camera pitch is
//!   `bond.pitch × π/2` [M nf-view lib.rs — the `--bond` parser stores the raw word]).
//! - The target [D placements-all dump, 0700000c.bin]: the ray's only contact is placement #139
//!   `hingelock1` (7.0916, 15.7046, −112.3706), a kind-0x2E Lock_Create object, record param 0 = 1
//!   (the laser-burn kind [M Lock_Update 0x1ae3d8 case 0's 0x4E/0x4F arm]), param 5 = 14 (the out
//!   channel [M Lock_Unlock 0x1b0650 writes record u16[+8] on burn-through]). The loaded world
//!   sphere (the glist bounds, `Control_BuildWorldSph` 0x133070's shape): centre = the placement
//!   point, radius 0.2325 [D]; the repro ray's closest approach is 0.041 from it (the other four
//!   0c locks are ≥ 1.0 off the ray) — the "bolted hinge of a grated gate" is the lower-left
//!   hinge of the `ventmesh-front` #138 skylight grate.
//! - What the burn does [M + D]: ch 14 alone does nothing — the AND mux #505 (type 0xEC,
//!   list 14,15,16,17 → out 18 [D triggers]) needs all four hinges #139–142 (out chs 14–17);
//!   ch 18 drives SS #138 `ventmesh-front` (6 keys, rate 50 — the grate slides down to y 14.76)
//!   and SS #179 `VentBlockingPoly` (2 keys, rate 10 — rises y 16.3 → 20.0) [D #138/#179 params;
//!   M SS_Create 0x1a9c58:51 + SS_Update 0x1a9948]. That chain is fix-level-0c's; THIS file pins
//!   it from the repro spot through the REAL fire path.
//! - The game's burn time [M Lock_Update 0x1ae3d8:57-101 + control_init_object 0x134630]: one
//!   contact per held-fire tick (the record's +0x40 refire is 1 [L weapon_data 0x2bf150 +
//!   0x4E·0x10C]; `Player_WeaponFiring` 0x1a5460:653-658 spawns one round per tick through
//!   `Player_WeaponInitBullet` 0x1a4bf0), each contact −3 off BOTH burn bytes from 0xff, complete
//!   when +0x105 < 4 → **84 contact ticks** (255 − 3·84 = 3 < 4). The bullet that carries the
//!   contact dies at the record's +0x1C flight range = 4.0 [L; M Bullet_update 0x12e7b0 case 1
//!   kills at weapon+0x1C < BU+0xF4] — the hinge sits 1.34 units along the repro ray, inside the
//!   first (state-0) segment, so contact lands every held tick. 831's 210.0 stand-in covered the
//!   same first segment here; the stand-in is gone (drone.rs, laser-watch-0c).
//!
//! Opt-in via `NF_ISO`.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load() -> nf_view::scene::Scene {
    let iso = iso().expect("NF_ISO");
    let mut scene = nf_view::load(&iso, "0700000c.bin", false).expect("load 0700000c");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    for _ in 0..100 { sim.step_bond(Input::default(), false, false, false); }
    scene
}

/// The repro view, held: Bond on the balcony floor at the reported spot looking at the grate's
/// lower-left hinge — no aim helper, the raw `--bond` yaw/pitch.
const REPRO_POS: [f32; 3] = [7.61, 15.71, -113.39];
const REPRO_YAW: f32 = 5.783;
const REPRO_PITCH: f32 = -0.363;

/// Aim Bond's view at `target` from his head (the fix-level-0c helper; the port's look
/// convention: pitch ±1 → ±π/2, yaw around +z).
fn aim_at(sim: &mut nf_view::scene::Sim, target: [f32; 3]) {
    let h = sim.bond.head();
    let d = [target[0] - h[0], target[1] - h[1], target[2] - h[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    sim.bond.yaw = d[0].atan2(d[2]);
    sim.bond.pitch = (d[1] / l).clamp(-1.0, 1.0).asin() * 2.0 / std::f32::consts::PI;
}

fn hinge(sim: &nf_view::scene::Sim, out: u16) -> [f32; 3] {
    sim.drones.locks.iter().find(|l| l.out == out).map(|l| l.pos)
        .unwrap_or_else(|| panic!("the out-{out} hinge lock exists [D #139-142]"))
}

/// The repro, end to end through the real input path: equip the Laser (0x4E) with the switch
/// input, hold the fire on the grate's lower-left hinge from the reported spot and view, and the
/// game's outcome happens in the game's time — the 84-contact burn completes (ch 14 rises at the
/// 84th contact tick [M 0x1ae3d8:82-101, :194-195]), the contact is the RAY's lock alone (the
/// sister hinges stay 0xff [M Collide_PickObj 0x1eae80's per-object pick]), ch 18 stays OFF (one
/// hinge is not the AND's answer [D #505]), and burning the remaining three opens the display
/// vent (ch 18 → SS #138 + #179, the opening passes a ray [M SS_Update 0x1a9948]).
#[test]
fn the_repro_spot_burns_the_grate_hinge_in_the_games_time() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load();
    let sim = scene.sim.as_mut().unwrap();
    assert_eq!(sim.drones.locks.iter().filter(|l| l.out == 14).count(), 1, "hinge #139 (out 14) exists [D]");
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, REPRO_POS, REPRO_YAW);
    sim.bond.pitch = REPRO_PITCH;
    // The floor under the repro spot is the balcony (14.606 [D coll-floor]); stand_at rests Bond
    // at floor + 1.1 — the reported 15.71 root.
    assert!((sim.bond.pos[1] - 15.71).abs() < 0.05, "Bond rests on the balcony root 15.71, got {}", sim.bond.pos[1]);
    // Equip the watch laser through the REAL switch input (the draw runs its anim; the fire gate
    // holds shots until it settles [M Player_WeaponFiring 0x1a5460's switching gate]).
    let draw = Input { switch_to: Some(0x4e), ..Default::default() };
    for _ in 0..40 { sim.step_bond(draw, false, false, false); }
    assert_eq!(sim.gun.weapon, 0x4e, "the Laser must equip; weapon {:#x}", sim.gun.weapon);
    let h139 = hinge(sim, 14);
    eprintln!("trace: hinge #139 at {h139:?} (the head-ray closest approach 0.041, r 0.2325 [D probe])");

    // HOLD the fire with the repro view frozen (no turn/look): the beam parks on the first shot
    // tick and every held tick spawns one round [L +0x40 refire 1; M 0x1a5460:653-658].
    let mut first_beam: Option<u32> = None;
    let mut done_tick: Option<u32> = None;
    let mut fail_checked = false;
    for tick in 1..=300u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), true, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if first_beam.is_none() && sim.laser_beam.is_some() { first_beam = Some(tick); }
        let Some(fb) = first_beam else { continue };
        let held = tick - fb; // held ticks since the first beam/shot
        if held == 40 && !fail_checked {
            fail_checked = true;
            let l = sim.drones.locks.iter().find(|l| l.out == 14).unwrap();
            // The fail-first pin [M 0x1ae3d8:82-101]: 40 contacts (−3 each) must NOT complete
            // (255 − 120 = 135 ≥ 4), and the byte must have descended by the contact cadence —
            // one contact per held tick (±1 for the beam-park/shot ordering inside the tick).
            assert!(!l.burned, "40 contact ticks (< 84) must NOT complete hinge #139 [M 0x1ae3d8:82-101]");
            assert!((132..=138).contains(&l.burn_b),
                "40 held ticks burn −3 each from 0xff (one contact per held tick [M 0x1a5460:653-658]), got {:#x}", l.burn_b);
            // The pick is per-object: the sister hinges (#140-142) take nothing from THIS ray
            // [M Collide_PickObj 0x1eae80 lands the contact on the picked object only].
            for other in [15u16, 16, 17] {
                let o = sim.drones.locks.iter().find(|l| l.out == other).unwrap();
                assert_eq!((o.burn_a, o.burn_b), (0xff, 0xff),
                    "hinge out {other} is off the repro ray — no contact, no burn [M 0x1eae80:92-108], got {:#x}/{:#x}", o.burn_a, o.burn_b);
            }
        }
        if done_tick.is_none() && sim.drones.channels.on(14) { done_tick = Some(tick); }
        if done_tick.is_some() && held > 95 { break; }
    }
    let sim = scene.sim.as_ref().unwrap();
    let fb = first_beam.expect("the held fire parks the laser beam [M Player_LaserPointer 0x1a65d8]");
    let done = done_tick.expect("84 contact ticks complete the hinge [M 0x1ae3d8:82-101]");
    eprintln!("trace: first beam tick {fb}, ch 14 rises at tick {done} (+{} after the first beam)", done - fb);
    // The game's time: 84 contacts from the first shot tick, the completion ON the 84th contact
    // [M :82-101 — the unlock runs in the same Lock_Update pass]. The observable window is the
    // 84-contact burn plus ≤ 2 ticks of beam-vs-shot ordering; anything outside means the pass
    // dropped or doubled contacts.
    let delta = done - fb;
    assert!((82..=86).contains(&delta),
        "ch 14 must rise 84 contact ticks after the first beam [M 0x1ae3d8:82-101, 255 − 3·84 = 3 < 4], got +{delta}");
    let l = sim.drones.locks.iter().find(|l| l.out == 14).unwrap();
    assert!(l.burned, "the lock record is burned");
    assert!(!sim.drones.channels.on(18), "one hinge alone must NOT raise ch 18 — the AND #505 needs 14,15,16,17 [D #505]");
    let ss138 = sim.drones.sss.of_placement(138).expect("the ventmesh-front SS exists [D #138]");
    assert_eq!(sim.drones.sss.list[ss138].t, 0.0, "the grate has not played");

    // The remaining three hinges, through the same real path (the aim re-converges per hinge —
    // the player's own action in the game): the clip 500 [L +0x92] covers 4 × 85 spends.
    for out in [15u16, 16, 17] {
        let target = {
            let sim = scene.sim.as_ref().unwrap();
            hinge(sim, out)
        };
        let mut burned_at = None;
        for tick in 1..=170u32 {
            let s = scene.sim.as_mut().unwrap();
            aim_at(s, target);
            s.step_bond(Input::default(), true, false, false);
            if scene.sim.as_ref().unwrap().drones.channels.on(out as u32) { burned_at = Some(tick); break; }
        }
        assert!(burned_at.is_some(), "hinge out {out} must burn through the real path (84 contacts + the aim convergence)");
        eprintln!("trace: hinge out {out} burned at fire-tick {:?}", burned_at);
    }
    // The AND completes → ch 18 → the grate SSs play to their last keys and the opening passes
    // the collision ray (fix-level-0c's opening pins, now from the repro spot's chain).
    let mut d138 = None;
    let mut d179 = None;
    {
        let sim = scene.sim.as_mut().unwrap();
        let ss138 = sim.drones.sss.of_placement(138).expect("ventmesh-front SS [D #138]");
        let ss179 = sim.drones.sss.of_placement(179).expect("VentBlockingPoly SS [D #179]");
        for tick in 1..=300u32 {
            sim.step_bond(Input::default(), false, false, false);
            let (t138, t179) = (sim.drones.sss.list[ss138].t, sim.drones.sss.list[ss179].t);
            if d138.is_none() && t138 >= 5.0 { d138 = Some(tick); }
            if d179.is_none() && t179 >= 1.0 { d179 = Some(tick); }
            if d138.is_some() && d179.is_some() { break; }
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(18), "the AND mux #505 raised ch 18 [D #505 list 14,15,16,17]");
    assert!(d138.is_some(), "ventmesh-front (#138, rate 50) swept [M 0x1a9c58:51 + 0x1a9948:107], got {d138:?}");
    assert!(d179.is_some(), "VentBlockingPoly (#179, rate 10) swept, got {d179:?}");
    let ray_from = [5.78f32, 16.3, -111.7];
    let ray_to = [5.78f32, 16.3, -113.0];
    assert!(sim.world.segment_hit(ray_from, ray_to).is_none(), "the open vent lets the ray through [M the SS end poses]");
}

/// The reach sister: the burn contact rides the round's per-tick segment, and the watch round's
/// flight dies at the record's +0x1C = 4.0 [L; M Bullet_update 0x12e7b0 case 1] — so from the
/// repro spot ONLY the locks on the ray within the segment reach can descend. The warehouse
/// `VentLatch` #182 sits 48 units away [D probe] on no part of this ray: 120 held ticks at the
/// frozen repro view leave it 0xff/0xff, while hinge #139 completes. (The old 210.0 stand-in
/// tiled the ray to 210 units; the pick is per-object either way — this pins the far latch
/// against any whole-level burn regression.)
#[test]
fn the_far_latch_takes_no_burn_from_the_repro_spot() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load();
    let sim = scene.sim.as_mut().unwrap();
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, REPRO_POS, REPRO_YAW);
    sim.bond.pitch = REPRO_PITCH;
    let draw = Input { switch_to: Some(0x4e), ..Default::default() };
    for _ in 0..40 { sim.step_bond(draw, false, false, false); }
    let mut done_tick = None;
    for tick in 1..=300u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), true, false, false);
        if done_tick.is_none() && scene.sim.as_ref().unwrap().drones.channels.on(14) { done_tick = Some(tick); break; }
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(done_tick.is_some(), "hinge #139 completes through the real path [M 0x1ae3d8:82-101]");
    let latch = sim.drones.locks.iter().find(|l| l.out == 84).expect("VentLatch #182 [D]");
    assert!((latch.burn_a, latch.burn_b) == (0xff, 0xff),
        "the far VentLatch must take no burn from the repro spot [M Bullet_update 0x12e95c: the +0x1C = 4.0 death; the latch is 48 units out], got {:#x}/{:#x}", latch.burn_a, latch.burn_b);
}
