//! walkreal-0c shared prologue: the main walk's legs 1-9c, MOVED VERBATIM from
//! walkreal_0700000c_live's `walk_the_whole_mission_from_the_start_to_the_0d_handoff`
//! (same load, same ticks, same order) so a second test can START on the gate band by
//! REPLAYING the game's own route instead of teleporting to it (the eastroute test's
//! former start write). The two documented stand-ins inside — the trench exit and the
//! mezzanine reach — carry their reason + sisters inline exactly as in the main test.

#[path = "walker_b2.rs"]
pub mod walker;

pub use walker::{Act, Walker, Wp, wrap_pi};

/// The spawn asserts + legs 1-9c of the main walk, ending on the slot jump's band landing
/// (grounded, floor 10.2..11.7, x > 42.6). Deterministic: the caller continues from here.
pub fn walk_spawn_to_the_band(iso: &str) -> Walker {
    let iso = iso.to_string();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap());
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - 8.98).abs() < 2.0 && (p[1] - 15.07).abs() < 2.0 && (p[2] + 149.65).abs() < 2.0,
            "the walkway start #1 [D], got {p:?}");
        assert!(sim.drones.inventory.owned.get(0x54).copied().unwrap_or(false),
            "the camera gadget (0x54) is in the loadout [M 0x1a7ef8 case 0x700000c]");
        // The boot loadout's other members, for the record ([D loadout; no assert on the order]).
        eprintln!("[0c] the boot loadout: {:?}", (0..0x60).filter(|i| sim.drones.inventory.owned.get(*i).copied().unwrap_or(false)).collect::<Vec<_>>());
    }
    w.wait_enabled(3000, "the intro");
    w.wait(120);

    // Leg 1 (REAL, blocker 2 replaced): the walkway NORTH to the inner balcony, the LASER-WATCH
    // burn of the four hinge locks, and the vent drop through the trench — the game's own route
    // to objective 1 (the vent/pulley stand-in is gone). Evidence: the walkway floor 14.61
    // [D probe blk]; the balcony repro spot (7.61, 15.71, −113.39) [L 991]; the hinges #139-142
    // (out 14..17, (7.09/4.54, 15.70/16.87, −112.37) [D]); one contact per held tick, −3 off
    // both burn bytes from 0xff, complete when +0x105 < 4 → 84 contact ticks [M Lock_Update
    // 0x1ae3d8:82-101 + Player_WeaponFiring 0x1a5460:653-658]; the AND #505 (14,15,16,17 → 18
    // [D]) drives SS #138 ventmesh-front 16.4 → 14.76 and SS #179 VentBlockingPoly 16.3 → 20.0
    // [D #138/#179; M SS_Update 0x1a9948]; after both sweep, the y-16.3 ray crosses the vent
    // [M the SS end poses]. The trench (x 5..6.5, z −110.5..−111.5, floor 11.84 [D probe blk
    // floors]) carries plate #274 (kind 2, out 0x1b, centre (5.79, 14.09, −111.10) [D trig]) in
    // the fall path — the game fires it while the body's volume crosses [L hatch-2].
    w.walk_to(Wp::new(9.0, 15.7, -140.0).r(1.5), "the walkway north 1");
    // The east lane: the walkway carries a raised block x 5..10, z −122..−125 (top 16.49, west
    // face x 10.15 [D probe0c-lane floors+rays]) — the x 11..13 lane is the clear walk north.
    w.walk_to(Wp::new(12.0, 15.7, -118.0).r(1.5), "the walkway north 2 (east of the raised block)");
    w.walk_to(Wp::new(7.61, 15.71, -113.39).r(0.8), "the inner balcony repro spot [L 991]");
    // The laser out through the REAL switch input (the draw runs its anim; the fire gate holds
    // shots until it settles [M 0x1a5460's switching gate]).
    w.tick(nf_game::player::Input { switch_to: Some(0x4e), ..Default::default() }, false);
    for _ in 0..60 {
        if w.scene.sim.as_ref().unwrap().gun.weapon == 0x4e { break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x4e, "the Laser equips [M 0x1a7ef8]");
    // Burn the hinges: the aim re-converges per hinge through turn/look (the player's own
    // re-aim) and the trigger HOLDS — 84 contacts each [M 0x1ae3d8:82-101]; the clip 500 [L
    // +0x92] covers 4 × 85 spends.
    for out in [14u32, 15, 16, 17] {
        let target = w.scene.sim.as_ref().unwrap().drones.locks.iter()
            .find(|l| l.out as u32 == out).map(|l| l.pos)
            .unwrap_or_else(|| panic!("the out-{out} hinge lock exists [D #139-142]"));
        let mut burned = false;
        let start = w.total_ticks;
        while w.total_ticks - start <= 500 {
            let fire_input = {
                let sim = w.scene.sim.as_ref().unwrap();
                if sim.drones.channels.on(out) { burned = true; break; }
                let h = sim.bond.head();
                let d = [target[0] - h[0], target[1] - h[1], target[2] - h[2]];
                let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                let yaw_err = walker::wrap_pi(d[0].atan2(d[2]) - sim.bond.yaw);
                // the port's look convention: pitch ±1 → ±π/2 [M nf-view lib.rs --bond parser]
                let pitch_err = (d[1] / l).clamp(-1.0, 1.0).asin() * 2.0 / std::f32::consts::PI - sim.bond.pitch;
                nf_game::player::Input {
                    turn: yaw_err.clamp(-0.08, 0.08),
                    look: pitch_err.clamp(-0.08, 0.08),
                    ..Default::default()
                }
            };
            w.tick_fire(fire_input);
        }
        assert!(burned, "hinge out {out} never burned under the held laser fire [M 0x1ae3d8:82-101]");
        eprintln!("[0c] hinge out {out} burned at +{} ticks", w.total_ticks - start);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(18), "the AND mux #505 raised ch 18 [D #505 list 14,15,16,17]");
    }
    // The SSs animate after ch 18 (rate 50 / rate 10 [D]) and each RESETS to t 0 / state 0
    // when its sweep completes (this run: t138 0.1 → 3.1 → 0 by +60) — so the completion pin
    // is the END POSE: the y-16.3 ray crossing the open vent [M the SS end poses; the same
    // ray the laser-watch test pins].
    let mut saw_sweep = false;
    let mut open = false;
    for _ in 0..300 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let ss138 = sim.drones.sss.of_placement(138).expect("ventmesh-front SS [D #138]");
            let ss179 = sim.drones.sss.of_placement(179).expect("VentBlockingPoly SS [D #179]");
            let (t138, t179) = (sim.drones.sss.list[ss138].t, sim.drones.sss.list[ss179].t);
            if t138 > 0.0 || t179 > 0.0 { saw_sweep = true; }
            if sim.world.segment_hit([5.78, 16.3, -111.7], [5.78, 16.3, -113.0]).is_none() { open = true; break; }
        }
        w.tick(nf_game::player::Input::default(), false);
    }
    eprintln!("[0c] the vent sweep ran ({saw_sweep}) and the end pose passed the ray ({open})");
    assert!(open, "the open vent never let the y-16.3 ray through [M the SS end poses]");
    eprintln!("[0c] all four hinges burned — the vent is open");
    // Through the open vent (the passage the y-16.3 ray proves) and off the lip into the
    // trench — the REAL fall crosses plate #274 mid-fall (ch 0x1b) and lands on the lower
    // walkway (11.84 [D probe blk floors]).
    w.walk_to(Wp::new(5.78, 15.7, -112.3).r(0.6), "through the open vent");
    let mut fired_at = None;
    let start = w.total_ticks;
    let landed = loop {
        let sim = w.scene.sim.as_ref().unwrap();
        let (p, grounded) = (sim.bond.pos, sim.bond.grounded);
        if grounded && fired_at.is_some() { break true; }
        if w.total_ticks - start > 400 { break false; }
        if !grounded && fired_at.is_none() && sim.drones.channels.on(0x1b) {
            fired_at = Some(w.total_ticks - start);
        }
        let desired = (5.78f32 - p[0]).atan2(-110.8f32 - p[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = nf_game::player::Input {
            turn: err.clamp(-0.08, 0.08),
            forward: if err.abs() > 0.6 { 0.0 } else { 1.0 },
            ..Default::default()
        };
        w.tick(input, false);
    };
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(landed && fired_at.is_some() && sim.bond.grounded && (p[1] - 12.93).abs() < 0.6,
            "the vent drop must fire plate #274 mid-fall (ch 0x1b [D #274; L hatch-2]) and land on the \
             lower walkway (11.84 [D probe blk floors]), got pos {p:?} fired {fired_at:?}");
        assert_eq!(sim.drones.mission.states[11][0], 4, "objective 1 completes on the #274 drop [M 0x1827d8:88-99]");
        assert_eq!(sim.drones.mission.states[11][1], 3, "ch 0x1b reveals objective 2 [L record 11 obj 1]");
    }
    eprintln!("[0c] the vent drop fired plate #274 mid-fall at +{:?} — objective 1 complete on the game's own route", fired_at);

    // Leg 2 (REAL): out of the case's trench south-west onto the 11.27 hall, the hall east
    // under the case, the upper east lane (11.27, x 13..14 [D probe]) south to the STAIRCASE
    // HEAD, the ramp descent to the display ground (6.27), the ledge north to the SUBJECT's
    // clear-line eye, and the photograph through the REAL camera switch and bullet path [M
    // CamSubject_Update 0x1bc668:14-24]. (Route floors from the crossings session's probe: the
    // direct south lane hits the 1.28
    // riser wall at z −114.09 [D probe]; the x 13..14 upper lane carries.)
    // The trench EXIT — the stand-in stays, and this session PINNED why it must: the "bay" is
    // the display case's vent SHAFT (floor 11.84 = the case's glass roof over the jetpack
    // display, case ground 6.27 below [D this session's tri dump]), a one-way drop in the
    // disc's own data:
    //   - the shaft is closed E/W (x 4.78/6.74, walls to 14.6 and up to 15.86), N (z −110.39)
    //     and by the roof 17.05..17.26 over z −111.97..−110.39 [D]; the only opening is the
    //     vent hole 15.03..17.42 at the south wall top — 3.2 over the floor, unjumpable (the
    //     0.78 apex [M player.rs]) and ladderless (the level's ladders are the yard #164/#495
    //     only [D]);
    //   - the south end z −111.89..−112.34 is a CRAWL POCKET (floor 11.84, ceiling 13.34 [D the
    //     down-facing quad]) under the sill stack (14.81/15.03), closed south by the wall slab
    //     z −112.34..−112.363 whose outer face spans y 11.62..13.57, x 4.56..6.96 [D];
    //   - the crouch capsule (A = pos + 0.1 − (cc − 0.55), B = pos + 0.45 − cc, r 0.55 [M
    //     Player_Collision 0x194df0 case 3; cc 0.556 [L 05-truck]]) FITS the pocket (top =
    //     floor + 1.251 = 13.09 < 13.34) and walks in — and stops ON the slab: contact face
    //     −112.363 + r 0.55 = −111.813, the stall pinned deterministically below (the prior
    //     session's three x starts all stalled at the same z −111.81 [probe runs]);
    //   - the body-height rays CANNOT see the slab: at capsule height the slab's north face
    //     does not exist (its face pieces sit only below the floor 11.62..11.84 and above the
    //     pocket ceiling 13.34..13.57 [D]) and the outer face points away from a north ray
    //     (the rays are front-face only [M 0x10a8c8:168]) — "every ray at y ≥ 11.9 clear" was
    //     honest and the block is still real: the CAPSULE touches the slab double-sided;
    //   - the game seals it the same way: [L 2026-10-26 hatch-3, fresh boot] the game HOLDS a
    //     south-walking body on the bay floor; and no glass exit exists ([D census, this
    //     session: 0x20 Break placements in 0700000c = 0; the glass-shatter path runs only
    //     through Break_Kill on 0x20 kind 2 [M 0x12b970; psiCreateBreakableData 0x117418's
    //     only caller is parsemap 0x20 kind 2, 0x1d04f0:70-73]]);
    //   - the OLD comment's escape story is REFUTED: "the real game's way out is the vent
    //     duct's own descent into the case interior (~7.3) per the hatch sessions' map"
    //     misread those sessions — the ~7.3 was hatch-1's UNMAPPED-leg question, not a
    //     finding; the hatch sessions' [L]-backed map says the case interior (root 7.31 =
    //     6.27 + 1.04) is entered from the east floor only, and the drop rests on the bay
    //     floor 11.84 ([L 2026-10-26 hatch-2] fresh-boot rest (5.42, 12.8773, −111.38)) — the
    //     port's drop MATCHES the game (the landing assert above).
    //   - What would settle the real post-drop flow is live check 1 (the report): whether the
    //     real game ever moves the body out of the shaft (a script, a checkpoint nudge, an
    //     exit the static data hides), and whether its capsule stops at −111.813 like the
    //     port's.
    // The stand-in moves Bond onto the 11.27 hall just south of the case (the pinned floors
    // [D probe]) — across the disc's own south seal, documented here; the sisters pin the
    // pocket's geometry so the stand-in never silently becomes a walk.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // The pocket-entrance wall (z −111.89, y 13.34..14.6): the hit the old "lintel" sister
        // found is THIS wall's face, not a glass lintel above 13.36.
        let wall = sim.world.segment_hit([5.5, 13.5, -111.4], [5.5, 13.5, -113.0]);
        assert!(wall.is_some(), "the pocket-entrance wall (z −111.89, y 13.34..14.6 [D block dump]) vanished; the exit stand-in's landmark moved");
        // The pocket ceiling 13.34 over the crawl (a ray up from inside the pocket hits its
        // underside).
        let ceil = sim.world.segment_hit([5.5, 12.0, -112.1], [5.5, 14.0, -112.1]);
        assert!(ceil.is_some(), "the pocket ceiling (13.34 over z −111.89..−112.34 [D block dump]) vanished; the crawl story moved");
        // The ray-blindness pin: at capsule height the south slab is ray-invisible (the
        // mechanism above). If a face ever appears here, the exit story must be re-probed.
        assert!(sim.world.segment_hit([5.5, 12.5, -111.7], [5.5, 12.5, -113.0]).is_none(),
            "the south slab grew a ray-visible north face at capsule height; re-probe the exit");
    }
    // The REAL crouch walk into the pocket and onto the disc's own stall: crouch once (the
    // game's toggle [M Player_SSCrouch 0x1924c0:84-105]) and hold south — the body advances to
    // the slab contact and stops at −111.813 (−112.363 + 0.55) with the bay floor still under
    // it. This is the deterministic pin the old "stalls at z −111.81 … UNPINNED" note asked
    // for.
    {
        let mut held_z = None;
        for t in 0..420 {
            {
                let sim = w.scene.sim.as_ref().unwrap();
                let desired = std::f32::consts::PI; // south (−z), the vent-drop loop's yaw sense
                let err = walker::wrap_pi(desired - sim.bond.yaw);
                let input = nf_game::player::Input {
                    crouch: t < 2, // the toggle press; the 60-tick lockout eats the repeat [M 0x1924c0]
                    turn: err.clamp(-0.08, 0.08),
                    forward: if err.abs() > 0.6 { 0.0 } else { 1.0 },
                    ..Default::default()
                };
                w.tick(input, false);
            }
            let sim = w.scene.sim.as_ref().unwrap();
            if t == 329 { held_z = Some(sim.bond.pos[2]); }
        }
        let sim = w.scene.sim.as_ref().unwrap();
        let z = sim.bond.pos[2];
        assert!(sim.bond.crouched, "the crouch toggle did not hold through the pocket walk");
        assert!(z < -111.7, "the crouch walk never reached the pocket mouth (z {z}); the shaft's south changed");
        assert!((z - -111.813).abs() <= 0.02,
            "the crouch walk did not stop on the south slab contact (−112.363 + 0.55 = −111.813 [M 0x194df0 case 3; D the slab]): stopped at z {z}");
        let held = held_z.expect("held_z sampled");
        assert!((held - z).abs() <= 0.02, "the stall is not deterministic: z {held:.4} at t 330 vs {z:.4} at t 420 while still pressing south");
        // The bay floor is still under the stalled body (the prior session's "grounded false"
        // read the walker's push dither, not a hole): the take ray down from the capsule's B
        // end hits the 11.84 floor.
        let p = sim.bond.pos;
        let floor = sim.world.segment_hit([p[0], p[1] - 0.106, p[2]], [p[0], 11.0, p[2]]);
        assert!(floor.is_some(), "the bay floor vanished under the stalled body at {p:?}");
        // Stand back up (the toggle's stand arm [M Player_SSCrouch 0x1924c0:84-105]) so the
        // walk continues from the stand-in standing, as the pre-pin route did.
        for _ in 0..3 {
            w.tick(nf_game::player::Input { crouch: true, ..Default::default() }, false);
        }
        for _ in 0..40 {
            w.tick(nf_game::player::Input::default(), false);
        }
        assert!(!w.scene.sim.as_ref().unwrap().bond.crouched, "the stand-up toggle failed after the pocket stall");
    }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [5.0, 12.35, -112.9];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    eprintln!("[0c] the trench-exit stand-in (the disc's own south seal — the shaft is a one-way drop) — on the 11.27 hall");
    w.walk_to(Wp::new(8.0, 12.4, -113.2).r(1.0), "the hall east (south of the x 6..7 pillars)");
    w.walk_to(Wp::new(13.5, 12.4, -115.0).r(1.2), "the upper east lane");
    // The STAIRCASE descent — the game's own route to the display ground, no class port
    // needed: the 0xE2 SS class is fully ported (SS_Create 0x1a9c58 / SS_Update 0x1a9948 /
    // SS_Operate 0x1a9df8 / SS_Activate 0x1a9ea8 = crates/nf-game/src/ss.rs create/tick/
    // activate), and the level's 0xE2 census [D] has NO travel lift: the obj_stor*_pully*
    // winches #351/#364/#377 carry four SAME-position keys (a ±0.026-rad wheel sway, mask 0x2
    // = the bullet contact class [M 0x1a9df8:35-37]) — dead props, not rides. The walkable
    // descent is the hall floor's own opening (the floor is MISSING at x 9..12.4 for
    // z −115.5..−131.5 [D probe rays]) crossed by the static `obj_stor2_staircase01` #359
    // (kind 0x19, bounds x 9.97..12.29, y 6.13..12.14, z −127.5..−117.1 [D placement]): the
    // landing at 11.12..11.29 (z −127..−128, level with the 11.27 hall floor) and the ramp
    // down to 6.29 at z −120 [D probe rays]. The lane walk stays EAST of x 12.5 (the shaft
    // is open to the 2.18 deep floor); the descent lands on the south ledge (6.27,
    // x 12.6..14.5 [D probe rays]) whose north end joins the display ground at z −113.5
    // (the 6.27 floor spans x 1..14.5 there [D probe rays]).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // Sisters pinning the route's real geometry: the shaft hole east of the lane, the
        // landing Bond is about to walk onto, and the ramp's mid surface.
        let shaft = sim.world.segment_hit([11.0, 12.0, -122.0], [11.0, 10.6, -122.0]);
        assert!(shaft.is_none(), "the stair shaft gained hall-height floor at (11, −122); the lane walk could cross it [D probe rays]");
        let landing = sim.world.segment_hit([11.2, 13.0, -128.0], [11.2, 10.6, -128.0]);
        assert!(landing.is_some(), "the stair landing (11.29 at z −128 [D probe rays]) vanished; the descent has no head");
        let ramp = sim.world.segment_hit([11.2, 12.0, -124.0], [11.2, 7.0, -124.0]);
        assert!(ramp.is_some(), "the ramp's mid surface (~8.52 at z −124 [D probe rays]) vanished; the descent would free-fall");
    }
    w.walk_to(Wp::new(13.2, 12.4, -124.0).r(1.0), "the lane south to the stair head");
    w.walk_to(Wp::new(13.0, 12.4, -128.2).r(0.8), "the stair approach");
    // West onto the landing (z −128: the hall floor 11.27 at x 12.5+, the landing 11.29..11.32
    // at x 10.5..12 — level [D probe rays]).
    w.walk_to(Wp::new(11.2, 12.35, -128.1).r(0.6), "west onto the stair landing");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && (sim.bond.pos[1] - 12.35).abs() < 0.4,
            "Bond stands on the stair landing (11.29 [D probe rays]), got {:?} grounded {}", sim.bond.pos, sim.bond.grounded);
    }
    // Down the ramp (south, ~0.8 y per z [D probe rays]): targets every ~1.5 z keep the
    // steering on the surface; the body rides the slope with short airborne hops.
    for (x, z, label) in [
        (11.2f32, -126.5f32, "the ramp 1"),
        (11.2, -125.0, "the ramp 2"),
        (11.2, -123.5, "the ramp 3"),
        (11.3, -122.0, "the ramp 4"),
        (11.4, -120.8, "the ramp 5"),
    ] {
        w.walk_to(Wp::new(x, 7.0, z).r(0.9), label);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.pos[1] < 9.0 && sim.bond.pos[2] > -122.0,
            "the ramp descent reaches the bottom landing (6.29 at z −120 [D probe rays]), got {:?}", sim.bond.pos);
    }
    // East onto the south ledge (the bottom 6.29..6.32 joins the ledge 6.27 at x 12.6 [D
    // probe rays]), then NORTH along it to the display pocket — the ledge's west edge is the
    // shaft (x 11.5..12.3 open to 2.18 at z −114..−117 [D probe rays]), so the west turn
    // happens at z −113.5 where the 6.27 floor spans the full width.
    w.walk_to(Wp::new(12.2, 7.4, -120.3).r(0.7), "the stair bottom east");
    w.walk_to(Wp::new(13.2, 7.4, -120.0).r(0.8), "onto the ledge");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && (sim.bond.pos[1] - 7.41).abs() < 0.5,
            "Bond stands on the south ledge (6.27 [D probe rays]), got {:?} grounded {}", sim.bond.pos, sim.bond.grounded);
        let hole = sim.world.segment_hit([11.8, 8.0, -115.5], [11.8, 1.4, -115.5]);
        let hole_y = hole.map(|(t, _)| 8.0 + t * (1.4 - 8.0));
        assert!(hole_y.map_or(false, |y| y < 3.0), "the ledge's west shaft (x 11.8, z −115.5 open to 2.18 [D probe rays]) gained floor: {hole_y:?}");
    }
    w.walk_to(Wp::new(13.2, 7.4, -116.0).r(0.9), "the ledge north 1");
    w.walk_to(Wp::new(13.2, 7.4, -113.4).r(0.8), "the ledge north 2");
    w.walk_to(Wp::new(10.5, 7.4, -113.4).r(0.9), "the display ground west");
    w.walk_to(Wp::new(5.4, 7.35, -113.0).r(0.7), "the display eye");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && (sim.bond.pos[1] - 7.41).abs() < 0.5,
            "Bond stands on the display ground at the eye spot (6.27 floor [D probe rays]), got {:?} grounded {}",
            sim.bond.pos, sim.bond.grounded);
    }
    eprintln!("[0c] the staircase descent walked for real — the eye on the display ground at {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);
    let subject = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.props.subjects.iter().find(|s| s.channel == 0x1c)
            .map(|s| (s.lo, s.hi)).expect("the jetpack subjects [D #414/#454]")
    };
    eprintln!("[0c] the display eye (the staircase-routed spot) — the camera next");
    // The camera out through the REAL switch input (the old gun.switch_request write is gone).
    w.tick(nf_game::player::Input { switch_to: Some(0x54), ..Default::default() }, false);
    for _ in 0..60 {
        if w.scene.sim.as_ref().unwrap().gun.weapon == 0x54 { break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x54, "the camera is held [M 0x1a7ef8]");
    let sc = [(subject.0[0] + subject.1[0]) / 2.0, (subject.0[1] + subject.1[1]) / 2.0, (subject.0[2] + subject.1[2]) / 2.0];
    let photographed = shoot_at(&mut w, sc, 30.0, 900, "the jetpack photograph",
        &|sim| sim.drones.channels.on(0x1c));
    assert!(photographed, "the photograph never wrote ch 0x1c [M 0x1bc668:14-24]");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for _ in 0..2 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
        assert_eq!(sim.drones.mission.states[11][1], 4, "objective 2 completes [M 0x1827d8:88-99]");
    }
    eprintln!("[0c] objective 2 complete — the photograph (the staircase-routed eye spot)");

    // Leg 4: the secret-door reach — the DOCUMENTED stand-in (master's shape), because this
    // session's probes closed every walking route off the display-ground complex toward the
    // yard/warehouse: the slab's south face stands at z −155.77..−156.26 from y ~2.5 up
    // (x 2.5..15.25 [probe rays]), leaving only a 0.23..0.4 slot above the 2.37 deck — no
    // standing or crouched capsule passes [D probe rays]; the raw south walk stops at the
    // edge z −155.04 standing AND crouched [probe runs]; the 2.18 deep-floor lane (x 8.5..12.2,
    // the shaft's bottom) runs south only to z ~−150 and steps up onto the same 2.37 deck
    // pocket, wall-closed south at x ≥ 10.5 [D probe rays]; and the east deck pocket (2.37,
    // x 12.5..15.4, z −154..−155.5) is closed south by the same face [D probe rays]; east of
    // ST-1 the void span x 15..45 has faces at x 14.9/31.7/45.2 at y 6.3 and nothing above
    // body height to walk [D probe rays]. THIS session also mapped the far side: the plates
    // #558/#559 stand in a ground-level corridor (floor 0.90, x 52.5..55.5) that runs NORTH
    // under the warehouse mezzanine (5.44..6.00 [D this session's under-mezzanine scan]) with
    // the vent plug #184 hanging in it at z −165..−164, and the corridor's only proven
    // entrances are the secret door (west, z −174.3..−173.1) and that unproven north end. The
    // mezzanine's annex vent (#180/#182/#184, ch 0x54) is the warehouse-side link, for a
    // future leg. So the plates reach keeps the master's reach stand-in onto the plates; the
    // corridor walk after it is REAL (leg 5), and the door's west side gives the REAL yard
    // descent (leg 5b) — the master's yard-descent stand-in is GONE.
    // Leg 4 (the REAL route — one documented stand-in): the plates' corridor is entered the
    // game's own way, DOWN through the mezzanine's annex vent. The vent SS #180 `ventmesh` +
    // the blocking poly #184 hang in the corridor's ceiling at (53.8, 5.5, −163.59) with the
    // Q-Laser latch #182 `VentLatch` (out ch 0x54 [D #182 p5]) at (53.80, 5.90, −165.36), all
    // above the mezzanine deck (5.44..6.03, x 52..56, z −176..−152 [D reach-1 scans]). The
    // [L 2026-10-31] fresh-boot runs closed every other entrance: the corridor's north end
    // stops at z −150.23 on a barrel row in every lane (the port pins the same stop —
    // walkreal_probe0c_barrelrow_live), the ST-1 walkway's south end holds at z −153.65 (a
    // railing, no walk-off and no jump clears it in the port either —
    // walkreal_probe0c_lip_live), and the mezzanine deck itself floods as a separate component
    // in the port's statics (reach-1 comp 3). So the ONE stand-in left is the mezzanine REACH
    // (the filed live check: how the game climbs to it — the grapple points gp0/gp1 at y 12.3
    // [D #308/#346/#500/#508/#511] hang over the warehouse); everything after it is real input.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [54.0, 5.8, -158.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && (5.2..6.2).contains(&sim.bond.pos[1]),
            "the mezzanine-deck stand-in (5.44..6.03 [D reach-1 scans]) must hold Bond, got {:?} grounded {}",
            sim.bond.pos, sim.bond.grounded);
        assert!(sim.drones.locks.iter().any(|l| l.out as u32 == 0x54),
            "the ch-0x54 VentLatch #182 exists [D #182 p5]");
        assert!(!sim.drones.channels.on(0x54), "ch 0x54 is off before the burn [D #182]");
    }
    w.walk_to(Wp::new(54.0, 5.8, -160.8).r(0.8), "the deck south");
    // The deck's internal 0.56 step-down around z −161 holds the walk (the game's own lip-hold
    // [L 2026-10-31 walkway]); cross it with a grounded jump, real input.
    {
        let start = w.total_ticks;
        let mut jumped = false;
        for _ in 0..400u32 {
            let (p, grounded, yaw) = {
                let sim = w.scene.sim.as_ref().unwrap();
                (sim.bond.pos, sim.bond.grounded, sim.bond.yaw)
            };
            if p[2] < -164.5 && grounded { break; }
            let desired = (53.8f32 - p[0]).atan2(-165.5f32 - p[2]);
            let err = wrap_pi(desired - yaw);
            let jump = !jumped && grounded && p[2] < -160.4;
            if jump { jumped = true; }
            w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, jump, ..Default::default() }, false);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && sim.bond.pos[2] < -163.0,
            "the deck's step-down crossing must reach the latch side (z < −163), got {:?} grounded {}",
            sim.bond.pos, sim.bond.grounded);
        eprintln!("[0c] the deck step-down crossed at +{} ticks onto ({:.2},{:.2},{:.2})",
            w.total_ticks - start, sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
    }
    w.walk_to(Wp::new(53.8, 5.8, -164.8).r(0.6), "under the latch");
    // The Q-Laser burn of the VentLatch #182 through the held-fire path (the same shape as the
    // hinge burn above [M Lock_Update 0x1ae3d8:82-101]) — ch 0x54.
    w.tick(nf_game::player::Input { switch_to: Some(0x4e), ..Default::default() }, false);
    for _ in 0..60 {
        if w.scene.sim.as_ref().unwrap().gun.weapon == 0x4e { break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x4e, "the laser is held [M 0x1a7ef8 case 0x700000c]");
    {
        let latch = w.scene.sim.as_ref().unwrap().drones.locks.iter()
            .find(|l| l.out as u32 == 0x54).map(|l| l.pos)
            .expect("the ch-0x54 VentLatch #182 lock [D]");
        let mut burned = false;
        let start = w.total_ticks;
        while w.total_ticks - start <= 900 {
            {
                let sim = w.scene.sim.as_ref().unwrap();
                if sim.drones.channels.on(0x54) { burned = true; break; }
            }
            let fire_input = {
                let sim = w.scene.sim.as_ref().unwrap();
                let h = sim.bond.head();
                let d = [latch[0] - h[0], latch[1] - h[1], latch[2] - h[2]];
                let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                let yaw_err = wrap_pi(d[0].atan2(d[2]) - sim.bond.yaw);
                let pitch_err = (d[1] / l).clamp(-1.0, 1.0).asin() * 2.0 / std::f32::consts::PI - sim.bond.pitch;
                nf_game::player::Input { turn: yaw_err.clamp(-0.08, 0.08), look: pitch_err.clamp(-0.08, 0.08), ..Default::default() }
            };
            w.tick_fire(fire_input);
        }
        assert!(burned, "the VentLatch #182 never burned under the held laser fire [M 0x1ae3d8:82-101]");
        eprintln!("[0c] the VentLatch #182 burned — ch 0x54 at +{} ticks", w.total_ticks - start);
    }
    // The vent SSs sweep (ch 0x54 [D #180/#184]) and the end pose clears the hole.
    {
        let mut swept = false;
        for _ in 0..300u32 {
            w.tick(nf_game::player::Input::default(), false);
            let sim = w.scene.sim.as_ref().unwrap();
            if let Some(ss) = sim.drones.sss.list.iter().find(|s| s.channel == 0x54) {
                if ss.state == 0 { swept = true; break; }
            }
        }
        assert!(swept, "the vent SS never swept open on ch 0x54 [D #180/#184; M SS_Update 0x1a9948]");
    }
    // Onto the vent hole and the drop into the corridor (the floor 0.90 [D secretreach]; the
    // 4.5 m fall lands inside the corridor SOUTH of the barrel row at z −150.23 [L]).
    w.walk_to(Wp::new(53.8, 5.8, -163.6).r(0.5), "onto the vent hole");
    let mut landed = false;
    for _ in 0..300u32 {
        w.tick(nf_game::player::Input { forward: 0.3, ..Default::default() }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.grounded && sim.bond.pos[1] < 3.0 {
            landed = true;
            break;
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(landed, "the vent drop never landed in the corridor, pos {:?}", sim.bond.pos);
        assert!(sim.bond.pos[2] < -162.0 && sim.bond.pos[2] > -165.5 && (51.5..56.0).contains(&sim.bond.pos[0]),
            "the vent drop must land inside the corridor past the barrel row (x 52..56, z −162..−165.5), got {:?}",
            sim.bond.pos);
        eprintln!("[0c] the vent drop landed in the corridor at ({:.2},{:.2},{:.2}) — past the barrel row [L z −150.23]",
            sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
    }
    // The corridor south to the plates (the REAL walk the stand-in used to skip).
    w.walk_to(Wp::new(53.5, 1.0, -167.0).r(1.0), "the corridor south 1");
    w.walk_to(Wp::new(53.2, 1.0, -171.0).r(1.0), "the corridor south 2");
    w.walk_to(Wp::new(52.9, 1.0, -173.3).r(0.7), "the plates");
    w.wait(20);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // Sister: the corridor floor under the plates (0.90 [D this session's scan]) holds.
        let floor = sim.world.segment_hit([52.76, 3.4, -173.59], [52.76, -1.0, -173.59]).map(|(t, _)| 3.4 - 4.4 * t);
        assert!(floor.map_or(false, |y| (0.6..1.2).contains(&y)),
            "the secret-door corridor floor under the plates moved (expected 0.90 [D this session]), got {floor:?}");
        // Sister: the corridor's SOUTH end is the 2.07 step (z −176.0, x 52.0..55.6 [D this
        // session's scan]; its north face slopes 2.07 → ~1.71 over z −176.0..−175.9) —
        // pinning why the descent goes WEST through the door instead.
        let step = sim.world.segment_hit([53.8, 4.0, -176.0], [53.8, -1.0, -176.0]).map(|(t, _)| 4.0 - 5.0 * t);
        assert!(step.map_or(false, |y| (1.8..2.3).contains(&y)),
            "the corridor's south step moved (expected 2.07 [D this session]), got {step:?}");
    }
    eprintln!("[0c] the corridor walked for real through the open vent — on the plates at {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);

    // Leg 5 (REAL): the secret-door chain — the co-located plates [D #558/#559]: #559 (mask 8)
    // is the player BODY touch; #558 (mask 4) is the E-interactable arm (`Trigger_Activate`
    // 0x1b2318:23-33's type-2 `mask & 4` gate). The AND #560 [D: out 8, inputs 2, 3] raises
    // ch 8 and `Door_secret1` #544 (ch 8, 2 keys, rate 400, wrap 1 [D]) sweeps open over two
    // E pulses and holds.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(8), "the secret door's ch 8 is off before the chain runs [D #560]");
        assert!(sim.drones.channels.on(2), "the body-touch plate fires ch 2 under Bond [D #559]");
        assert!(!sim.drones.channels.on(3), "sister: the body touch alone never writes ch 3 [M 0x1b2318:23-33]");
    }
    let mut and_at = None;
    for _ in 0..120 {
        w.tick(nf_game::player::Input::default(), true);
        if and_at.is_none() && w.scene.sim.as_ref().unwrap().drones.channels.on(8) { and_at = Some(w.total_ticks); break; }
    }
    assert!(and_at.is_some(), "E on the plate never raised ch 8 (the AND of the body touch + the E arm [D #560; M 0x1b2318:23-33]) within 120 ticks");
    let mut open_at = None;
    for _ in 0..900 {
        w.tick(nf_game::player::Input::default(), true);
        let sim = w.scene.sim.as_ref().unwrap();
        let door = sim.drones.sss.list.iter().find(|s| s.channel == 8).expect("the ch-8 door SS [D #544]");
        if door.state == 0 { open_at = Some((w.total_ticks, door.pos())); break; }
    }
    assert!(open_at.is_some(), "the door never reached its spent (wrap-1 [D #544]) open pose — state {:?}",
        w.scene.sim.as_ref().unwrap().drones.sss.list.iter().find(|s| s.channel == 8).map(|s| (s.state, s.t)));
    eprintln!("[0c] the secret door swept spent open at +{:?}", open_at.map(|(t, _)| t));
    // Walk WEST through the spent door for real (the doorway is in the corridor's west wall —
    // the closed leaf sat at x ≈ 52.0 spanning z −174.3..−173.1 [D this session's x-rays]) and
    // step down onto the warehouse yard (0.69) — the REAL descent the master's stand-in
    // replaced. The opening's top clears the standing body (the frame's top surfaces ≥ 3.0
    // [D this session's scan]; the standing capsule tops at root + 0.825 ≈ 2.76 [M
    // Player_Collision 0x194df0 default arm]).
    w.walk_to(Wp::new(50.4, 1.7, -174.2).r(0.55), "west through the open secret door onto the yard");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let under = sim.world.segment_hit([sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]],
            [sim.bond.pos[0], sim.bond.pos[1] - 3.0, sim.bond.pos[2]]).map(|(t, _)| sim.bond.pos[1] - 3.0 * t);
        assert!(sim.bond.grounded && sim.bond.pos[0] < 51.5 && under.map_or(false, |y| (0.4..1.0).contains(&y)),
            "Bond must walk west through the open secret door onto the yard (0.69 [D this session's scan]), got pos {:?} under {under:?}",
            sim.bond.pos);
    }
    eprintln!("[0c] the yard reached for real through the open secret door — on the yard floor at {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);

    // Leg 6 (REAL): south-west across the yard to the ladder base.
    w.walk_to(Wp::new(48.0, 1.4, -179.0).r(1.5), "the yard south of the door");
    w.walk_to(Wp::new(40.0, 1.4, -184.5).r(2.0), "the yard west");
    w.walk_to(Wp::new(20.0, 1.4, -184.5).r(1.5), "the yard west 2");
    w.walk_to(Wp::new(11.6, 1.2, -186.6).r(1.2), "the ladder base");
    // Leg 7 (REAL): the ladder walk-in grab + climb (the walk substate rides the DEFAULT arm
    // r 0.55 [M 0x1953f4 via the jump table @0x2fc0d0]).
    w.walk_to(Wp::new(10.3, 1.05, -186.8).r(0.7), "the ladder approach");
    let mut climbed = false;
    for _ in 0..900 {
        let sim = w.scene.sim.as_mut().unwrap();
        if sim.bond.substate == nf_game::player::SS_CLIMB { climbed = true; break; }
        // yaw = atan2(dx, dz) — forward = (sin yaw, cos yaw) — [S] unverified (Bond::look is the port's look maths; the game's heading convention is not pinned here).
        let desired = (9.669741f32 - sim.bond.pos[0]).atan2(-187.3427f32 - sim.bond.pos[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = nf_game::player::Input {
            turn: if err.abs() > 0.2 { err.clamp(-0.08, 0.08) } else { 0.0 },
            forward: if err.abs() > 0.2 { 0.3 } else { 1.0 },
            ..Default::default()
        };
        sim.step_bond(input, false, false, false);
        w.total_ticks += 1;
    }
    assert!(climbed,
        "walking into the yard ladder #164 never grabbed it — the walk substate must ride the switch's DEFAULT arm r 0.55 [M 0x1953f4 via the jump table @0x2fc0d0]");
    // Climb for real: hold forward until the mantle fires (the dismount at pos.y > climb_top
    // 12.139 [M Player_Climb 0x196310:54-62 + Player_CollLadder 0x196118's +0.85 top]) and the
    // walk resumes, then keep the stick forward through the lunge (+0.6 [M 0x196310:56]).
    let mut mantled = false;
    for _ in 0..2400 {
        let sim = w.scene.sim.as_mut().unwrap();
        if sim.bond.substate != nf_game::player::SS_CLIMB && sim.bond.pos[1] > 2.0 { mantled = true; break; }
        sim.step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    assert!(mantled, "the yard climb never mantled past climb_top 12.139 [M 0x196118/0x196310], y {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    for _ in 0..600 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.grounded && sim.bond.substate == nf_game::player::SS_WALK && sim.bond.pos[1] > 10.5 { break; }
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && sim.bond.pos[1] > 10.5,
            "Bond lands on the ladder-top walkway (11.086 [D probes]) after the mantle, got {:?} grounded {}",
            sim.bond.pos, sim.bond.grounded);
    }
    eprintln!("[0c] the yard ladder climbed for real to y {:.2}", w.scene.sim.as_ref().unwrap().bond.pos[1]);

    // Leg 8 (REAL): the patrol-ring diagonal from the ladder top to the north-west corner
    // ((6-9,−188) → (9-11,−190) → (11-12,−192) → (12-13,−194) [D probes]) and west along the
    // north-west walkway to its west edge (0..8, −208 [D probes]).
    w.walk_to(Wp::new(10.5, 11.1, -190.5).r(1.2), "the ring diagonal");
    w.walk_to(Wp::new(12.0, 11.1, -196.0).r(1.2), "the ring diagonal 2");
    w.walk_to(Wp::new(12.0, 11.1, -203.0).r(1.2), "the ring north-west corner");
    w.walk_to(Wp::new(4.0, 11.1, -207.5).r(1.2), "the ring west edge");
    // Leg 8b (REAL, blocker 4 replaced): the west-gap wire crossing. The disc's own mechanism
    // [D probe blk/blk2]: the type-3 grab icon #4 (−3.30, 12.36, −207.51) sits at the west
    // edge's south end; `wire_firstsilo2mainsilos` #284's grab segment A (−4.63, 13.67, −209.88)
    // B (−8.47, 13.67, −216.52) spans the gap, and BOTH ends have landing floor: the west edge
    // (11.1) at A's side and the west ledge (11.24-11.38, x −12..−8, z −216..−218) at B's side.
    // The grab: the jump grab with the icon's ctx 3 [M Player_HandleJump 0x1916b0:118-147] or
    // the body-contact grab [M Player_CollWire 0x197130]; the ride is the strafe tree [M
    // Player_Wire 0x197658], the drop the action press [M 0x197758].
    let grabbed = {
        let mut got = false;
        for t in 0..240u32 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.substate == nf_game::player::SS_WIRE { got = true; break; }
            let (p, grounded) = (sim.bond.pos, sim.bond.grounded);
            if !grounded && p[1] < 9.0 { break; }
            let desired = (-4.6f32 - p[0]).atan2(-209.9f32 - p[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let input = nf_game::player::Input {
                turn: err.clamp(-0.08, 0.08),
                forward: if err.abs() > 0.6 { 0.0 } else { 1.0 },
                jump: grounded && p[0] < 2.0 && t > 6,
                strafe: if sim.bond.substate == nf_game::player::SS_WIRE { 1.0 } else { 0.0 },
                ..Default::default()
            };
            w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
            w.total_ticks += 1;
        }
        got
    };
    assert!(grabbed, "the west-edge jump never grabbed wire #284 (icon ctx 3 [D #4] or the body sphere [D probe blk])");
    // Ride to B (−8.47, −216.52): strafe +1 faces wire.yaw + 3π/2 = toward B [M Player_Wire's
    // mode-7 facing]; drop with the action press inside 0.9 of B.
    let dropped = {
        let mut dropped = false;
        for _ in 0..2400u32 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.substate != nf_game::player::SS_WIRE { dropped = true; break; }
            let d = ((sim.bond.pos[0] + 8.47).powi(2) + (sim.bond.pos[2] + 216.52).powi(2)).sqrt();
            // The drop is the jump press inside 0.9 of B (blockers-2's green drop). The
            // FIRE-action variant (real-crossings 29b43fa9, never run on that branch) rode the
            // full 2400 ticks to the natural stop d_B ≈ 1.05 past B WITHOUT releasing [this
            // run's hang at (−8.98, 12.65, −217.44) — the y 12.65 hang root = wire 13.67 − 1.02
            // [M hang 1.0207]], so the wire release does not answer the trigger here; the jump
            // press does (blockers-2's three green drops).
            let input = nf_game::player::Input { strafe: 1.0, jump: d < 0.9, ..Default::default() };
            w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
            w.total_ticks += 1;
        }
        dropped
    };
    for _ in 0..200 {
        if w.scene.sim.as_ref().unwrap().bond.grounded { break; }
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        w.total_ticks += 1;
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(dropped && sim.bond.grounded && p[0] < -6.5 && p[0] > -13.0 && p[2] < -214.5 && p[2] > -219.5
            && (p[1] - 12.4).abs() < 0.8,
            "the #284 ride must drop onto the west ledge (11.24-11.38 [D probe blk2 west map]), got {p:?} dropped {dropped}");
    }
    eprintln!("[0c] the west gap crossed on wire #284 — landed {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    // The diagonal ledges south-east to icon #6 [D probe blk2 west map].
    w.walk_to(Wp::new(-6.0, 12.4, -219.6).r(1.5), "the west ledge diagonal");
    w.walk_to(Wp::new(-4.5, 12.4, -221.5).r(1.5), "the west ledge diagonal 2");
    w.walk_to(Wp::new(-2.5, 12.4, -222.8).r(1.4), "the west ledge diagonal 3");
    w.walk_to(Wp::new(-1.4, 12.4, -223.6).r(1.0), "icon #6 at wire #282's B end");
    // REAL: grab wire #282 (`wire_silosilo2`, A (5.85, 14.11, −223.15) B (−0.25, 14.11, −223.15)
    // [D probe blk]) — the standing body at (−1.4, −223.6) is inside the wire's world sphere
    // (centre (2.80, 14.11, −223.15) r 5.05 [D probe blk]), so the walk-east contact grab fires
    // [M Player_CollWire 0x197130]; the jump grab at icon #6 (ctx 3 [D]) is the fallback.
    let grabbed = {
        let mut got = false;
        for _ in 0..240u32 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.substate == nf_game::player::SS_WIRE { got = true; break; }
            let grounded = sim.bond.grounded;
            let input = nf_game::player::Input {
                forward: 1.0,
                jump: grounded,
                strafe: if sim.bond.substate == nf_game::player::SS_WIRE { -1.0 } else { 0.0 },
                ..Default::default()
            };
            w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
            w.total_ticks += 1;
        }
        got
    };
    assert!(grabbed, "walking east at wire #282's B end never grabbed it (sphere r 5.05 [D probe blk])");
    // Ride EAST to A (5.85, −223.15): strafe −1 (mode 8 faces wire.yaw + π/2 = +X toward A) [M
    // Player_Wire's mode-8 facing]; drop inside 0.9 of A.
    let dropped = {
        let mut dropped = false;
        for _ in 0..2400u32 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.substate != nf_game::player::SS_WIRE { dropped = true; break; }
            let d = ((sim.bond.pos[0] - 5.85).powi(2) + (sim.bond.pos[2] + 223.15).powi(2)).sqrt();
            // The drop is the jump press inside 0.9 of B (see the #284 note).
            let input = nf_game::player::Input { strafe: -1.0, jump: d < 0.9, ..Default::default() };
            w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
            w.total_ticks += 1;
        }
        dropped
    };
    for _ in 0..200 {
        if w.scene.sim.as_ref().unwrap().bond.grounded { break; }
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        w.total_ticks += 1;
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(dropped && sim.bond.grounded && p[0] > 3.0 && p[0] < 9.0 && p[2] < -220.0 && p[2] > -227.0
            && (p[1] - 12.4).abs() < 0.8,
            "the #282 ride must drop onto the (4..8, −221..−226) ledge (11.24 [D probe blk2]), got {p:?} dropped {dropped}");
    }
    eprintln!("[0c] wire #282 ridden east — landed {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    // The SW arc: the ledge to the arc's west end and east along it [D probes].
    w.walk_to(Wp::new(8.0, 12.4, -222.0).r(1.0), "the ledge to the arc");
    w.walk_to(Wp::new(13.0, 12.4, -218.5).r(1.2), "the arc west end");
    w.walk_to(Wp::new(24.0, 12.4, -218.0).r(1.5), "the ring south-west arc");
    // REAL: grab wire #283 (`wire_silo2turbine`, A (29.58, 14.44, −216.96) B (35.89, 14.44,
    // −209.44) [D probe blk]) at the arc's east end — the body sphere-HIT is confirmed at
    // (28, 12.3, −217.5) [D probe blk]; icon #12 (28.42, 12.24, −218.43, ctx 3 [D]) marks it.
    let grabbed = {
        let mut got = false;
        for _ in 0..240u32 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.substate == nf_game::player::SS_WIRE { got = true; break; }
            let grounded = sim.bond.grounded;
            let input = nf_game::player::Input {
                forward: 1.0,
                jump: grounded,
                strafe: if sim.bond.substate == nf_game::player::SS_WIRE { 1.0 } else { 0.0 },
                ..Default::default()
            };
            w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
            w.total_ticks += 1;
        }
        got
    };
    assert!(grabbed, "walking east at the arc's end never grabbed wire #283 (the sphere HIT [D probe blk])");
    // Ride to B (35.89, −209.44): strafe +1 (mode 7 faces wire.yaw + 3π/2 = north-east toward B)
    // [M Player_Wire]. The ride must run to the wire's END before the jump: the corner floor's
    // west edge is x ≈ 35.8 (10.89 step at z −210..−209.8, the 11.06..11.09 corner east of it
    // [D scratch probe rays; the corner 11.09..11.17 at x ≤ 40.5, z −210..−208 D probe door2]),
    // so the old `jump at d < 0.9` dropped Bond at x ≈ 35.15 — 0.65 WEST of the edge, over the
    // void (no face y 15..8 under (35.15, −209.95) [D rays]) and he fell to the yard. The
    // strafe zero rule parks the idle at d_B ≈ 0.5 [M 0x1977c0's `d_b <= 0.5`], but the playing
    // mode-7 move anim finishes its root motion and overshoots to d_B ≈ 1.1 PAST B — x ≈ 36.7,
    // over the 11.06 corner floor. Jump there (the ride's natural stop, d no longer shrinking).
    let dropped = {
        let mut dropped = false;
        let mut best_d = f32::MAX;
        let mut still = 0u32;
        for _t in 0..2400u32 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.substate != nf_game::player::SS_WIRE { dropped = true; break; }
            let d = ((sim.bond.pos[0] - 35.89).powi(2) + (sim.bond.pos[2] + 209.44).powi(2)).sqrt();
            // Hold the jump one tick past the stop so the exit reads the overshoot pose.
            let at_stop = still >= 90;
            if d < best_d - 1e-4 { best_d = d; still = 0; } else { still += 1; }
            let input = nf_game::player::Input { strafe: 1.0, jump: at_stop, ..Default::default() };
            w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
            w.total_ticks += 1;
        }
        dropped
    };
    // Settle through the REAL fall: the wire ride leaves `grounded` stale-true on the exit tick
    // (wire_tick zeroes vel and returns; the flag clears the next tick [M wire_tick 0x197758's
    // exit]), so the old `if grounded break` snapped the assert onto the HANG pose — the merge
    // RED's [35.15, 13.55, −209.95] is exactly the hang root (wire y 14.44 − 1.02 [M hang
    // 1.0207]). Wait for air, then ground.
    let mut saw_air = false;
    for _ in 0..300 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.grounded && saw_air { break; }
        if !sim.bond.grounded { saw_air = true; }
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        w.total_ticks += 1;
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(dropped && sim.bond.grounded && p[0] > 33.5 && p[0] < 40.5 && p[2] < -207.0 && p[2] > -211.5
            && (p[1] - 12.2).abs() < 0.8,
            "the #283 ride must drop onto the ring SE corner (11.09..11.17 [D probe door2]), got {p:?} dropped {dropped}");
    }
    eprintln!("[0c] the south gap crossed on wire #283 — landed {:?}", w.scene.sim.as_ref().unwrap().bond.pos);

    // Leg 9 (REAL from the SE corner through the door): the district's true geometry [D, probe
    // walkreal_probe0c_door2_live]: the corner (11.10..11.17, x ≤40.5, z −210..−208) is where
    // #283's B end drops Bond; the corridor (11.09, x 38.05..39.55, z −212.75..−212) runs to
    // door2turbine #281's mouth; the E stance (39.0, 11.09, −210.8) — the live sphere (40.01,
    // 12.57, −209.69) r 1.62 within the 2.62 pick [M Collide_PickObj 0x1eae80's 0x101 arm].
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let wall = sim.world.segment_hit([40.2, 13.0, -213.51], [40.2, 11.9, -213.51]);
        assert!(wall.is_some(), "the slot end wall at z −213.51 (y 12.02..14.22 [D room_turbine01 tri 463]) vanished");
    }
    // The corner to the door stance (the port's kind-0xDB doors carry no leaf collision [M
    // build_LinkToRoom 0x1c9680:63-66, Collide_Pick 0x1eb320:369-375]).
    w.walk_to(Wp::new(38.9, 11.1, -210.8).r(0.7), "the corner to the door stance");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // Sister: pin the door closed right before the E press — if a fix opens it early, this
        // leg must change.
        let door = sim.drones.doors.list.iter().find(|d| (d.pos[0] - 40.0).abs() < 0.5 && (d.pos[2] + 210.4).abs() < 0.5)
            .expect("the door2turbine #281 [D]");
        assert_eq!(door.state, 0x2, "door2turbine opened before the E press");
    }
    // The REAL E press at the door mouth: face the leaf, pulse E every 6 ticks.
    // [M 0x135be0: info+0x68 (ch_lock) == 0 skips the channel gate; kind 0 (obj+0xf6) closed
    //  (obj+0xf4 & 0x12 == 0) → Door_SetState(obj, 8); M 0x135618: stores obj+0xf4 = 8 and
    //  writes switch_channels[info+0x66] = (8 & 0xc != 0) → the door's ch_open 5 turns on.]
    let mut opened = false;
    for t in 0..90u32 {
        let turn = {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            let desired = (40.0f32 - b.pos[0]).atan2(-210.4f32 - b.pos[2]);
            walker::wrap_pi(desired - b.yaw).clamp(-0.08, 0.08)
        };
        w.tick(nf_game::player::Input { turn, ..Default::default() }, t % 6 == 0);
        if t % 6 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            if let Some(d) = sim.drones.doors.list.iter().find(|d| (d.pos[0] - 40.0).abs() < 0.5 && (d.pos[2] + 210.4).abs() < 0.5) {
                if d.state != 0x2 { opened = true; break; }
            }
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let door = sim.drones.doors.list.iter().find(|d| (d.pos[0] - 40.0).abs() < 0.5 && (d.pos[2] + 210.4).abs() < 0.5)
            .expect("the door2turbine #281 [D]");
        assert!(opened, "the E press at the corridor mouth never moved door2turbine (state {})", door.state);
    }
    eprintln!("[0c] door2turbine moved on the real E press (the swing-open arm)");
    // The channel write lands on the transitions where the OLD state's 0xc bits are set [M
    // 0x135618's switch_channels store]: the E press's OPENING (8) write happens with the old
    // CLOSED (2) → stays off; the channel turns ON exactly when the swing completes
    // (set_state(OPEN=4) with the old OPENING=8). Poll for that completion.
    let mut open_settled = false;
    for _ in 0..300 {
        w.tick(nf_game::player::Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(5) { open_settled = true; break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let door = sim.drones.doors.list.iter().find(|d| (d.pos[0] - 40.0).abs() < 0.5 && (d.pos[2] + 210.4).abs() < 0.5)
            .expect("the door2turbine #281 [D]");
        assert!(open_settled, "the door never completed its swing with ch_open 5 on [M 0x135618 via the OPEN transition], state {:#x}", door.state);
        // The ch5-on observable PROVES the OPEN transition ran (the channel only turns on from
        // an old state with the 0xc bits [M 0x135618]); the door's auto-close [D audit: #281
        // auto-close] may already have moved it to CLOSING by the time the poll breaks.
        assert!(door.state == nf_game::door::OPEN || door.state == nf_game::door::CLOSING,
            "the door state after the swing [M 0x135618], got {:#x}", door.state);
        assert!(sim.drones.channels.on(5), "the door's OPEN wrote its ch_open 5 [M 0x135618's switch_channels store]");
    }
    eprintln!("[0c] door2turbine opened on the real E press (state OPEN, ch 5 on)");
    // REAL: through the doorway (the port's kind-0xDB doors carry no leaf collision — the game
    // picks them through the ForcedList [M build_LinkToRoom 0x1c9680:63-66, Collide_Pick
    // 0x1eb320:369-375]) onto the corner.
    w.walk_to(Wp::new(39.4, 11.1, -209.6).r(0.9), "through the doorway onto the corner");
    // The corner's east boundary is a wall (x 40.01..40.37) with a 1.52-wide doorway tunnel
    // (z −208.93..−210.45, floor 11.10, ceiling 13.96 [D room_turbine01 tris 478-485 — no
    // x-face tris: open along x]); the band's west lip exists only at z ≥ −209.0 [(43.4, 10.89)
    // / (44.2, 11.17) [D map]]. Run up west of the wall, THROUGH the tunnel, and jump east of it.
    w.walk_to(Wp::new(38.6, 11.1, -209.7).r(0.5), "the jump run-up start");
    let mut jumped = false;
    for t in 0..300u32 {
        let (p, grounded) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            (b.pos, b.grounded)
        };
        // Land (after a real airborne phase) → the assert judges where.
        if jumped && grounded && t > 20 { break; }
        // Still falling deep below the band's lip → the shaft; the assert reports it.
        if !grounded && jumped && p[1] < 9.0 { break; }
        let (turn, forward) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            let desired = (44.5f32 - b.pos[0]).atan2(-208.8f32 - b.pos[2]);
            let err = walker::wrap_pi(desired - b.yaw);
            (err.clamp(-0.08, 0.08), if err.abs() > 0.6 { 0.0 } else { 1.0 })
        };
        // Fire the jump while grounded just EAST of the wall's east face (x 40.37 [D]) — full
        // run speed through the 1.8-unit run-up and the tunnel.
        let jump = !jumped && grounded && p[0] >= 40.42;
        if jump { jumped = true; }
        w.tick(nf_game::player::Input { turn, forward, jump, ..Default::default() }, false);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let under = sim.world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 3.0, p[2]])
            .map(|(t, _)| p[1] - 3.0 * t);
        assert!(sim.bond.grounded && p[0] > 42.6 && under.map_or(false, |y| (10.2..11.7).contains(&y)),
            "the slot jump must land on the band (10.89..11.17 floor at x 43.4+ [D]), got pos {p:?} under {under:?}");
    }
    eprintln!("[0c] the slot jump landed on the band at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    w
}

/// Fire the held weapon at a static point through the real trigger path (the walker's aim loop;
/// kept local to keep walker.rs additive).
fn shoot_at(w: &mut Walker, target: [f32; 3], range: f32, max_ticks: u32, _label: &str, done: &dyn Fn(&nf_view::scene::Sim) -> bool) -> bool {
    let start = w.total_ticks;
    let mut last_tap = 0u32;
    loop {
        if done(&w.scene.sim.as_ref().unwrap()) { return true; }
        if w.total_ticks - start >= max_ticks { return false; }
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let eye = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
            let dx = target[0] - bp[0];
            let dz = target[2] - bp[2];
            let dy = target[1] - eye;
            if (dx * dx + dy * dy + dz * dz).sqrt() > range { return false; }
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let want_pitch = (dy / walker::dxz(bp, target).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let on_target = err.abs() < 0.008 && perr.abs() < 0.012 && w.total_ticks - last_tap > 15;
            let trigger = on_target;
            if trigger { last_tap = w.total_ticks; }
            let input = if trigger {
                nf_game::player::Input { aiming: true, ..Default::default() }
            } else {
                nf_game::player::Input {
                    turn: err.clamp(-0.08, 0.08),
                    look: perr.clamp(-0.08, 0.08),
                    aiming: true,
                    ..Default::default()
                }
            };
            w.scene.sim.as_mut().unwrap().step_bond(input, trigger, false, false);
        }
        w.total_ticks += 1;
        if w.top_up_health { w.scene.sim.as_mut().unwrap().drones.health.health = 100.0; }
    }
}

