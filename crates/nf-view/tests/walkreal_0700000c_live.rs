//! walkreal-c: 0700000c (Chain Reaction, part 1 — the power station) walked with the player's own
//! movement — `step_bond` input only: no channel or state pokes; the ONLY position write left in
//! this walk is the DOCUMENTED gate-band crossing stand-in (x 55.5..67 — the crossing mechanism is
//! unpinned, live check below; the ladder495 sister test keeps its own documented west-ground
//! placement). Every blocker stand-in (the vent descent, the display eye, the secret-door reach,
//! the yard descent, the ring's west and south gaps) now runs as a real-input leg: the laser-watch
//! hinge burn, the trench drop, the pit/slab/deck/yard descents, the east-strip ledge descent and
//! the three wire crossings. Route: the ST-1
//! walkway start #1 (8.98, 15.07, −149.65 [D]) → north along the walkway (14.61 floor [D probe blk])
//! → INTO the display-case trench (x 5..6.5, z −110.5..−111.5, floor 11.84 [D probe blk: the
//! walkway's north end opens down to the lower walkway between the case's glass walls x 4.2/6.74]) →
//! the fall crosses plate #274 (4.63..6.95, 13.63..14.56, −112.08..−110.12 [D]; the game fires it
//! while the body's volume crosses and clears at rest [L 2026-10-26 hatch-2]) → ch 0x1b (objective 1)
//! → the lower walkway south (11.84/11.27 z −111..−153 [D coll-floor], passing the z −148..−150 hole
//! at x 13..14 [D probe blk2]) → the plate #506 sliver (5..6, −154, floor 11.84 [D probe blk2]) →
//! the pit drop onto the display-ground slab (6.27, x 0..14, z −152..−155.5 [D probe blk2]) → the
//! REAL camera photograph (0x54 [M 0x1a7ef8 case 0x700000c]) into the CamSubject #454 volume → ch 0x1c
//! (objective 2) → the slab's south edge drop onto the deck (2.37, x 2..8, z −156..−157.5 [D probe
//! blk3]) → the deck's south end drop onto the yard (−0.02, x 4..8, z −158 [D probe blk3]) → the yard
//! east to the warehouse slab (0.38/0.89/0.90 x 42..56 [D probe blk3]) → the secret-door plates
//! #558/#559 (52.24..53.29, 0.90..3.20, −174.45..−172.73 [D]): the REAL body-touch + E-arm AND chain
//! → Door_secret1 #544 sweeps (rate 400 [D]) → through the door onto the doorway ledge (0.90) → the
//! REAL east-south descent around the ledge's south wall (the wall z −175.8 spans x ≤ 55.5 only; x ≥
//! 56 is open, floor 0.90 → 0.69 [D probe blk rays]) → the yard ladder #164 (9.67, 5.53, −187.34 [D])
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
//! its true end x 55.0 [D probe door4 fine map] → STAND-IN over the gate-band crossing (x 55.5..67
//! at band height is void in the disc's own collision: the pit floor 6.17 spans x 55..63.5, the
//! 7.19 shelf x 64..64.5, then void to the resume x 67.5 [D probe door4 pit map]; the 10.85 strip
//! z −206.1..−205.2 has a 6.5-unit gap x 53.5..60 [D probe door4 0.25 map]; NO SS mover touches
//! the box, no wire serves it, no ladder exists in the east half, no drone moves there in 300
//! ticks, and the type-2 zipline y 17.85 has no floor at either end [D probes door3/5/6]; the
//! game's own nav path17 links (49.69, 10.74, −207.29)↔(63.06, 10.74, −207.54) ACROSS the void
//! but the per-path A* rejects it [D probe door4]) → the resumed band (10.34 x 67.5+ [D]) → the
//! gate → E → level_exit 0x0700000d ([M Player_Activate
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

    // Leg 2 (REAL): the lower walkway south to the #506 sliver, the PIT DROP onto the
    // display-ground slab (6.27, x 0..14, z −152..−155.5 [D probe blk2's pit map]), the slab
    // north to the SUBJECT's clear-line eye (the walkway shots are glass-blocked; the
    // display-ground eye to #454 is the clear line [D map-powerstation 5.4,7.3,-113 + probe]),
    // and the photograph through the REAL camera switch and bullet path [M CamSubject_Update
    // 0x1bc668:14-24]. The 0xE2 display stand-in is gone.
    w.walk_to(Wp::new(5.78, 12.9, -125.0).r(1.5), "the lower walkway south 1");
    w.walk_to(Wp::new(5.8, 12.9, -146.0).r(1.5), "the lower walkway south 2 (the z −148..−150 hole is at x 13..14 [D blk2])");
    w.walk_to(Wp::new(5.84, 12.9, -153.6).r(0.9), "the #506 sliver");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && (sim.bond.pos[1] - 12.93).abs() < 0.6,
            "Bond stands on the pad sliver (11.84 [D probe blk2]), got {:?}", sim.bond.pos);
        assert!(sim.drones.channels.on(0x1b), "ch 0x1b held on — objective 1 came from the #274 drop [D #506 out 0x1b]");
    }
    w.walk_to(Wp::new(5.8, 7.4, -155.2).r(1.0), "the pit drop onto the display ground");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && (sim.bond.pos[1] - 7.35).abs() < 0.6,
            "Bond lands on the display-ground slab (6.27 [D probe blk2]), got {:?}", sim.bond.pos);
    }
    eprintln!("[0c] the pit dropped onto the display ground at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    let subject = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.props.subjects.iter().find(|s| s.channel == 0x1c)
            .map(|s| (s.lo, s.hi)).expect("the jetpack subjects [D #414/#454]")
    };
    w.walk_to(Wp::new(5.4, 7.4, -130.0).r(1.5), "the display ground north 1");
    w.walk_to(Wp::new(5.4, 7.4, -118.0).r(1.2), "the display ground north 2");
    w.walk_to(Wp::new(5.4, 7.35, -113.0).r(0.8), "the clear-line eye [D map-powerstation]");
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
    eprintln!("[0c] objective 2 complete — the photograph from the walked-in display ground");

    // Leg 4 (REAL, blocker 3 replaced): the slab south to its south edge, the deck drop, the
    // yard drop, and the yard east to the secret-door plates — the descent the disc's own
    // collision carries [D probe blk3: the slab (6.27) drops at z ~−155.7 onto the deck
    // (2.37, x 2..8, z −156..−157.5), which drops at z ~−157.5 onto the yard (−0.02, x 4..8,
    // z −158), running east to the warehouse slab (0.38/0.89/0.90, x 42..56)]. The
    // teleport-to-the-plates stand-in is gone. (The game's ALTERNATIVE connection — the annex
    // vent, lock #180 → SS lid → the hole down to 0.90 [D locks/keys; M Lock_Update 0x1ae3d8 +
    // drone.rs:14004's real bullet pass] — is not needed on this route.)
    w.walk_to(Wp::new(5.0, 7.4, -153.2).r(1.2), "the slab back south");
    w.walk_to(Wp::new(4.5, 3.5, -156.4).r(1.0), "the deck drop (2.37 [D probe blk3])");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && (sim.bond.pos[1] - 3.45).abs() < 0.6,
            "Bond lands on the deck (2.37 [D probe blk3]), got {:?}", sim.bond.pos);
    }
    w.walk_to(Wp::new(5.0, 1.1, -158.4).r(1.0), "the yard drop (−0.02 [D probe blk3])");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && (sim.bond.pos[1] - 1.06).abs() < 0.6,
            "Bond lands on the yard (−0.02 [D probe blk3]), got {:?}", sim.bond.pos);
    }
    eprintln!("[0c] the display-ground descents landed on the yard at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    w.walk_to(Wp::new(20.0, 1.4, -164.0).r(2.0), "the yard east 1");
    w.walk_to(Wp::new(40.0, 1.6, -171.0).r(2.0), "the yard east 2");
    w.walk_to(Wp::new(52.7, 2.0, -173.6).r(1.0), "onto the secret-door plates");
    let plate2 = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.triggers.iter()
            .find(|t| (t.kind == 1 || t.kind == 2) && t.out == 2 && !t.tris.is_empty())
            .map(|t| (t.placement, t.centre(), t.mask))
            .unwrap_or_else(|| panic!("the out-ch 2 touch plate #559 [D]"))
    };
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(walker::dxz(sim.bond.pos, plate2.1) < 2.0,
            "Bond walks onto the plate #559 volume [D #559 centre], got {:?} vs centre {:?}", sim.bond.pos, plate2.1);
    }
    eprintln!("[0c] walked onto the plates ({:.2},{:.2},{:.2}) — no reach stand-in", plate2.1[0], plate2.1[1], plate2.1[2]);

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
    // Walk through the spent door for real (the doorway is east of the plates — Bond crosses
    // the door slab's plane x ≈ 51.9 [D #544]) and drop back onto the warehouse floor.
    w.walk_to(Wp::new(55.0, 2.1, -175.2).r(0.9), "through the secret door");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.pos[0] > 53.5, "Bond crossed the doorway for real (x {:?})", sim.bond.pos[0]);
    }
    // The yard descent (REAL, blocker 5 replaced): the ledge's east strip — the south wall
    // (z −175.80, y 1.5..2.9+) spans x ≤ 55.5 only; the north-south rays at x 56.5/57.5 hit
    // nothing at y 1.5..2.9, and the floor map shows the ledge (0.90) stepping down to the
    // yard (0.69) across x ~55.75 for z −174..−175.5 [D probe blk floors+rays]. The real
    // descent is east-then-south around the wall's east end (the old [56.5, 2.8, −176.5]
    // teleport is gone).
    w.walk_to(Wp::new(55.8, 2.0, -174.3).r(0.8), "the ledge east strip");
    w.walk_to(Wp::new(56.8, 1.8, -175.0).r(0.8), "around the wall's east end");
    w.walk_to(Wp::new(57.0, 1.8, -177.5).r(1.0), "the descent onto the yard");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && sim.bond.pos[1] < 2.2,
            "Bond descends to the yard floor (0.69 [D probe blk floors]) around the wall's east end, got {:?}", sim.bond.pos);
    }
    eprintln!("[0c] the east-strip descent reached the yard at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);

    // Leg 6 (REAL): south-west across the yard to the ladder base.
    w.walk_to(Wp::new(56.0, 1.4, -182.0).r(1.5), "the yard south");
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
    // [M Player_Wire]; drop inside 0.9 of B onto the SE corner.
    let dropped = {
        let mut dropped = false;
        for _ in 0..2400u32 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.substate != nf_game::player::SS_WIRE { dropped = true; break; }
            let d = ((sim.bond.pos[0] - 35.89).powi(2) + (sim.bond.pos[2] + 209.44).powi(2)).sqrt();
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

    // Leg 10 (REAL): the band east to its true end. The disc's band floor 10.34 runs x 44..55.0
    // (z −208.5..−206.5) — the -2 session's "x 44..53" under-measured it; x 55.5..67 at band
    // height is VOID, floored only by the 6.17 pit (x 55..63.5) and the 7.19 shelf (x 64..64.5)
    // [D probe walkreal_probe0c_door4_live's pit map]. The 10.85 strip (z −206.1..−205.2) spans
    // x 47.5..53.5 and ~60..70 with a 6.5-unit gap x 53.5..60 [D probe door4's 0.25 map] — the
    // -2 session's measured run-jump carry is 4.42 units (the slot jump), short of it. Nothing
    // bridges it: no SS mover touches the box (the level's only movers are the case lid and the
    // ch-0x54 vent), no wire serves it, no ladder exists in the east half, no drone moves in the
    // box across 300 ticks, the zipline (type 2, y 17.85) has no floor at either end, and the
    // 15.73 plane at z −206.0 (x 42..76) has no found access [D probes door3/5/6]. The game's
    // own nav graph links band-height nodes ACROSS the void (path17 n0 (63.06, 10.74, −207.54)
    // ↔ n1 (49.69, 10.74, −207.29) [D probe door4]) yet the per-path A* rejects it — the
    // crossing mechanism is UNPINNED (live check 1 below); the walk uses the third stand-in.
    w.walk_to(Wp::new(50.0, 11.0, -207.6).r(1.5), "the band east 1");
    w.walk_to(Wp::new(54.5, 11.0, -207.6).r(0.8), "the band east to its true end x 55.0 [D probe door4]");
    // STAND-IN 3 of 3 (the gate-band crossing, pinned): the evidence block above. The game's
    // crossing mechanism over x 55.5..67 at band height is not pinned by the port's data —
    // filed as a live check; the stand-in drops Bond onto the resumed band (10.34 x 67.5+ [D]).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        // Sister: pin the void so the stand-in never moves onto false floor: the pit (floor
        // 6.17 [D probe door4]) must stay BELOW band height at (59, −207.5), and the resume's
        // west edge must keep its floor.
        let gap = sim.world.segment_hit([59.0, 12.0, -207.5], [59.0, 8.0, -207.5]);
        assert!(gap.is_none(), "the band crossing gained floor at band height; the stand-in could be dropped");
        sim.bond.pos = [68.0, 11.4, -207.9];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(14);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let under = sim.world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 3.0, p[2]])
            .map(|(t, _)| p[1] - 3.0 * t);
        assert!(sim.bond.grounded && under.map_or(false, |y| (10.2..10.6).contains(&y)),
            "the crossing stand-in must land on the resumed band (10.34 x 67.5+ [D probe door4]), got pos {p:?} under {under:?}");
    }
    eprintln!("[0c] the band-crossing stand-in (the unpinned crossing) — the resumed band east");
    w.walk_to(Wp::new(70.0, 11.2, -208.0).r(2.5), "the band east");
    w.walk_to(Wp::new(90.0, 11.2, -208.8).r(1.2), "the gate approach");
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
/// probes]; the small 1.936 ledge sits ~0.9 east of the plane). The stand-in: Bond placed on the
/// west ground — the ported route never crosses the yard's west edge (the walkreal's route stand-
/// ins are all documented in the header) — the walk and the climb after it are real input.
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
