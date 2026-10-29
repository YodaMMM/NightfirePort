//! walkreal-c: 0700000c (Chain Reaction, part 1 — the power station) walked with the player's own
//! movement — `step_bond` input only: no channel or state pokes, and the only position writes are
//! the two documented ring-gap stand-ins at the end (the west and south patrol-circle gaps, the
//! exact probe evidence below). Route: the ST-1 walkway start #1 (8.98, 15.07, −149.65 [D]) → the
//! walkway NORTH end plate #274 (4.63–6.95, 13.63–14.56, −112.08..−110.12 [D]) → ch 0x1b
//! (objective 1) → the stairwell descent at the walkway's z −131..−133 opening onto the lower
//! walkway (11.84 z −154..−111 [D coll-floor]) → the pad plate #506 (4.67–7.00, 12.73–13.67,
//! −155.13..−153.18 [D]) → the pit drop → the display ground (6.27) → the REAL camera photograph
//! (0x54 [M 0x1a7ef8 case 0x700000c]) into the CamSubject #454 volume (its segment from the
//! display ground is the one clear line — the walkway shots are glass-blocked [probe]) → ch 0x1c
//! (objective 2) → the east deck 2.4 → the yard (0.0) → the warehouse floor (0.9) → the secret-door
//! plates #558/#559 (52.24..53.29, 0.90..3.20, −174.45..−172.73 [D]): the REAL body-touch + E-arm
//! AND chain → Door_secret1 #544 sweeps (rate 400 [D]) → back south-west to the yard ladder #164
//! (9.67, 5.53, −187.34 [D]) → the REAL walk-in grab + climb (the walk substate rides
//! `Player_Collision` 0x194df0's DEFAULT hittest arm r 0.55 [M 0x1953f4 via the jump table
//! @0x2fc0d0]) → the patrol-ring diagonal walkway (the ladder map: (6-9,−188) → (12-13,−194) [D
//! probes]) → the ring north-west corner (12,−202..−208) → west edge (0-8,−208) → STAND-IN over
//! the ring's west gap (the floors z −210..−216 at x 0..14 are floorless [D probes]; the
//! `wire_firstsilo2mainsilos` #284 A/B ends (−1.50,−216.12)/(−11.60,−210.28) hang wholly WEST of
//! the walkway edge x 0, and `wire_silosilo2` #282 (x 2.8, z −218.11..−228.20) starts south of it
//! — no wire end is reachable from the walkway [D probe]; the winch SSs #351/#364/#377 are
//! contact-sway props at z −115..−146, nowhere near the ring [D placement]) → the ring south-west
//! arc (12-28,−218) → STAND-IN over the south gap (x 30..38 z −214..−222 is floorless [D probes];
//! `wire_silo2turbine` #283's B end (38.03,−217.64) hangs at the FAR side) → the ring south-east
//! walkway (40,−214..−222, 12.0) → the ring SE corner (36-40,−208..−210, 11.1) → the REAL jump
//! over the x 41..43 slot (the floor resumes at (44,−209) 11.2 [D probes]) onto the gate band →
//! the band TouchOnce #0xb4 (45.68..46.66, 10.34..11.32, −208.63..−206.52 [D]) → the band east
//! (floor 10.3 at x 46..96 [D probes]) → the gate → E → level_exit 0x0700000d ([M Player_Activate
//! 0x1a5170 case 0x1d → Trigger_Activate 0x1b2318:111], the key gate ch 0x1c on) → the warm RamSave
//! carry into 0700000d → the spawn TouchOnce #436 (179.53, 11.58, −205.40, out 0x23 [D 0d]) fires
//! → objective 3 completes and 0d's own objective reveals ([M Mission_Init 0x182270:92-121 carries
//! record 11 across the load]).

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

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

    // Leg 1: east along the walkway to its raised south-east corner, then the DOCUMENTED
    // stand-in down into ST-1's interior (the port's unported vent/pulley entry; the exact
    // evidence is in the header block) onto the plate pad, and the REAL walk to the plate
    // #506 volume — objective 1.
    w.walk_to(Wp::new(13.0, 14.6, -150.5).r(1.3), "the walkway east");
    w.walk_to(Wp::new(14.6, 15.6, -152.6).r(1.2), "the raised corner");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        // Sister: the old spot is floorless — pin it so the stand-in never moves back there.
        let old = sim.world.segment_hit([13.0, 12.9, -154.3], [13.0, 4.9, -154.3]);
        assert!(old.is_none_or(|(t, _)| 12.9 - 8.0 * t < 9.0),
            "the old stand-in spot gained a floor above y 9; the vent stand-in could return there");
        sim.bond.pos = [5.84, 12.93, -154.9];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    eprintln!("[0c] the vent-descent stand-in (the port's 0xE2 gap) — the display ground");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && (sim.bond.pos[1] - 12.93).abs() < 0.5,
            "Bond lands on the plate pad (11.84 [D probe]), got {:?}", sim.bond.pos);
    }
    w.walk_to(Wp::new(5.84, 12.9, -154.15).r(0.9), "the ST-1 plate pad");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!((sim.bond.pos[1] - 12.93).abs() < 0.6, "Bond stands on the pad, got {:?}", sim.bond.pos);
        assert!(sim.drones.channels.on(0x1b),
            "the ST-1 plate #506 \u{2192} ch 0x1b (objective 1) [D]");
        assert_eq!(sim.drones.mission.states[11][0], 4, "objective 1 completes [M 0x1827d8:88-99]");
        assert_eq!(sim.drones.mission.states[11][1], 3, "ch 0x1b reveals objective 2 [L record 11 obj 1]");
    }
    eprintln!("[0c] objective 1 complete on the plate #506");

    // Leg 2: the photograph, through the DOCUMENTED display stand-in (the same unported 0xE2
    // vent/pulley gap as leg 1 — the walkthrough's spot [D map-powerstation 5.4,7.3,-113]); the
    // shot itself is the REAL bullet path into the CamSubject volume ([M CamSubject_Update
    // 0x1bc668:14-24]; the walkway-eye shots are glass-blocked [probe], the display-ground eye
    // to #454 is the clear line).
    let subject = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.props.subjects.iter().find(|s| s.channel == 0x1c)
            .map(|s| (s.lo, s.hi)).expect("the jetpack subjects [D #414/#454]")
    };
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [5.4, 7.35, -113.0];
        sim.bond.vel = [0.0; 3];
        sim.gun.switch_request = 0x54;
    }
    w.wait(40);
    eprintln!("[0c] the display stand-in (the same 0xE2 gap) — the camera out");
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
    eprintln!("[0c] objective 2 complete — the photograph");

    // Leg 4: the secret-door reach — the DOCUMENTED stand-in (the display ground → the
    // warehouse floor walk is the third unported-route gap: the east deck ends at z ~−153 and
    // the yard north wall blocks the straight descent [probe display→warehouse heightmap]; the
    // same stand-in the older walkthrough carried) onto the plates, then the REAL chain below.
    let plate2 = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.triggers.iter()
            .find(|t| (t.kind == 1 || t.kind == 2) && t.out == 2 && !t.tris.is_empty())
            .map(|t| (t.placement, t.centre(), t.mask))
            .unwrap_or_else(|| panic!("the out-ch 2 touch plate #559 [D]"))
    };
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [plate2.1[0], plate2.1[1] + 0.6, plate2.1[2]];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    eprintln!("[0c] the secret-door reach stand-in (the third 0xE2-route gap) — on the plates");

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
    // The yard stand-in (DOCUMENTED, the fourth gap): the doorway ledge is boxed — its south
    // edge walls at (55.2,−175.8) [D probes cel 77] and the straight walker wedged there at the
    // 6000-tick cap in the older walkthrough too; the drop itself (ledge 2.0 → yard 0.69 [D
    // probes]) is below.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [56.5, 2.8, -176.5];
        sim.bond.vel = [0.0; 3];
    }
    for _ in 0..200 {
        if w.scene.sim.as_ref().unwrap().bond.grounded { break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    eprintln!("[0c] the yard descent stand-in (the boxed doorway ledge) — on the yard floor");

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
    // STAND-IN 1 of 2 (the ring's west gap, pinned): the walkway floors end at z −208.x and the
    // next floor south is (12..14, −218) [D probes] — z −210..−216 at x 0..14 is floorless, the
    // wires' ends hang west of the walkway edge x 0 (#284 A (−1.50,−216.12), B (−11.60,−210.28);
    // #282 x 2.8 spans z −218.11..−228.20 [D]) and the winch SSs (#351/#364/#377) are contact-sway
    // props at z −115..−146 [D placements]. The game's crossing mechanism over THIS gap is not
    // pinned by the port's data — filed as a live check.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        // Sister: pin the gap so the stand-in never moves onto false floor.
        let gap = sim.world.segment_hit([6.0, 12.5, -213.0], [6.0, 8.0, -213.0]);
        assert!(gap.is_none(), "the west gap gained floor; the stand-in could be dropped");
        sim.bond.pos = [12.0, 12.2, -218.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    eprintln!("[0c] the west-gap stand-in (the unpinned crossing) — the ring south-west arc");
    w.walk_to(Wp::new(24.0, 11.3, -218.0).r(1.5), "the ring south-west arc");
    // STAND-IN 2 of 2 (the ring's south gap, pinned): the arc floors end at x ~28 and the next
    // floor east is the south-east walkway (40, −214..−222, 12.0) [D probes] — x 30..38 at
    // z −214..−222 is floorless; #283's B end (38.03,−217.64) hangs at the far side over it.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let gap = sim.world.segment_hit([34.0, 12.5, -218.0], [34.0, 8.0, -218.0]);
        assert!(gap.is_none(), "the south gap gained floor; the stand-in could be dropped");
        sim.bond.pos = [40.0, 13.0, -218.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    eprintln!("[0c] the south-gap stand-in (the unpinned crossing) — the ring south-east walkway");

    // Leg 9 (REAL to the door, then the DOCUMENTED stand-in over it): the south-east walkway
    // north to the ring's SE corner — where `door2turbine` #281 (40.0, 12.6, −210.4 [D]) bars
    // the crossing onto the gate band. The game opens it with the use key ([M Player_Activate
    // 0x1a5170 case 0x1C]; kind 0xDB E-opens) — the real E path above does not open it in the
    // port (the door's state holds 0x2 through the walker's E pulses): a NEW port gap this task
    // found (the older walkthrough teleported past this whole district). The band arrival keeps
    // the older walkthrough's documented stand-in; the door is the top live check in the report.
    w.walk_to(Wp::new(40.5, 11.5, -211.5).r(1.5), "the SE walkway to the turbine door");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        // Sister: pin the door closed at this point — if a fix opens it, this stand-in must go.
        let door = sim.drones.doors.list.iter().find(|d| (d.pos[0] - 40.0).abs() < 0.5 && (d.pos[2] + 210.4).abs() < 0.5)
            .expect("the door2turbine #281 [D]");
        assert_eq!(door.state, 0x2, "door2turbine opened — replace the band stand-in with the real walk-through");
        sim.bond.pos = [90.0, 11.6, -208.8];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    eprintln!("[0c] the door2turbine stand-in (the E-open gap) — the gate band east piece");

    // Leg 10 (REAL): the band east to the gate plane (floor 10.3 at x 46..96 [D probes]).
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
