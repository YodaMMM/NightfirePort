//! map-tower2 walkreal: 07000011 "HT Tower2 A" — the office-floor ROUTE from the spawn to the
//! 07000012 exit through the real input path (walker_b2: `Sim::step_bond` inputs only; no bond.pos
//! writes after the spawn). The route the level's own data defines [D placements; M addresses cited
//! inline]:
//! 1. Cold start #146 (−0.40, 46.97, 25.83) yaw π; the spawn TouchOnce #282 fires ch 0x7E [D].
//! 2. The keypad lock #177 (0x2E, out ch 0x12) is create-unlocked on Operative [D p6 = 1; M
//!    0x1ae1f8:67-69] and `Lock_Update` case 1 (0x1ae3d8) holds ch 0x12 on — door_large_stairwell
//!    #192 (lock ch 0x12 [D]) opens on the E.
//! 3. From the waiting room through door #13 (13.82, 46.78, 7.09 [D]) into the corridor ledge —
//!    and then the level's own route to the stairwell runs WEST (see the machinery-row block
//!    below), over the west bridge, along the north strip, to #192 [D nav].
//! 4. The stairwell descent x 14-24, z −4…6, floors 47 → 16 [D — the flight cover nodes and the
//!    flight wake plates #250-253/#285], walked (the leg chain shared with
//!    walkreal_07000011_live).
//! 5. E at the Load Point #158 (0xE8, next 07000012) → the level exit [M Trigger_Activate 0x1b2318].

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{nav_route, Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_office_floor_route_reaches_the_07000012_exit() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000011.bin", false).expect("load 07000011"));
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.level, 0x0700_0011);
    }
    // The spawn touch fires (the walker's first ticks) — the route's first channel fact.
    w.wait(30);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x7e), "the spawn touch #282 fires ch 0x7E [D]");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x12),
        "the keypad's Operative unlock holds ch 0x12 [D #177 p6; M 0x1ae3d8 case 1]");
    // Across the waiting room and through door #13 (13.82, 46.78, 7.09 [D]) into the corridor
    // ledge — the doorway's south band passes in both leaf states (the closed leaf blocks only
    // z 7.0-7.2 [D probe]; the open leaf is swung aside), so the E-opened door is walked through
    // on its south half.
    let d13 = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| (d.pos[0] - 13.82).abs() < 0.5 && (d.pos[2] - 7.09).abs() < 0.5)
        .expect("door_waitingroom_corridor_right #13 [D]");
    w.walk_to(Wp::at(1.0, 46.4, 20.0, Act::None).r(1.4), "the waiting room");
    w.walk_to(Wp::at(13.82, 46.4, 8.8, Act::None).r(1.2), "door #13's office side");
    w.use_door(d13, "the waiting-room corridor door");
    w.wait(45); // the swing finishes before the doorway walk (the auto-close is 360 ticks [M Door_Update])
    // THE CORRIDOR'S DIRECT NORTHWARD LEG HAS NO ROUTE — the old stand-in teleport's cause, now
    // pinned instead of transited. The disc's own data blocks it:
    //   - the corridor machinery row stands across the ledge's north edge: a body-height ray at
    //     x 9.0/10.22/11.5/13.0/14.5 (y 47.0 and 47.6) first hits at z ≈ 5.09-5.15, and the
    //     raised machinery tops reach y ≈ 47.2-47.3 at z 4.5, x 9-11 [D collision — the static
    //     cel row corridor_machinery1-3 #18-20];
    //   - the floor beyond is the open well: no floor at z 3.5..1.5 for x 8-15 [D];
    //   - the level's own nav graph never links the two: the stair chain's first node n94
    //     (11.80, −6.29) links only eastward to n95 (15.99, −5.32) and no corridor node reaches
    //     it, while the office-floor component runs n10 (10.85, 6.97) → n11 (7.68, 6.49) west
    //     along the ledge [D nav].
    // So the game's own route from this corridor is WEST (the walk below); the direct leg's wedge
    // is reproduced through the real input path (the walker's sidestep cycles cannot pass the row):
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let hit = sim.world.segment([10.22, 47.0, 6.9], [10.22, 47.0, 4.2]);
        assert!(hit.is_some(), "a body-height ray across the corridor's north edge must hit the machinery row [D]");
        assert!(sim.world.floor_below([12.0, 48.0, 2.5], 8.0).is_none(),
            "the well strip z 1.5-4.5 has no floor at x 12 [D]");
        assert!(nav_route(sim, [13.82, 47.4, 6.0], [10.22, 47.4, -3.6]).is_none(),
            "no nav route corridor → the stairwell door [D nav: n94 links only n95]");
        assert!(nav_route(sim, [2.2, 47.4, 6.6], [10.22, 47.4, -3.6]).is_some(),
            "the ledge's west end → the stairwell door IS nav-linked (the west bridge) [D nav]");
    }
    w.lenient = true;
    let direct = w.walk_to(Wp::new(10.22, 47.4, -3.6).r(1.2), "the corridor's direct northward leg (the pinned wedge)");
    w.lenient = false;
    assert_eq!(direct, u32::MAX,
        "the direct leg must wedge on the machinery row [D collision z≈5.15; the old stand-in's cause]");
    // The game's own route west: the ledge to its west end (nav n10 → n11 [D]), the west bridge
    // south (the office floor is continuous z −2..7 only at x ~1 — the nav's own bridge crossing
    // sits at x ≈ −1.45 [D nav_route (2.2,6.6)→(10.22,−3.6)]), the north balcony, and the strip
    // east to the stairwell door's front tile (the leg chain shared with
    // walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point, which walks it green).
    w.walk_to(Wp::new(13.0, 47.4, 6.4).r(0.9), "the corridor's west leg");
    w.walk_to(Wp::new(12.0, 47.4, 6.6).r(0.9), "the ledge west");
    w.walk_to(Wp::new(2.2, 47.4, 6.6).r(0.9), "the ledge's west end (n12)");
    w.walk_to(Wp::new(1.2, 47.4, 3.0).r(0.6), "the west bridge south");
    w.walk_to(Wp::new(1.4, 47.4, -1.0).r(0.8), "onto the north balcony (n45)");
    w.walk_to(Wp::new(5.0, 47.4, -1.8).r(0.9), "the balcony east (n46/n47)");
    w.walk_to(Wp::new(9.0, 47.4, -2.2).r(0.9), "the north strip east");
    w.walk_to(Wp::new(9.6, 47.4, -4.4).r(0.8), "the strip west of the parapet");
    w.walk_to(Wp::new(10.7, 47.3, -4.1).r(0.4), "the gap's north approach");
    // The stairwell door #192 (10.22, 46.75, −5.21 [D]) and E it — the ch-0x12 unlock lets it
    // swing (the door's ch_lock field CLEARS on the rising edge [M door.rs], so the find is by
    // the disc's position).
    let di = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| (d.pos[0] - 10.22).abs() < 0.5 && (d.pos[2] + 5.21).abs() < 0.5)
        .expect("door_large_stairwell #192 [D]");
    w.walk_to(Wp::new(10.22, 47.4, -3.6).r(0.9), "the stairwell door's front tile");
    w.use_door(di, "the unlocked stairwell door");
    let st = w.scene.sim.as_ref().unwrap().drones.doors.list[di].state;
    assert!(st & 0xc != 0, "the stairwell door opens on the route's E, state {st:#x}");
    w.wait(45); // the swing finishes before the doorway walk (the auto-close is 360 ticks [M Door_Update])
    w.walk_to(Wp::new(11.0, 47.4, -5.5).r(0.9), "through the stairwell doorway");
    // The stairwell descent (floors 47 → 16, the flights at x 14-24, z −4…6, marked by the flight
    // wake plates #250-253/#285 [D]) — WALKED: the leg chain is
    // walkreal_07000011_live's (green there; the stairwell guards shoot, so the walk tops the
    // health up and pins the route, not the firefight).
    w.top_up_health = true;
    let band = |w: &Walker, lo: f32, hi: f32, what: &str| {
        let y = w.scene.sim.as_ref().unwrap().bond.pos[1];
        assert!(y > lo && y < hi, "{what}: y {y} not in ({lo}, {hi}) — the descent left the flights");
    };
    w.walk_to(Wp::new(13.0, 46.9, -6.2).r(0.9), "the top flight east");
    w.walk_to(Wp::new(19.5, 44.6, -5.2).r(1.0), "the north flight down (46.8→44.4)");
    w.walk_to(Wp::new(22.3, 43.6, -3.2).r(1.0), "the north-east landing 43.1");
    band(&w, 42.6, 44.6, "the NE landing");
    w.walk_to(Wp::new(23.0, 39.5, 3.6).r(1.0), "the east flight down (43.1→39.1)");
    w.walk_to(Wp::new(20.5, 39.5, 5.8).r(1.0), "the south-east landing 39.1");
    band(&w, 38.6, 40.5, "the SE landing");
    w.walk_to(Wp::new(15.2, 35.6, 6.0).r(1.0), "the south run down (39.1→35.2)");
    w.walk_to(Wp::new(14.8, 35.4, 5.0).r(0.9), "the south-west landing 35.2");
    band(&w, 34.6, 36.6, "the SW landing");
    w.walk_to(Wp::new(14.8, 31.4, -2.6).r(1.0), "the west flight C down (35.2→31.1)");
    band(&w, 30.4, 32.5, "flight C's bottom");
    w.walk_to(Wp::new(22.4, 31.3, -3.0).r(1.0), "the north strip 31.1 east");
    w.walk_to(Wp::new(23.0, 27.4, 3.8).r(1.0), "the east connector down (31.1→27.1)");
    band(&w, 26.4, 29.3, "connector 1's bottom");
    w.walk_to(Wp::new(15.0, 27.4, 4.8).r(1.0), "the south strip 27.1 west");
    w.walk_to(Wp::new(14.8, 23.3, -2.6).r(1.0), "the west flight B down (27.1→23.0)");
    band(&w, 22.4, 24.8, "flight B's bottom");
    w.walk_to(Wp::new(22.4, 23.2, -3.0).r(1.0), "the north strip 23.0 east");
    w.walk_to(Wp::new(23.0, 19.3, 3.8).r(1.0), "the east connector down (23.0→18.9)");
    band(&w, 18.2, 21.1, "connector 2's bottom");
    w.walk_to(Wp::new(15.0, 19.2, 4.8).r(1.0), "the south strip 18.9 west");
    w.walk_to(Wp::new(14.8, 15.4, -2.6).r(1.0), "the west flight A down (18.9→15.0)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let y = sim.bond.pos[1];
        assert!(y < 17.2, "the descent never reached the lower floor (y {y}) [D 15.0 probe + the 1.09 standing offset]");
        assert!(!sim.drones.channels.on(0x7c), "the descent never crossed the atrium plates (ch 0x7C) [D §4]");
    }
    // The Load Point #158 (0xE8, next 07000012) — walked to from the stairwell bottom (the
    // lower floor is flat from flight A's foot at x 14.8 to the trigger [D]), then E.
    w.walk_to(Wp::new(14.6, 17.0, -2.8).r(0.9), "the Load Point");
    let mut exited = false;
    for _ in 0..60 {
        w.tick(nf_game::player::Input::default(), true);
        if w.scene.sim.as_ref().unwrap().drones.level_exit == Some(0x0700_0012) { exited = true; break; }
    }
    assert!(exited, "E at the Load Point #158 exits to 07000012 [M 0x1b2318]");
    // The next archive loads (the viewer's own transition step).
    let next = nf_view::level_archive_name(0x0700_0012).to_string();
    let scene2 = nf_view::load(&iso(), &next, false).expect("07000012 loads after the exit");
    assert_eq!(scene2.sim.as_ref().expect("sim").level, 0x0700_0012);
}
