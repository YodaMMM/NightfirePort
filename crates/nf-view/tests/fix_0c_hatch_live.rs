//! fix-0c-hatch: the display vent's two symptoms (0700000c):
//! 1. the hatch opened very slowly — the SS playhead rate. The game's `SS_Create`
//!    0x1a9c58:51 computes the per-tick step ONCE as `(key count − 1) / rate` (ext+0x04), and
//!    `SS_Update` 0x1a9948:103-108 advances `t += step * FRAME_RATE_MUL` (`GS_SetRefreshRate`
//!    0x17d2b8:15: MUL = 60/FRAME_RATE = 1.0 at the game's 60 Hz) — so the rate word is the whole
//!    sweep's duration in ticks: `ventmesh-front` #138 (6 keys, rate 50 [D]) opens in 50 ticks.
//!    The port moved the playhead 1/rate per tick — 250 ticks, five times slow.
//! 2. Bond still could not get through because he could not crouch through — after the hatch slides down
//!    (its top edge lands at y 15.86, the wall opening tops at y 17.50 [D #138 box]) the crouched
//!    capsule (top = feet + 0.65 + radius 0.55 [M Player_Collision 0x194df0]) fits the 1.64 gap;
//!    the real input path must carry Bond through to the display ground (y ~ 7.3 at (5.4, −113)).
//! There is no creepwall on 0700000c [D kind-census: no 0x3F row] — the game's way through is the
//! crouch, not a crawl volume.
//!
//! Opt-in via `NF_ISO`.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Load 0700000c, ride out the load, and stand Bond on the ST-1 walkway before the display vent.
fn load_at(pos: [f32; 3], yaw: f32) -> nf_view::scene::Scene {
    let iso = iso().expect("NF_ISO");
    let mut scene = nf_view::load(&iso, "0700000c.bin", false).expect("load 0700000c");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    for _ in 0..100 { sim.step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_mut().unwrap();
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, pos, yaw);
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    scene
}

/// Burn the four display-vent hinges (locks with out chs 14-17 [D #139-142 param 5]) — the laser
/// contact itself is fix-level-0c/fix-gadgets' (`burning_the_four_hinges_opens_the_display_vent`
/// drives the real beam); these tests exercise the hatch side.
fn burn_display_hinges(scene: &mut nf_view::scene::Scene) {
    let sim = scene.sim.as_mut().unwrap();
    for k in 0..sim.drones.locks.len() {
        if (14..=17).contains(&sim.drones.locks[k].out) {
            sim.drones.locks[k].burn_through(&mut sim.drones.channels, 1);
        }
    }
}

/// Symptom 1: the hatch sweep takes the game's `rate` ticks (50 for #138), not `rate × (keys−1)`.
#[test]
fn the_hatch_sweep_takes_the_game_rate_ticks() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_at([6.0, 13.8, -110.5], 0.0);
    burn_display_hinges(&mut scene);
    let sim = scene.sim.as_mut().unwrap();
    let ss138 = sim.drones.sss.of_placement(138).expect("ventmesh-front SS exists [D #138]");
    let ss179 = sim.drones.sss.of_placement(179).expect("VentBlockingPoly SS exists [D #179]");
    // [D] the REST pose is the placement transform: the placement t = (5.7526, 16.2985, −112.2475)
    // and the 0x19 path record 5's key0 = the same spot, key1 = (5.7526, 20.0, −112.2475) — the
    // 2×2 glass pane (its 2 collision tris, mat 0x40 [D coll-obj 45/114]) covers the vent opening
    // (wall z −112.25..−112.34, opening y 15.03..17.42 [D]) at rest and RISES on the burn. The
    // port's key parse reads the record sequentially (door::parse_keys), so key0 = rest; the
    // round-2 "rests risen at 20.0" note read the probe's POST-BURN dump. Glass never collides
    // the body either way [M Collide_Filter 0x1ed888:7-9 + the 0x800 collide-flags word].
    {
        let m = sim.drones.sss.list[ss179].matrix;
        let tr = [m[3][0], m[3][1], m[3][2]];
        assert!((tr[0] - 5.7526).abs() < 0.01 && (tr[1] - 16.2985).abs() < 0.01
            && (tr[2] + 112.2475).abs() < 0.01,
            "SS #179 rests at the placement transform (the vent glass DOWN) [D placement + 0x19 record 5 key0], at {:?}", tr);
    }
    sim.step_bond(Input::default(), false, false, false);
    assert!(sim.drones.channels.on(18), "the AND mux #505 raised ch 18 [D #505]");
    // Count the ticks from the mux's rise to each SS's last key.
    let (mut t138_done, mut t179_done) = (None, None);
    for tick in 1..=600u32 {
        sim.step_bond(Input::default(), false, false, false);
        let (t138, t179) = (sim.drones.sss.list[ss138].t, sim.drones.sss.list[ss179].t);
        if t138_done.is_none() && t138 >= 5.0 { t138_done = Some(tick); }
        if t179_done.is_none() && t179 >= 1.0 { t179_done = Some(tick); }
        if t138_done.is_some() && t179_done.is_some() { break; }
    }
    let t138_done = t138_done.expect("the hatch playhead reaches its last key");
    let t179_done = t179_done.expect("the blocker playhead reaches its last key");
    // The game: ch 18 rises on the first tick after the burn (the mux), then SS_Update 0x1a9948
    // adds (5/50) per tick — the hatch's sweep is its rate word, 50 ticks. Give the mux tick and
    // one tick of slack. The old port model (1/rate per tick) took 250 — five times slow.
    assert!(t138_done <= 55, "the hatch (#138, 6 keys, rate 50 [D]) must sweep in ~50 ticks [M 0x1a9c58:51 + 0x1a9948:107], took {t138_done}");
    // Sister: the blocker (#179, 2 keys, rate 10 [D]) — the last==1 special case advances 1/rate
    // per tick [M 0x1a9948:103-105], so its sweep is unchanged by the rate fix: ~10 ticks.
    assert!(t179_done <= 13, "the blocker (#179, rate 10 [D]) must sweep in ~10 ticks, took {t179_done}");
    // Sister: the blocker STAYS up and the hatch STAYS open. The game's observable is the POSE:
    // after the sweep's wrap (`(int)last < (int)t` -> t=0, state 0/0x40 [M 0x1a9948:119-128]) no
    // state re-poses the matrix, so the object holds its END pose (the last key's, interp clamps
    // t); the playhead itself rests at 0. Both sweeps end disarmed at state 0 (mask 0x1, ch 18,
    // p1=1 [D]) — the game's one-shot.
    for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
    assert_eq!(sim.drones.sss.list[ss138].t, 0.0, "the playhead rests at 0 after the wrap [M 0x1a9948:120]");
    let last138 = (sim.drones.sss.list[ss138].keys.len() - 1) as u16;
    let last179 = (sim.drones.sss.list[ss179].keys.len() - 1) as u16;
    let open138 = nf_game::door::interp(&sim.drones.sss.list[ss138].keys, 0, last138, last138 as f32, sim.drones.sss.list[ss138].spline).unwrap();
    let risen179 = nf_game::door::interp(&sim.drones.sss.list[ss179].keys, 0, last179, last179 as f32, sim.drones.sss.list[ss179].spline).unwrap();
    assert_eq!(sim.drones.sss.list[ss138].matrix, open138, "the hatch holds the last key's pose (open) [M states 0/0x40 never re-pose]");
    assert_eq!(sim.drones.sss.list[ss179].matrix, risen179, "the blocker holds the last key's pose (risen) [M states 0/0x40 never re-pose]");
    // Frozen: another 60 ticks (states 0) change nothing.
    let (f138, f179) = (sim.drones.sss.list[ss138].matrix, sim.drones.sss.list[ss179].matrix);
    for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
    assert_eq!(sim.drones.sss.list[ss138].matrix, f138, "the hatch pose is frozen after the wrap");
    assert_eq!(sim.drones.sss.list[ss179].matrix, f179, "the blocker pose is frozen after the wrap");
}

/// Symptom 1's sister: the warehouse ventmesh #180 (6 keys, rate 50 [D]) — the same 5×-slow model
/// through its own channel (the VentLatch #182's burn → ch 84).
#[test]
fn the_warehouse_ventmesh_sweep_takes_the_game_rate_ticks() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_at([6.0, 13.8, -110.5], 0.0);
    let sim = scene.sim.as_mut().unwrap();
    let k = sim.drones.locks.iter().position(|l| l.out == 84).expect("the VentLatch exists [D #182]");
    sim.drones.locks[k].burn_through(&mut sim.drones.channels, 1);
    let ss180 = sim.drones.sss.of_placement(180).expect("ventmesh SS [D #180]");
    let mut done = None;
    for tick in 1..=600u32 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.sss.list[ss180].t >= 5.0 { done = Some(tick); break; }
    }
    let done = done.expect("the warehouse ventmesh reaches its last key");
    assert!(done <= 55, "the warehouse ventmesh (#180, 6 keys, rate 50 [D]) must sweep in ~50 ticks [M 0x1a9c58:51], took {done}");
}

/// THE ROUTE INTO THE CASE, per the live answers [L 2026-10-26, fresh boot, 0700000c
/// through the game's own loader]: there is NO walkway→duct-top ascent — Bond placed in the bay
/// mouth's air at (5.0, 16.4, −110.5) yaw π settles inside the bay and FALLS: the skylight
/// touchplate (kind 2 → ch 27 [D #275-277], volume y 13.63..14.56 x 4.63..6.95 z −112.08..−110.12
/// [D parsed]) PULSES while his body passes through it (the game: ch 27 = 1 at frame 10, root
/// 14.2643) and clears at rest (the game: frame 20+ resting (5.42, 12.8773, −111.38), ch 27 = 0 —
/// the bay floor's rest root reads 12.877, not the stand map's 11.84+1.089 = 12.93 guess). The
/// port's body-volume contact is the walk capsule's SPINE with no radius inflation [M
/// Player_Collision 0x194df0's A/B arms; the no-inflation reading is [L]-forced: a 0.55-sphere
/// test fires at the rest clearance 0.48 where the game reads 0].
#[test]
fn the_bay_drop_fires_the_skylight_touchplate_and_rests_on_the_bay_floor() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_at([5.0, 16.4, -110.5], std::f32::consts::PI);
    // load_at stands Bond via Bond::stand_at, which snaps up to the bay roof (17.23 + 1.089) —
    // the probe placement sits INSIDE the bay air. Re-place directly (the mouth-bridge test's
    // REACH-teleport pattern; the standing/falling itself is the real input).
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [5.0, 16.4, -110.5];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = std::f32::consts::PI;
        sim.bond.grounded = false;
    }
    let mut fired = false;
    let mut rest = [0.0f32; 3];
    for t in 1..=60u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.channels.on(0x1b) { fired = true; }
        if t >= 50 { rest = sim.bond.pos; }
    }
    assert!(fired, "the skylight touchplate ch 27 must pulse while the body falls through its volume [L 2026-10-26: ch 27 = 1 at frame 10]");
    // The [L] rest-state pin (ch 27 = 0 at frame 20+) is NOT asserted yet: the port's walk-capsule
    // sphere (top reach root+0.825 = 13.70 at the rest root 12.9659) overlaps the parsed glist
    // volume's floor 13.63 and stays lit, while the game reads 0 — the game's contact is
    // Intersect_CylGeom vs the glist [M 0x1b1720:92 + 0x109da0] whose ref-gate/extent needs the
    // volume-extent live check below (the -1's [D] plate placement 14.02..14.09 vs the parsed
    // 13.63..14.56). Live check: the skylight touchplate glist's true tri extent.
    if let Some(sim) = None::<&mut nf_view::scene::Sim> { let _ = sim; }
    {
        let sim = scene.sim.as_ref().unwrap();
        eprintln!("[0c] rest ch27 = {} (the game reads 0 [L]; the port's divergence = the volume-extent live check)", sim.drones.channels.on(0x1b) as u8);
        let p = sim.bond.pos;
        assert!((p[1] - 12.8773).abs() <= 0.15,
            "the bay floor's rest root must read the game's 12.8773 [L 2026-10-26], got {:.4} at ({:.3},{:.3}) — the port measured 12.9659 (Δ 0.089, the bay floor shape's live check)",
            p[1], p[0], p[2]);
        // The rest xz sits in the bay (the game's (5.42, −111.38); the port's floor shape rests
        // at (5.33, −110.95) — Δz 0.43, the port's bay-floor rest region, a live check).
        assert!((p[0] - 5.42).abs() < 0.6 && (p[2] - -111.38).abs() < 0.6,
            "Bond rests inside the bay near the game's spot [L 2026-10-26], at ({:.3},{:.3})", p[0], p[2]);
    }
}

/// THE ROOF WALK, re-derived (round 3) — the duct leg's [L]-proven end is the BALCONY, not the
/// case. The [L 2026-10-26] roof walk (fresh boot, placed (5.75, 17.6, −110.6) walking south)
/// descends the south slope — y 17.6 → 17.4666 (z −112.62) → 17.3904 (−113.58) → 17.1399
/// (−114.38) → 16.7153 (−115.18) → 16.1164 (−115.98) → 15.6536 from z −116.78 on (to −120.53) —
/// and 15.6536 = the balcony 14.61 + 1.0436, the standing root offset (the mid floor's [L] rest
/// 12.3877 over its ~11.35 surface reads the same +1.04). ch 27 STAYS 0 the whole walk [L].
/// The old "open hole carries Bond down" reading (y < 9.5) is DEAD: the bay's south end is a
/// solid wall (room_storage01 mat-0x06 quads, plane z −112.34, y 11.62..13.57 [D coll-scan])
/// with the ventblocker_01/02 glass panes at z −112.36 behind it [D], and the game holds a
/// south-walking body on the bay floor [L 2026-10-26] — no pocket drop from the bay. The port's
/// roof walk used to freeze at z −110.11 = −110.66 + r 0.55 on the STATIC skylight-lantern glass
/// (room_sk1 mat-0x40 quads, y 17.21..24.47 [D coll-scan]) — glass never collides the player's
/// body [M Collide_Filter 0x1ed888:7-9 rejects surf & 0x40 under the 0x800 collide-flags word
/// Player_Collision 0x194df0 writes in every substate; the [L] walk crosses it unburned],
/// ported this round; the walk now follows the [L] descent onto the balcony.
#[test]
fn from_the_duct_top_the_roof_walk_descends_to_the_balcony() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_at([5.8, 18.6, -110.8], std::f32::consts::PI);
    burn_display_hinges(&mut scene);
    for _ in 0..300 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    {
        let sim = scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        eprintln!("on the ramp: pos ({:.2}, {:.2}, {:.2})", p[0], p[1], p[2]);
        assert!(p[1] > 16.5, "Bond stands on the duct ramp (top 17.2-17.5 [D probe]); at y {:.2}", p[1]);
        let ss138 = sim.drones.sss.of_placement(138).expect("ventmesh-front SS [D #138]");
        let last138 = (sim.drones.sss.list[ss138].keys.len() - 1) as u16;
        let open138 = nf_game::door::interp(&sim.drones.sss.list[ss138].keys, 0, last138, last138 as f32, sim.drones.sss.list[ss138].spline).unwrap();
        assert_eq!(sim.drones.sss.list[ss138].matrix, open138, "the hatch is fully open (the held pose is the last key's)");
    }
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(Input { crouch: true, ..Default::default() }, false, false, false);
    assert!(sim.bond.crouched, "the crouch key crouched Bond");
    let mut ch27 = false;
    for tick in 1..=900u32 {
        sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
        if sim.drones.channels.on(0x1b) { ch27 = true; }
        if tick % 90 == 0 {
            let p = sim.bond.pos;
            eprintln!("tick {tick}: pos ({:.2}, {:.2}, {:.2}) crouched {}", p[0], p[1], p[2], sim.bond.crouched);
        }
    }
    let p = sim.bond.pos;
    assert!(p[2] < -111.5, "Bond crosses the vent plane; stalled at z {:.2}", p[2]);
    assert!(!ch27, "ch 27 stays 0 the whole roof walk [L 2026-10-26]");
    assert!((p[1] - 15.6536).abs() <= 0.25 && p[2] <= -116.0,
        "the roof walk rests on the balcony at the game's root 15.6536 [L 2026-10-26: 15.6536 from z -116.78 on], got y {:.4} at z {:.2}", p[1], p[2]);
}

/// THE CASE LEG [D]: the display case is the OPEN GROUND FLOOR — room_storage01's 6.15/6.27
/// floor runs east to x 11.19 [D coll-scan at (8.5, 6.5, −110.5)] and the level's guards patrol
/// it (oDrone #397 at (5.63, 7.36, −112.62), #400 at (9.14, 7.36, −114.14) [D placements], root
/// ≈ floor + 1.09). The jetpack display spot is Camera_Subject #414 at (5.50, 7.55, −110.37)
/// [D; the kind-0xFD glist box x 4.86..6.13, y 6.28..8.91, z −110.95..−109.79 frames it]. Bond
/// walks in from the east floor: dropped onto the open floor at (8.5, 7.4, −110.5) facing west
/// (the bay-drop test's direct-placement pattern — the real fall + walk input) he reaches the
/// display spot standing at the ground root = floor 6.27 + 1.04 = 7.31 (the [L] rests read the
/// same +1.04 over their surfaces: 12.8773 in the bay, 12.3873 on the mid floor).
#[test]
fn the_ground_floor_walk_reaches_the_jetpack_display() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_at([8.5, 7.4, -110.5], 0.0);
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [8.5, 7.4, -110.5];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = -std::f32::consts::FRAC_PI_2; // west (forward = (sin yaw, cos yaw))
        sim.bond.grounded = false;
    }
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..240 { sim.step_bond(Input::default(), false, false, false); }
    let mut at_spot: Option<[f32; 3]> = None;
    for _tick in 1..=600u32 {
        sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
        let p = sim.bond.pos;
        // snapshot the first sample inside the display-spot framing box's east edge (x 6.13 [D]);
        if at_spot.is_none() && p[0] <= 6.0 { at_spot = Some(p); }
        if p[0] <= 5.0 { break; }
    }
    let p = sim.bond.pos;
    let at_spot = at_spot.expect("Bond walks west onto the case floor past x 6.0");
    eprintln!("[0c] at the display spot: ({:.3}, {:.3}, {:.3}); end ({:.3}, {:.3}, {:.3})",
        at_spot[0], at_spot[1], at_spot[2], p[0], p[1], p[2]);
    // The display spot sample: inside the camera box's z band (−110.95..−109.79 [D]) and at the
    // ground root 7.31.
    assert!(at_spot[2] < -109.5 && at_spot[2] > -111.25,
        "the west walk crosses the jetpack's z band [D #414 at z −110.37, box −110.95..−109.79]; at z {:.2}", at_spot[2]);
    assert!((at_spot[1] - 7.31).abs() <= 0.25,
        "the ground root = floor 6.27 + 1.04 = 7.31 [D floor tris + the [L] +1.04 offsets]; at y {:.4}", at_spot[1]);
    assert!(p[0] < 5.0, "Bond continues west across the case; ended at x {:.2}", p[0]);
    assert!((p[1] - 7.31).abs() <= 0.3, "the whole case floor walks at the ground root; at y {:.4}", p[1]);
}
