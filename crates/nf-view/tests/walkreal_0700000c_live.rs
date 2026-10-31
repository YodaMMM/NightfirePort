//! walkreal-c: 0700000c (Chain Reaction, part 1 — the power station) walked with the player's own
//! movement — `step_bond` input only: no channel or state pokes. The REAL legs this session:
//! the laser-watch hinge burn (the four #139-142 locks through the held-fire path), the vent
//! drop firing plate #274 mid-fall (objective 1 on the game's own route), the 11.27 hall +
//! the upper east lane south, the yard reached WEST through the open secret door, and the three wire
//! crossings (jump-grab + hand-over-hand rides), and the STAIRCASE descent to the display
//! ground (the hall floor's own opening over `obj_stor2_staircase01` #359's ramp [D] — the
//! display eye reached by real input). The DOCUMENTED stand-ins left (each with its evidence
//! inline): the case's trench exit (the vent shaft is a ONE-WAY DROP in the disc's own data:
//! the south crawl pocket is sealed by the wall slab z −112.34..−112.363 and the crouch
//! capsule stops ON it at −111.813 [D this session's block dump + M 0x194df0 case 3]; the
//! game holds the body too [L 2026-10-26 hatch-3]; no breakable glass exists [D census]), the
//! plates reach (every
//! walking route off the display-ground complex is closed — the slab's south face leaves a
//! 0.23..0.4 slot above the 2.37 deck [D probe rays]; the game's own connection is the
//! warehouse-side corridor/annex, a live check), the east
//! climb ground→band (PINNED an island, level-0c-east-climb: the disc's own collision connects
//! NOTHING to the band deck — staircase01's bottom 9.52 floats over the 6.35 under-stairs [D
//! eastclimb3/4], the turb4 stair chain's slot x 81.5..83.7 is EMPTY 6.5..9.3 [D eastclimb5/6],
//! the band strip gaps 10.4 m over the chasm, the north-strip mezzanine breaks at x 54..64.5,
//! the crates4 gap is 3.1 m + 2.06 [D eastclimb6/7]; the deck is unreachable in the flood-fill
//! at step law 0.55 AND 0.80 [D eastclimb7]; no lift kind, no east-side mover, west-only
//! ladders/grapples [D kind census]; the nav patrols G3/G4 are separate [D nav dump]; the
//! stand-in lifts Bond onto the band at the gate — the live checks decide whether the real
//! game has a dynamic the parse cannot see), and the ladder495 sister test's west-ground placement. Route: the ST-1
//! walkway start #1 (8.98, 15.07, −149.65 [D]) → north along the walkway (14.61 floor [D probe blk],
//! east of the raised block x 5..10 [D probe0c-lane]) → the inner balcony repro spot (7.61, 15.71,
//! −113.39 [L 991]) → the LASER-WATCH burn of the four hinge locks #139-142 through the held-fire
//! path [M Lock_Update 0x1ae3d8:82-101 + Player_WeaponFiring 0x1a5460:653-658] → the AND #505 →
//! ch 18 sweeps the vent open (SS #138/#179, the y-16.3 ray the end-pose pin) → through the open
//! vent into the display-case trench (x 5..6.5, z −110.5..−111.5, floor 11.84 [D probe blk]) →
//! the fall crosses plate #274 (4.63..6.95, 13.63..14.56, −112.08..−110.12 [D]; the game fires it
//! while the body's volume crosses and clears at rest [L 2026-10-26 hatch-2]) → ch 0x1b (objective 1)
//! → the lower hall east lane south to the STAIRCASE HEAD — the hall floor's own opening
//! (x 9..12.4, z −115.5..−131.5 [D probe rays]) crossed by `obj_stor2_staircase01` #359's ramp
//! (the landing 11.12..11.29 at z −127..−128, down to 6.29 at z −120 [D probe rays; the kind-0x19
//! mesh bounds x 9.97..12.29, y 6.13..12.14, z −127.5..−117.1]) → the ramp descent to the
//! display-ground level (6.29 bottom, x 10.5..12.3, z −119.5..−121) → the south ledge (6.27,
//! x 12.6..14.5 [D probe rays]) north to the display pocket → the REAL camera photograph (0x54
//! [M 0x1a7ef8 case 0x700000c]) into the CamSubject #454 volume → ch 0x1c (objective 2) → the
//! DOCUMENTED plates-reach stand-in (the walking routes off the display-ground complex are
//! all closed — see Leg 4; this session also mapped the far side: the plates stand in a
//! ground-level corridor (0.90) running north under the warehouse mezzanine, whose only
//! proven entrances are the secret door and an unproven north end) onto the secret-door plates
//! #558/#559 (52.24..53.29, 0.90..3.20, −174.45..−172.73 [D]): the REAL body-touch + E-arm AND chain
//! → Door_secret1 #544 sweeps (rate 400 [D]) → through the open door WEST for real onto the
//! yard (0.69 [D this session's scan]) — the master's yard-descent stand-in is GONE — → the yard ladder #164 (9.67, 5.53, −187.34 [D])
//! → the REAL walk-in grab + climb (the walk substate rides `Player_Collision` 0x194df0's DEFAULT
//! hittest arm r 0.55 [M 0x1953f4 via the jump table @0x2fc0d0]) → the patrol-ring diagonal walkway
//! (the ladder map: (6-9,−188) → (12-13,−194) [D probes]) → the ring north-west corner (12,−202..−208)
//! → west edge (0-8,−208) → the REAL west-gap wire crossing: the jump grab at the type-3 icon #4
//! (−3.30, 12.36, −207.51 [D icons]) onto `wire_firstsilo2mainsilos` #284 (grab A (−4.63, 13.67,
//! −209.88) B (−8.47, 13.67, −216.52) [D probe blk], hand-over-hand [M Player_CollWire 0x197130 +
//! Player_Wire 0x197658; the port's wire.rs]) → drop at B onto the west ledge (11.24-11.38, x −12..−8,
//! z −216..−218 [D probe blk2 west map]) → the diagonal ledges south-east (11.24: (−8,−219) → (−2,−223)
//! [D]) → the jump grab at icon #6 (−1.54, −223.57 [D]) onto `wire_silosilo2` #282 (A (5.85, 14.11,
//! −223.15) B (−0.25, 14.11, −223.15) [D probe blk]) → ride east to A, drop onto the (4..8, −221..−226)
//! ledge (11.24 [D]) → the ring south-west arc (12-28,−218) → the jump grab at icon #12 (28.42, 12.24,
//! −218.43 [D]) onto `wire_silo2turbine` #283 (grab A (29.58, 14.44, −216.96) B (35.89, 14.44, −209.44)
//! [D probe blk; the body sphere-HIT at (28, 12.3, −217.5) confirmed]) → ride to B, drop onto the ring
//! SE corner (11.09..11.17, x ≤ 40.5, z −210..−208 [D]) → the REAL door2turbine #281 E press [M
//! 0x135be0/0x135618] through to the corner → the REAL jump over the x 41..43 slot onto the gate band
//! → the band TouchOnce #0xb4 (45.68..46.66, 10.34..11.32, −208.63..−206.52 [D]) → the band east to
//! its true end x 55.0 [D probe door4 fine map] → the band crossing is REPLACED by the game's
//! own ground route, walked on real input (the eastroute legs, green on master): turb2's
//! staircase flights down [D band9 plc20-22], the y-0 yard east under the chasm [D the nav
//! kind-0x202 ring path], the running jump onto the turb4 lower ramp [D the 0.25-grid step
//! scan], up to the east ground 6.17 [D plc42] — the straight crossing is dead ([L 2026-10-30
//! fresh boot: Bond drops into the pit] + the void sisters below) → the east climb STAND-IN (the
//! only unproven link: the deck is a collision ISLAND — the disc's static data connects nothing;
//! staircase01's bottom step 9.52 floats over the 6.35 under-stairs at x 84.5 [D eastclimb3/4],
//! the stair slot x 81.5..83.7 is empty 6.5..9.3 [D eastclimb5/6], the band's south rail 11.17
//! walls the deck [D eastclimb3], the 0.78 jump apex [M player.rs]; sisters 5b/5c pin the slot
//! and the three remaining crossing attempts) lifting Bond onto the band
//! at (95, −209.5), deck 10.34 [D plc44] → the gate steps → E → level_exit 0x0700000d ([M Player_Activate
//! 0x1a5170 case 0x1d → Trigger_Activate 0x1b2318:111], the key gate ch 0x1c on) → the warm RamSave
//! carry into 0700000d → the spawn TouchOnce #436 (179.53, 11.58, −205.40, out 0x23 [D 0d]) fires
//! → objective 3 completes and 0d's own objective reveals ([M Mission_Init 0x182270:92-121 carries
//! record 11 across the load]).

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn walk_the_whole_mission_from_the_start_to_the_0d_handoff() {
    let iso = iso();
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
    let plate2 = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.triggers.iter()
            .find(|t| (t.kind == 1 || t.kind == 2) && t.out == 2 && !t.tris.is_empty())
            .map(|t| (t.placement, t.centre(), t.mask))
            .unwrap_or_else(|| panic!("the out-ch 2 touch plate #559 [D]"))
    };
    {
        let sim = w.scene.sim.as_mut().unwrap();
        // The plates-reach stand-in, unchanged from master (see Leg 4's comment for the
        // probe evidence closing the walking routes). Sisters below pin the corridor's own
        // geometry so the stand-in's landing stays on the disc's floor.
        sim.bond.pos = [plate2.1[0], plate2.1[1] + 0.6, plate2.1[2]];
        sim.bond.vel = [0.0; 3];
    }
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
    eprintln!("[0c] the secret-door reach stand-in (the closed display-ground descents) — on the plates");

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

    // Leg 10 (REAL): the band east to its true end. The disc's band floor 10.34 runs x 44..55.6
    // (z −209.06..−206.31: obj_turb1_walkway01 x 43.2..49.3 + obj_turb2_walkway01 x 49.3..56.1
    // [D band2 probe's tri dump]); x 56.1..66.3 at band height is VOID — the 6.17 pit (x 55..63.5),
    // the 7.19 shelf (x 60..64 at z −209.0 only), then a floor-less chasm x 64..66.5 down past
    // y 2 [D eastclimb probe's 0.5 map], to the resumed band 10.34 at x 67.5+ (obj_turb3_walkway04
    // [D]; the pit floor itself is obj_turb2_walkway04 plc28 x 49.29..63.66, y 6.16..7.19,
    // z −209.06..−205.12 [D band9 probe's placed extents], continuing east as the sliver
    // obj_turb3_walkway03 plc33 x 63.66..64.05 (y 6.17..7.19) — then the chasm x 64.05..66.27
    // down past y 2, and the resume sits 10.16+ — +3.2 ABOVE the pit floor: the pit route
    // dead-ends, no jump crosses up [D band9 extents vs the 5.2-unit carry]. The staircases are
    // symmetric FLANKS, not a crossing: turb2's rise 2.97→11.17 at x 49.29..58.73 and turb4's
    // 0.16→11.20 at x 78.14..87.59 (both z −208.96..−211.67, SOUTH of the band) — they connect
    // the yards and the gate ground to the band heights, nothing spans x 59..78 [D band9
    // extents]. The supportbeams are columns (turb2 pair x 53.57..54.57 y 14.25..18.03; turb3
    // #30 x 68.02..68.82 y 6.08..17.24 — 0.8 wide, no deck [D band9]). The 10.85/10.17 strip
    // (z −206.33..−205.12) is NO ROUTE: its west part x 49.3..53.57 ends in a 4-unit wall at
    // x 53.57 (y 10.85..14.85 [D lipline probe's tris]) — a cul-de-sac — and the east slope's
    // west tip (x 58.86 at 10.17 [D]) recedes to x 61.5 at the flight's z −205.7 (the tri
    // (58.86,−205.12)|(62.19,−205.34)|(63.32,−206.08) [D]): a 7.98-unit gap against the
    // engine's 5.2-unit drop-jump carry (3.92 m/s up, 9.8 gravity [M player.rs], 5.5 run — the
    // -2 session measured 4.42 flat) — IMPOSSIBLE. Nothing carries across: the
    // full leaf-coverage audit decodes EVERY triangle the disc declares (0 gaps in all 46 maps
    // [D collaudit probe]) and none lands at band height over the void; no SS (all 9 listed:
    // two display vents, two warehouse vents, three storeroom pulleys, Door_secret1, and the
    // VentmeshBlockingPloy [D band7 probe]), no wire (4 total: three ring wires #282/#283/#284
    // + the type-2 zipline x 46.79 y 24.6→11.6 z −207.0→−182.7 — a NORTH-yard ride [D band7/
    // grapple probes]), no grapple point (5 total, none over the void: the near one gp4 is at
    // (54.27, 15.35, −237.02) — the south yard [D grapple probe]), no script object (the
    // PhoenixForkLift SP #341 script 06000762 drives the yard at y 0, x 21..72.25, z −205..−181
    // [D band6 probe's 2400-tick track]), no door, no drone movement in the box. The catwalk
    // (15.53..15.96) is CONTINUOUS x 40.7..79.6 [D catwalk probe] but has no found access (no
    // ladder within 40 units — the level's three ladders are the origin junk, the yard #164 and
    // the (−6.4, −177.4) one [D]). The game's own nav graph links band-height nodes ACROSS the
    // void (path17 n0 (63.06, 10.74, −207.54) ↔ n1 (49.69, 10.74, −207.29) [D probe door4]) and
    // a guard is PLACED mid-void at band height ((62.99, 11.42, −207.43) plc60 [D]) — the game's
    // own data places a guard spawn mid-void — but the link is NOT floor evidence: the game's
    // node-to-node move test sweeps the body from node to node [M NDrone2_MoveTest 0x155330,
    // the port's mirror `nav::move_test`; its gates are |dy| ≤ 4 and the room boundary
    // [M 0x155330 via drone.rs] — and the game's AI is per-path (NDrone2_AssignAIPath assigns
    // ONE path per drone [M 00154190]; AINetwork_CalcRoute walks the assigned path's nodes
    // [M 00156ba8])], and a horizontal sweep across a HOLE hits no wall and no floor — it
    // passes trivially. A designer-linked pair over a pit tests clean; the guard plc60 spawn
    // (62.99, 11.42, −207.43 [D band8 probe]) pins a placement, not a walkable surface. The
    // port's static world equals the disc's declared collision exactly [D collaudit probe], so
    // the port side has nothing to fix in the parse: the crossing mechanism stays UNPINNED by
    // everything the disc declares (live check 1 below — the fresh-boot PCSX2 watch decides);
    // the walk takes the game's own ground route around (the legs below).
    w.walk_to(Wp::new(50.0, 11.0, -207.6).r(1.5), "the band east 1");
    w.walk_to(Wp::new(54.5, 11.0, -207.6).r(0.8), "the band east to its true end x 55.0 [D probe door4]");
    // THE CROSSING IS REPLACED (level-0c-whole-walk-east-route): the sisters below keep the
    // crossing's negative evidence fresh (the straight crossing is dead — [L 2026-10-30 fresh
    // boot: Bond drops into the pit] — so the walk takes the game's own ground route around);
    // then the proven ground-route legs (the walkreal_0700000c_eastroute_live legs, green on
    // master), then the ONLY stand-in left: the east climb ground→band.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // Sister 1: the band-height void over the pit/chasm (the pit floor 6.17 [D probe door4])
        // must stay BELOW band height at (59, −207.5).
        let gap = sim.world.segment_hit([59.0, 12.0, -207.5], [59.0, 8.0, -207.5]);
        assert!(gap.is_none(), "the band crossing gained floor at band height; the ground route's reason vanished");
        // Sister 2: the strip cul-de-sac wall (x 53.57, y 10.85..14.85, z −206.33..−205.12 [D
        // lipline]) must stand — if it ever vanishes the strip route reopens and the route map
        // changes. (A horizontal ray — the wall is an x-constant plane.)
        let wall = sim.world.segment_hit([53.0, 12.0, -205.7], [54.2, 12.0, -205.7]);
        assert!(wall.is_some(), "the strip's east wall (x 53.57 [D lipline]) vanished; re-check the crossing route");
        // Sister 3: the pit floor's presence and height (obj_turb2_walkway04 plc28 x 49.29..63.66,
        // y 6.16..7.19 [D band9 extents]) — the walked descent's landing floor (the flight's
        // south end). If it moves the descent legs re-derive.
        let pit = sim.world.segment_hit([59.0, 8.0, -207.5], [59.0, 5.5, -207.5]).map(|(t, _)| 8.0 - 2.5 * t);
        assert!(pit.map_or(false, |y| (5.8..7.6).contains(&y)),
            "the pit floor at (59, −207.5) moved (expected 6.16..7.19 [D band9 plc28]), got {pit:?}");
        // Sister 4: the pit's east sliver (obj_turb3_walkway03 plc33 x 63.66..64.05, y 6.17..7.19
        // [D band9 extents]) — the pit's far end before the chasm x 64.05..66.27.
        let sliver = sim.world.segment_hit([63.9, 8.5, -207.5], [63.9, 5.5, -207.5]).map(|(t, _)| 8.5 - 3.0 * t);
        assert!(sliver.map_or(false, |y| (6.0..7.5).contains(&y)),
            "the pit's east sliver at (63.9, −207.5) moved (expected 6.17..7.19 [D band9 plc33]), got {sliver:?}");
    }
    eprintln!("[0c] the crossing sisters hold — taking the game's own ground route around the band");
    // The ground route (the eastroute legs on the walk's own arrival):
    // 1. The staircase descent (the turb2 flights, real walk legs [D band9 plc20-22]).
    w.walk_to(Wp::new(51.5, 11.0, -209.6).r(1.2), "onto the turb2 staircase top");
    w.walk_to(Wp::new(51.5, 4.0, -211.4).r(1.2), "down the turb2 flight");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 49.0 && p[0] < 54.0 && p[2] < -210.5,
            "the descent must reach the flight's south end, got {p:?}");
    }
    // 2. The crate field / y-0 yard east (the nav ring's level [D kind 0x202 path]).
    w.walk_to(Wp::new(62.0, 4.0, -212.0).r(1.5), "east along the south");
    w.walk_to(Wp::new(70.0, 2.0, -212.0).r(1.5), "into the crate field");
    w.walk_to(Wp::new(76.0, 1.2, -210.0).r(1.5), "the yard east");
    w.walk_to(Wp::new(77.0, 1.2, -210.2).r(1.0), "the yard south of the ramp's west landing");
    w.walk_to(Wp::new(79.6, 1.1, -210.2).r(0.5), "the due-east run-up under the ramp's west landing (the eastroute jump start)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let f = sim.world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 20.0, p[2]])
            .map(|(t, _)| p[1] - 20.0 * t);
        assert!(p[0] > 77.0 && p[0] < 80.0 && f.map_or(false, |y| y < 0.5),
            "the yard trunk must reach the ramp's west side at the y-0 floor, got {p:?} under {f:?}");
    }
    // 3. The jump-hops onto the east ramp (1.75 at x 81.5, under-steps 0.75..1.55 at x
    // 80.2..81.2 [D the step scan]) — the player's own idiom: hold forward, jump every time the
    // body is grounded below the exposed steps, stop on them (the walk's slot-jump family).
    let mut hops = 0u32;
    for _t in 0..300u32 {
        let (p, grounded, f) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let (p, grounded) = (sim.bond.pos, sim.bond.grounded);
            let f = if grounded {
                sim.world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 3.0, p[2]])
                    .map(|(tt, _)| p[1] - 3.0 * tt)
            } else { None };
            (p, grounded, f)
        };
        if grounded && p[0] > 80.3 && f.map_or(false, |y| y > 1.0) { break; }
        let jump = grounded && p[0] >= 80.0 && !f.map_or(false, |y| y > 1.0);
        if jump { hops += 1; }
        w.tick(nf_game::player::Input { forward: 1.0, jump, ..Default::default() }, false);
    }
    assert!(hops > 0, "the ramp jump never fired");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let f = sim.world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 20.0, p[2]])
            .map(|(t, _)| p[1] - 20.0 * t);
        assert!(p[0] > 80.3 && f.map_or(false, |y| y > 1.0),
            "the jump must land on the ramp's lower steps (1.75+ [D]), got {p:?} under {f:?}");
    }
    // 4. Up the ramp (0.2 steps) to the east ground (obj_turb4_walkway03 y 6.16..7.19 [D plc42]).
    w.walk_to(Wp::new(84.0, 4.5, -210.2).r(1.0), "up the ramp");
    w.walk_to(Wp::new(87.0, 7.0, -209.8).r(1.0), "the ramp top");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let f = sim.world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 20.0, p[2]])
            .map(|(t, _)| p[1] - 20.0 * t);
        assert!(f.map_or(false, |y| (5.5..7.4).contains(&y)),
            "the ramp top must stand on the east ground level (6.16..7.19 [D plc42]), got {p:?} under {f:?}");
    }
    // The east side reached: west along the east ground past the chasm's east edge (the
    // eastroute test's closing leg), then east under the band to the gate's ground spot.
    w.walk_to(Wp::new(86.0, 7.4, -208.2).r(1.0), "the east ground west");
    w.walk_to(Wp::new(78.0, 7.4, -208.2).r(1.2), "the east ground at the chasm's east");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let f = sim.world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 20.0, p[2]])
            .map(|(t, _)| p[1] - 20.0 * t);
        assert!(sim.bond.grounded && p[0] > 76.0 && f.map_or(false, |y| (6.0..7.4).contains(&y)),
            "the east ground leg must hold (6.17 [D]), got {p:?} under {f:?}");
    }
    // Sister 5 (the east climb's negative, at the climb zone): the upper flight's bottom step
    // FLOATS — obj_turb4_staircase01's extents y 9.00..11.20, x 83.47..87.59 [D plc35]; the
    // all-surfaces scan gives its bottom 9.52..9.68 at x 84.2..84.5 over the under-stair steps'
    // 6.35 top at x 87.0 [D eastclimb3/4 this session]; nothing spans 6.35..9.52 and the jump
    // apex is 0.78 [M player.rs 3.92 m/s up]. A floor appearing in the gap reopens the climb.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let under = sim.world.segment_hit([84.5, 8.7, -210.25], [84.5, 6.7, -210.25]);
        assert!(under.is_none(), "a floor appeared under staircase01's west end (8.7..6.7 at (84.5, −210.25)); the east climb reopened");
        let bottom = sim.world.segment_hit([84.5, 11.4, -210.25], [84.5, 8.7, -210.25]).map(|(t, _)| 11.4 - 2.7 * t);
        assert!(bottom.map_or(false, |y| (9.3..9.9).contains(&y)),
            "staircase01's bottom step moved (expected 9.52..9.68 [D eastclimb3]), got {bottom:?}");
    }
    // Sister 5b (level-0c-east-climb: the stair SLOT — the decisive missing link of the climb
    // chain). The turb4 flights are horizontally disjoint: staircase02 (plc36) tops at 8.10 at
    // x 81.3..81.4 and staircase01 (plc35) starts at 9.45 at x 83.8..84.4 [D the eastclimb5
    // level census this session]; between them (x 81.5..83.7, z −211..−209.5) the slot is EMPTY
    // down to the lower flight's steps 1.75..3.75 [D the eastclimb6 fine map]. Crossing needs a
    // 2.4 m jump that GAINS 1.35 — beyond the 0.78 apex [M player.rs]. A surface appearing in
    // the slot band reopens the climb.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for x in [81.75f32, 82.25, 82.75, 83.25] {
            let slot = sim.world.segment_hit([x, 9.3, -210.0], [x, 6.5, -210.0]);
            assert!(slot.is_none(), "a surface appeared in the stair slot at ({x}, −210.0) 9.3..6.5; the east climb reopened");
        }
        let top36 = sim.world.segment_hit([81.35, 8.5, -211.25], [81.35, 7.6, -211.25]).map(|(t, _)| 8.5 - 0.9 * t);
        assert!(top36.map_or(false, |y| (7.9..8.3).contains(&y)),
            "staircase02's top step moved (expected 8.10 [D eastclimb5]), got {top36:?}");
    }
    // Sister 5c (the band deck is an ISLAND — the flood-fill over the sim's own collision, 0.25
    // grid, directed edges, step law 0.55 AND 0.80 [S the law itself; the island holds at both]):
    // the gate deck + staircase01's deck form one component unreachable from the east ground, the
    // yard, the west band, the corridor, the gallery and the catwalks [D eastclimb7 this
    // session]. The remaining crossing attempts, each beyond the pinned apex:
    // (a) the west band's east end 56.08 → turb3_walkway04's deck 66.5 — 10.4 m at deck height
    //     (the [L 2026-10-30 fresh boot] drop at x 63.4 already pinned the walk-off);
    // (b) the north-strip mezzanine (10.85, z −206.0..−205.1) breaks at x 54..64.5 [D eastclimb6
    //     row −206.0: 6.17 ground, no 10.85 floor];
    // (c) crates4 (tops 7.31/8.31) → walkway02's dead south end 10.37 — 3.1 m + 2.06 up.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // (a) widened: the whole band-height strip over the chasm's west half.
        for x in [57.0f32, 59.0, 61.0, 63.0, 65.0] {
            let gap = sim.world.segment_hit([x, 11.9, -207.5], [x, 8.0, -207.5]);
            assert!(gap.is_none(), "band-height floor appeared over the chasm at ({x}, −207.5); the east climb reopened");
        }
        // (b) the north strip's break: the west mezzanine's 10.85 ends at x 53.5 and the next
        //     north-strip surface is room02's isolated 10.17 sliver [58.9..63.3, −206.1..−205.1
        //     [D eastclimb5 plc9]] — nothing spans x 55..58.5 at 9.5..11.5, so the north route
        //     never bridges the chasm (the sliver itself is an island: 5+ m to the west patch,
        //     and the flood at 0.80 leaves it unconnected [D eastclimb7]).
        for x in [55.0f32, 56.5, 58.0] {
            let gap = sim.world.segment_hit([x, 11.5, -205.7], [x, 9.5, -205.7]);
            assert!(gap.is_none(), "north-strip mezzanine floor appeared at ({x}, −205.7) 9.5..11.5; the east climb reopened");
        }
        // (c) the crates4 air gap: no floor 9.4..10.2 at z −229.0/−228.0, x 93..95.
        for z in [-229.0f32, -228.0] {
            for x in [93.0f32, 94.0, 95.0] {
                let gap = sim.world.segment_hit([x, 10.2, z], [x, 9.4, z]);
                assert!(gap.is_none(), "a surface appeared in the crates4 air gap at ({x}, {z}) 9.4..10.2; the east climb reopened");
            }
        }
    }
    w.walk_to(Wp::new(95.0, 7.3, -209.5).r(1.2), "the gate's ground spot under the band");
    // Sister 6: the gate does NOT fire from the ground (the east-route report's probe: Bond at
    // (95.0, 7.3, −209.5), 120 use pulses — no level_exit): the trigger quad is at y 11.25..13.98
    // [D plc556]; 120 real E pulses pin the negative on this tree.
    for _ in 0..120 { w.tick(nf_game::player::Input::default(), true); }
    assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(),
        "the gate #556 fired from the ground — the climb stand-in's evidence changed");
    // STAND-IN (now ONLY the east climb): lift Bond from the east ground onto the band deck
    // above the same spot — obj_turb5_walkway02's deck 10.34 at (95, −209.5) [D plc44 x
    // 92.38..95.31, y 10.16..11.17, z −227.26..−206.31; the scan z −209.5 x 95.0 = 10.34 [D
    // eastclimb3]]. The deck is a COLLISION ISLAND pinned by sisters 5b/5c: the disc's own
    // static data connects NOTHING to it (the turb4 stair flights are disjoint with an empty
    // slot between; the band strip gaps 10.4 m over the chasm; the mezzanine strips break; the
    // crates4 gap is 3.1 m + 2.06), no mover serves it (no lift kind on 0c [D kind census]; the
    // 8 SSs, 2 doors, 2 ScriptPlayers are west-side [D audit §6]; ladders #164/#495 west [D];
    // the five graple_pulley points elsewhere [D #308/346/500/508/511]), and the game's own nav
    // patrols the east band (G3, 10.34) and the east ground (G4, 6.17) as SEPARATE paths with
    // no links [D the 0c nav dump]. Live checks in the report settle whether the REAL game has
    // a dynamic the parse cannot see.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [95.0, 11.3, -209.5];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(14);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let under = sim.world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 3.0, p[2]])
            .map(|(t, _)| p[1] - 3.0 * t);
        assert!(sim.bond.grounded && under.map_or(false, |y| (10.2..10.6).contains(&y)),
            "the climb stand-in must land on the band deck at (95, −209.5) (10.34 [D plc44]), got pos {p:?} under {under:?}");
    }
    eprintln!("[0c] the ground route walked + the climb stand-in — on the band at the gate steps");
    w.walk_to(Wp::new(96.0, 11.2, -209.3).r(1.0), "the gate plane");
    // The E press: USE-activated LOAD_LEVEL [M 0x1a5170 case 0x1d → 0x1b2318:111].
    let mut loaded = false;
    for _ in 0..60 {
        w.tick(nf_game::player::Input::default(), true);
        if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { loaded = true; break; }
    }
    assert!(loaded, "the gate #556 loads 0700000d (key ch 0x1c on) [D; M 0x1b2318:111]");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.level_exit, Some(0x0700_000d), "the hand-off target [D]");
    eprintln!("[0c] walked out in {} ticks; stuck {}; falls {}", w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());

    // Leg 11 (REAL hand-off): the warm RamSave carry into 0700000d — the game's own level
    // transition ([M ResetMap_LevelToLoad 0x1b2318:111 → the RamSave is the port's record of the
    // carried channels/inventory; Mission_Init 0x182270:92-121 carries record 11 across the
    // load]). On 0d, the spawn TouchOnce #436 (179.53, 11.58, −205.40, out 0x23, mask 8 [D 0d])
    // sits on the arrival start and fires ch 0x23 under Bond: objective 3 ("Advance through
    // Warehouse TR-2") completes and 0d's first objective reveals.
    let save = {
        let sim = w.scene.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w2 = Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, Some(&save)).unwrap());
    w2.wait_enabled(3000, "the 0d arrival");
    w2.walk_to(Wp::new(179.53, 11.6, -205.40).r(1.2), "onto the TR-2 arrival plate");
    w2.wait(30);
    {
        let sim = w2.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x23), "the 0d spawn TouchOnce #436 fires ch 0x23 [D #436]");
        assert_eq!(sim.drones.mission.states[11][2], 4, "objective 3 completes on the 0d arrival [M 0x1827d8:88-99]");
        assert_eq!(sim.drones.mission.states[12][0], 3, "ch 0x23 reveals 0d's first objective [L record 12 obj 0]");
    }
    eprintln!("[0c] objective 3 complete on the 0d arrival — the whole mission walked (0c legs {} ticks + 0d {} ticks)",
        w.total_ticks, w2.total_ticks);
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

/// Sister (ladder-0c): the 0c's OTHER ladder, #495 `Ladder` (−6.42, 1.0, −177.39, yaw −0.2618,
/// mesh bounds x ±0.204 / y 0..2.0 / z degenerate [D placement #495 + its map object]) — the same
/// walk-in grab + climb through the real input path, on the flat west ground (floor ~0.0 [D
/// probes]; the small 1.936 ledge sits ~0.9 east of the plane). The placement below is a
/// MEASUREMENT SPOT, not a route stand-in: the test's subject is the ladder grab/climb classes
/// ([M 0x1953f4's DEFAULT arm + 0x196118/0x196310]) at #495's own plane, not the route to the
/// ladder — the walkreal route never crosses the yard's west edge (the walkreal's route stand-ins
/// are all documented in the header), so the only honest setup is Bond on the west ground beside
/// the ladder; the walk-in and the climb after it are real input.
#[test]
fn ladder495_walk_grab_and_climb_to_the_top() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-7.6, 1.1, -176.9];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    eprintln!("[0c] the #495 stand-in — the west ground (floor ~0.0 [D probes])");
    // The walk-in: face the ladder and push (the same real-input shape as the yard leg).
    let mut climbed = false;
    for _ in 0..900 {
        let sim = w.scene.sim.as_mut().unwrap();
        if sim.bond.substate == nf_game::player::SS_CLIMB { climbed = true; break; }
        // yaw = atan2(dx, dz) — forward = (sin yaw, cos yaw) — [S] unverified (Bond::look is the port's look maths; the game's heading convention is not pinned here).
        let desired = (-6.422f32 - sim.bond.pos[0]).atan2(-177.391f32 - sim.bond.pos[2]);
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
        "walking into the #495 ladder never grabbed it — the walk substate rides the DEFAULT arm r 0.55 [M 0x1953f4 via the jump table @0x2fc0d0]");
    // Climb: hold forward; the mantle fires at climb_top = 1.0 + 1.0206 + 0.85 = 2.87 [M
    // 0x196118: `+0x84 + +0x8C + 0.85`]. The top has no landing (the ledge 1.936 is 0.9 east [D
    // probes]; the lunge falls back to the ground), so the pin is the climb passing the mesh's
    // own top (2.0 [D]) through the real path — the grab-snap lift [M 0x196118:23-31] starts him
    // at bottom + 0.02 = 1.0.
    let mut max_y = 0.0f32;
    let mut done = false;
    for _ in 0..1200 {
        let sim = w.scene.sim.as_mut().unwrap();
        max_y = max_y.max(sim.bond.pos[1]);
        if sim.bond.substate != nf_game::player::SS_CLIMB && sim.bond.pos[1] > 1.9 && max_y > 2.7 { done = true; break; }
        sim.step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    assert!(max_y > 2.7, "the #495 climb never passed its top 2.87 [M 0x196118] — max y {max_y}");
    assert!(done, "the #495 climb never dismounted past its mesh top 2.0 [D]");
    eprintln!("[0c] the #495 ladder climbed for real to y {max_y:.2}");
}
