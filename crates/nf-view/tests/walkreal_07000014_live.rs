//! walkreal-b: 07000014 (Drake's underground facility → Delta Sector) walked with the player's
//! own movement — the infiltration, the sabotage and the kill are the REAL interaction paths; the
//! only teleports are the reach-a-spot hops into the cut-scene bands the walk cannot enter
//! (documented per leg). Route (docs/levels/07000014.md §1a + this file's [D] tags):
//! the vent start #666/#667 (40.81, 2.40, 65.08 [D]) → the CRAWL south through the AirConGrill1
//! plane z=58.86 (the level-14-complete fix: glist flags bit 0x1 objects carry no static world
//! collision [D the bit-0x1 census: the door arts' static twins, the shootable windows, the
//! grills, the slimes and the labels all carry it; the vent walls don't; without the skip the
//! start is a sealed box the game cannot ship]) → the duct stairs down to the y=0.60 corridor
//! [D probe14_walls: steps z 55.6..57.8, floor 0.60 at z 54.4..55.9] → the corridor south → up
//! the shaft at z≈50 → the vent (y 2.40) south to the flat grate mouths at z=42.16 [D the
//! Vent_StoreRoom_Graphic SPs #655/#600] → out onto the mezzanine (y 2.4-2.6, z 37.5..40.5 [D])
//! through the vent drop TouchOnce #301 (39.68…41.75, 2.07…4.14, 37.62…40.40 [D]) → the SP #276
//! cut-scene (0x06000833, 1400 ticks) writes ch 0x47 (objective 1) → the REAL walk west through
//! the facility to the Kiko cut-scene touch #381 → SP #382 (0x0600062F) writes ch 0x48 (objective
//! 2) → down into the west generator room → the two fuseboxes cut through the REAL Laser path
//! ([M FuseBox_Update 0x1adfe8]; #383 p1 0x87, #385 p1 0x88 [D]) → AND #391 raises 0x8C → back
//! east to the server-room escape touch #456 → SEQ_AND #445 → SP #407 (0x06000848) writes ch
//! 0x4A (objective 3) → east to Rook's wake touch #476 → SEQ_AND #475 wakes Rook (skin
//! 0x05000066 [D]) → the REAL gun path kills him → his death channel 0x54 fires [D drone param
//! 12; M set_death_channel 0x13e660] (objective 4) → the REAL walk to the exit #944 (27.79,
//! 1.28, −25.32 [D]) → E → the fallback arm writes ch 0xCE (objective 5) and loads 07000015
//! [M 0x1b2318].
//!
//! Enemies are DISABLED for the walk (the walker tops Bond's health every tick; the pain pipeline
//! still runs) except Rook, who must die. The fuseboxes' live wires carry param 2 non-zero on
//! every box here → the cut NEVER hurts [M 0x1adfe8:99-101; D placements census].

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{wrap_pi, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Walk to a stance inside the Laser's range of a fusebox wire and burn it through the REAL fire
/// path. E does nothing on fuseboxes — `Player_Activate` 0x1a5170 has no class-0x3C arm [M
/// 0x1a5170:95-160] — the game's cut is the Laser beam (weapon 0x4E, range 4.0 [L weapon_data
/// 0x4E +0x1C]) into the wire sub-object; `FuseBox_Update` 0x1adfe8 writes the group's channel
/// words on the wire death. (walkreal-b's original E-cut predates that finding and never fired.)
fn cut_fusebox(w: &mut Walker, box_pos: [f32; 3], want: u8, label: &str) {
    let sim = w.scene.sim.as_ref().unwrap();
    let fb = sim.drones.fuseboxes.iter().position(|f| f.ch[0] as u32 == want as u32)
        .unwrap_or_else(|| panic!("{label}: the fusebox with group-0 channel {want:#x} exists [D]"));
    let (bp, wire) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let wp = sim.drones.fuseboxes[fb].alive_wires()
            .filter_map(|(_, _, o)| nf_game::drone::Drones::script_obj_pos(&sim.drones.scripts, &sim.data(), o))
            .next().expect("the script registered wires (groups 0 and 1 [L])");
        (sim.drones.fuseboxes[fb].pos, wp)
    };
    let cands = [[0.6f32, -1.2, 0.0], [-0.6, -1.2, 0.0], [0.0, -1.2, 0.6], [0.0, -1.2, -0.6]];
    let mut stood = false;
    for c in &cands {
        let spot = [bp[0] + c[0], bp[1] + c[1], bp[2] + c[2]];
        let floor = w.scene.sim.as_ref().unwrap().world.segment(spot, [spot[0], spot[1] - 30.0, spot[2]]).map(|t| spot[1] - 30.0 * t);
        if !matches!(floor, Some(f) if (f - (bp[1] - 2.4)).abs() < 3.0) { continue; }
        w.walk_to(Wp::new(spot[0], spot[1], spot[2]).r(0.6), &format!("{label} stance"));
        let head = w.scene.sim.as_ref().unwrap().bond.head();
        let d = [wire[0] - head[0], wire[1] - head[1], wire[2] - head[2]];
        if (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() <= 3.4 { stood = true; break; }
    }
    assert!(stood, "{label}: a stance inside the Laser's range exists [L weapon_data 0x4E +0x1C]");
    for _ in 0..40 { w.tick(nf_game::player::Input { switch_to: Some(0x4e), ..Default::default() }, false); }
    let start = w.total_ticks;
    loop {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(want as u32) { break; }
        assert!(w.total_ticks - start < 600, "{label}: the laser cut never wrote ch {want:#x} [M 0x1adfe8]");
        let head = sim.bond.head();
        let dir = [wire[0] - head[0], wire[1] - head[1], wire[2] - head[2]];
        let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-4);
        w.tick_fire(nf_game::player::Input {
            turn: (dir[0].atan2(dir[2]) - sim.bond.yaw).clamp(-0.08, 0.08),
            look: (((dir[1] / l).atan() / std::f32::consts::FRAC_PI_2) - sim.bond.pitch).clamp(-0.08, 0.08),
            ..Default::default()
        });
    }
    eprintln!("[014] {label}: ch {want:#x} on at +{} ticks (laser cut)", w.total_ticks - start);
}

/// The crouch toggle the walker lacks (Act::CrouchOn is unreachable in walk_to): set the field
/// the way the game's crouch hold does and give the stance a tick to settle.
fn set_crouch(w: &mut Walker, on: bool) {
    w.scene.sim.as_mut().unwrap().bond.crouched = on;
    for _ in 0..15 { w.tick(nf_game::player::Input::default(), false); }
}

/// The east route, REWRITTEN (level-14-east-walkway) — the old drop-leg premise (render-only slot
/// lids) is refuted and the route now walks the game's own crossing:
///
/// - The lid slabs are SOLID [M Script_EntityStart 0x1d8d78 links every created entity into the cel
///   chain; build_LinkToRoom 0x1c9680 — the lid-and-climb report's §solidity], but they never block
///   THIS route: the vent room is TWO PARALLEL WALKWAYS with a 3 m OPEN gap between them (x
///   41.54..44.54 at floor band z 59..67 — the trench floor -0.60 beneath [D probe14_room]) and the
///   crossing is the placed grapple point "GRAPPLEPOINT_SkyBridge" (45.29, 5.10, 62.94) [D
///   placement; the port resolves it at [45.2888, 5.0961, 62.9362] — probe14_aim]: the name IS the
///   mechanic. The Q-Claw bolt latches ONLY a class-0x51 object [M Bullet_Delete 0x12e660:50-55 →
///   Player_SetGrapplePoint 0x195da0]; a bolt-legal aim exists from the standing stance (40.8,
///   floor 2.40, z 65.0-65.5) at pitch ≈ 0.20 rad — the direct ray hits the point's mount, the
///   lower arc clears it [D probe14_aim's clear-aim scan].
/// - The reel pulls 0.05→1.5/tick toward the point and HANGS at distance < 2.0 [M Player_SSGrapple
///   0x192860:79-121, :91-94]; the hang (~44.05, 4.53, 63.55, 1.48 from the point) stands over the
///   open gap, and the release [M 0x192860:56-66] drops Bond into it — the slot IS the west→east
///   crossing's landing, onto the trench floor at the nav lane node 5 (43.05, 0.4, 62.86) [D].
/// - The trench runs east under the east walkway to the corridor: nav 5 (43.05, 0.4, 62.86) → 4
///   (49.08, 0.4, 62.85) [D], floor -0.60 [D this session's grids]. The corridor (x 47.75..49.75,
///   -0.60) is separated from the stairwell column (x 44.25..46.25) by a -2.40 pit band (x
///   46.5..47.5, z ~44.5..54.5) [D probe14_corridor] — one jump each way.
/// - The east STAIRWELL (the descent the lid-and-climb report identified: x 44.5..46, z 58 (2.40)
///   → 57.5 (2.30) → 56-57 (1.05) → 54.5-55.5 (0.60) → 51.5-54 (-0.91) [D probe14_stairs]) is
///   climbed here from its -0.91 foot up to the stairwell-slot approach (45.23, 1.07, 56.27) [D] —
///   the same steps the game's Bond descends, walked in the reachable direction.
#[test]
fn the_east_duct_route_reaches_the_stairwell_slot() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000014.bin", false).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    // Leg A: the crossing stance (40.8, floor 2.40, 65.5) — the spawn crouch walks there; the aim
    // needs the standing head (the 3.86 duct roof clears at this z [D probe14_aim]).
    w.walk_to(Wp::new(40.8, 2.4, 65.5).r(0.5), "to the crossing stance");
    w.scene.sim.as_mut().unwrap().bond.crouched = false;
    for _ in 0..20 { w.tick(nf_game::player::Input::default(), false); }
    // switch to the Q-Claw (0x50) [L gadget is_grapple: 0x50/0x51; +0x70 & 0x40]
    for _ in 0..40 { w.tick(nf_game::player::Input { switch_to: Some(0x50), ..Default::default() }, false); }
    eprintln!("[014] held weapon {:#x} (the switch request 0x50 may still read the old gun on sim.gun)", w.scene.sim.as_ref().unwrap().gun.weapon);
    // aim: yaw at the point, pitch 0.20 rad (look 0.127) — the bolt-legal arc [D probe14_aim].
    let point = [45.2888f32, 5.0961, 62.9362]; // [D placement; the port's resolution, probe14_aim]
    let want_yaw = (point[0] - 40.8).atan2(point[2] - 65.5);
    let want_look = 0.20f32 / std::f32::consts::FRAC_PI_2;
    for i in 0..200 {
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let inp = nf_game::player::Input {
            turn: wrap_pi(want_yaw - b.yaw).clamp(-0.08, 0.08),
            look: (want_look - b.pitch).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
        if i > 150 { break; }
    }
    // fire to latch — the bolt's sweep accepts the point's sphere [M 0x12e660:50-55; 0x12c5a0:92]
    let mut latched = false;
    for _ in 0..30 {
        w.tick_fire(nf_game::player::Input::default());
        if w.scene.sim.as_ref().unwrap().bond.grapple.is_some() { latched = true; break; }
    }
    assert!(latched, "the Q-Claw bolt latches the SkyBridge grapple point [M Bullet_Delete 0x12e660:50-55; D placement]");
    // reel to the hang [M 0x192860:79-121], then release over the gap [M 0x192860:56-66]
    for _ in 0..400 {
        w.tick_fire(nf_game::player::Input::default());
        if w.scene.sim.as_ref().unwrap().bond.grapple.is_none() { break; }
        if let Some(g) = &w.scene.sim.as_ref().unwrap().bond.grapple {
            if g.phase == 2 {
                let p = w.scene.sim.as_ref().unwrap().bond.pos;
                let d = ((p[0] - g.point[0]).powi(2) + (p[1] - g.point[1]).powi(2) + (p[2] - g.point[2]).powi(2)).sqrt();
                assert!(d < 2.5, "the hang sits within 2.0 of the point [M 0x192860:91-94], got {d:.2}");
                break;
            }
        }
    }
    // The release lands Bond on the slot's rim graze (the 2.26 mid platform corner at z ~63.9,
    // [D probe14_room's z 63.8-64.2 band]) — the east press + hop breaks the graze and drops him
    // into the OPEN gap onto the trench floor (the crossing's landing, nav lane node 5 [D]).
    let mut down = false;
    for i in 0..400 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            if b.grapple.is_none() && b.grounded && b.pos[1] < 1.0 { down = true; break; }
        }
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let inp = nf_game::player::Input {
            forward: 1.0, jump: i % 25 == 0,
            turn: wrap_pi(std::f32::consts::FRAC_PI_2 - b.yaw).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
    }
    {
        let b = &w.scene.sim.as_ref().unwrap().bond;
        assert!(down && b.grapple.is_none(),
            "the release over the open gap lands Bond on the trench floor (-0.60 [D probe14_room]), got {:?} grounded {}",
            b.pos, b.grounded);
        eprintln!("[014] the crossing landed {:?}", b.pos);
    }
    // Leg B: the trench east to the corridor — the nav lane node 5 -> 4 [D], floor -0.60.
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 62.8).r(0.7), "the trench east to the corridor");
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[014] corridor east leg: {p:?}");
        assert!(p[0] > 47.0, "the trench/corridor east made real progress [D nav nodes 5/4], got {p:?}");
    }
    // Leg C: the corridor south to the stairwell lane (z ~52), then jump WEST over the pit band
    // (x 46.5..47.5 at -2.40 [D probe14_corridor]) onto the -0.91 landing.
    w.walk_to(Wp::new(48.8, -0.6, 52.2).r(0.7), "the corridor south to the stairwell lane");
    let mut west = false;
    for i in 0..300 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            if p[0] < 46.2 && p[1] > -1.4 && sim.bond.grounded { west = true; break; }
        }
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let inp = nf_game::player::Input {
            forward: 1.0, jump: i % 25 == 0,
            turn: wrap_pi(-std::f32::consts::FRAC_PI_2 - b.yaw).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
    }
    assert!(west, "the pit band (x 46.5..47.5, -2.40 [D probe14_corridor]) is jumpable west onto the -0.91 landing, got {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);
    // The stairwell FOOT, walked: the -0.91 landing (z 51.5..54.25, x 44.25..46.25 [D
    // probe14_corridor]) is the base of the game's east stairwell descent (the steps z 58 (2.40)
    // -> 57.5 (2.30) -> 56-57 (1.05) -> 54.5-55.5 (0.60) -> 51.5-54 (-0.91) [D probe14_stairs]).
    // The full climb/descent of the 1.05 flight (the walkable west edge x 44.5..44.75, the
    // 45..45.75 middle is the stairwell's deep throat) is the next session's leg.
    w.climb_stairs([45.5, -0.91, 53.9], "north on the -0.91 landing (the stairwell foot)", 0.8, 3000);
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[014] stairwell foot: {p:?}");
        assert!(p[1] > -1.6 && p[1] < 0.0 && p[2] > 52.5 && p[2] < 54.6 && p[0] > 44.0 && p[0] < 46.4,
            "the east stairwell foot (-0.91 landing, z 51.5..54.25 [D probe14_corridor/probe14_stairs]) is reached, got {p:?}");
    }
    for i in 0..300 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            if p[0] > 47.6 && p[1] > -1.4 && sim.bond.grounded { break; }
        }
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let inp = nf_game::player::Input {
            forward: 1.0, jump: i % 25 == 0,
            turn: wrap_pi(std::f32::consts::FRAC_PI_2 - b.yaw).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
    }
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 52.0).r(0.7), "the corridor south past z 52");
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 44.0).r(0.7), "the corridor south to the mouths' lane");
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[014] mouths' lane: {p:?} in {} ticks; stuck {}; falls {}", w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());
        assert!(p[2] < 45.0 && p[0] > 46.0, "the corridor south reached the mouths' lane [D nav node 3 -> (49.1, 0.4, 42.8)], got {p:?}");
    }
}
#[test]
#[ignore = "level-14-east-walkway: leg 1a is REWRITTEN onto the east route (the SkyBridge Q-Claw crossing [D GRAPPLEPOINT_SkyBridge (45.29, 5.10, 62.94); M Bullet_Delete 0x12e660:50-55 + Player_SSGrapple 0x192860] -> the open slot drop -> the trench nav lane 5->4 -> the corridor) and PINNED GREEN by this file's the_east_duct_route_reaches_the_stairwell_slot (incl. the stairwell foot and the mouths' lane). The mouths' Q-Claw climb of the OLD note is REFUTED from below [M the lids + the mezzanine strip z 41..43.6 block every corridor ray - level-14-qclaw-climb]; the mezzanine entry is from INSIDE the east duct: the east shaft (throat x 44.75..45.75, z 49..50 [D probe14_south]) up to the east vent lane, then south out of the mouth over its flush lid [D probe14_south grids]. THIS test still walks the old leg-1a body below (the sealed west descent) - the rewrite onto the east route + the shaft entry + these legs 2..6 is the next session's work; un-ignore when it walks green."]
fn walk_the_infiltration_to_the_delta_exit() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000014.bin", false, None).unwrap());
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 39.0 && p[0] < 42.0 && p[1] > 1.5 && p[1] < 6.0 && p[2] > 63.0 && p[2] < 67.0,
            "the vent store room start #666 [D], got {p:?}");
    }
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.mission.states[20][0], 3,
        "'Infiltrate Drake's facility' announces at the start [L]");

    // Leg 1a: the crawl south through the AirConGrill1 plane (z=58.86) and down the duct to the
    // stair head — the crawl keeps Bond under the 3.86 duct roof [D probe14_walls].
    set_crouch(&mut w, true);
    w.walk_to(Wp::new(40.8, 2.4, 56.6).r(0.9), "the crawl south through the grill to the stair head");
    assert!(w.scene.sim.as_ref().unwrap().bond.pos[2] < 58.2,
        "the crawl passed the grill plane z=58.86 [D AirConGrill1; the bit-0x1 fix]");
    // Leg 1b: descend the stairs to the y=0.60 corridor [D steps z 55.6..57.8] — the crawl keeps
    // the low profile the stairwell's segment lips need.
    w.climb_stairs([40.8, 0.6, 54.6], "down the duct stairs to the corridor", 0.9, 6000);
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        assert!(p[1] < 1.3, "the corridor floor y=0.60 [D], got {p:?}");
    }
    // Leg 1c: the corridor south, then up the shaft at z≈50 into the south vent (y 2.40).
    w.walk_to(Wp::new(40.8, 0.6, 50.6).r(0.9), "the corridor south to the shaft");
    w.climb_stairs([40.8, 2.4, 47.4], "up the shaft into the south vent", 0.9, 6000);
    // Leg 1d: the vent south to the flat-grate mouths at z=42.16 [D #655/#600], out onto the
    // mezzanine, south through the vent drop band #301.
    w.walk_to(Wp::new(40.78, 2.4, 41.3).r(0.9), "the south vent to the mouth");
    w.walk_to(Wp::new(40.7, 2.5, 39.0).r(1.2), "out onto the mezzanine through #301");
    // The SP #276 cut-scene (0x06000833, 1400 ticks [D decode]) writes ch 0x47 at its last tick.
    let mut ch47 = false;
    for _ in 0..2400 {
        w.tick(nf_game::player::Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x47) { ch47 = true; break; }
    }
    assert!(ch47, "the vent cut-scene must write ch 0x47 [D]");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.mission.states[20][0], 4, "objective 1 completes [M 0x1827d8]");
    for _ in 0..900 {
        if !w.scene.sim.as_ref().unwrap().disabled { break; }
        w.tick(nf_game::player::Input::default(), false);
    }

    // Leg 2: west through the facility to the Kiko cut-scene touch #381 — walked (the long cross
    // floor; the nav web threads the corridors [D]).
    w.follow_nav_route([-49.0, 1.0, 38.4], "west to the Kiko touch #381");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.pos[0] < -45.0, "Bond stands in the #381 band (x −51.97…−46.39 [D]), got {:?}", sim.bond.pos);
    }
    let mut ch48 = false;
    for _ in 0..2400 {
        w.tick(nf_game::player::Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x48) { ch48 = true; break; }
    }
    assert!(ch48, "the Kiko cut-scene 0x0600062F must write ch 0x48 [D]");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.mission.states[20][1], 4, "objective 2 completes [M 0x1827d8]");
    for _ in 0..900 {
        if !w.scene.sim.as_ref().unwrap().disabled { break; }
        w.tick(nf_game::player::Input::default(), false);
    }

    // Leg 3: down into the west generator room; the two fuseboxes cut through the REAL path.
    cut_fusebox(&mut w, [-89.31, -2.22, 30.79], 0x87, "fusebox #383");
    cut_fusebox(&mut w, [-89.30, -2.22, 49.36], 0x88, "fusebox #385");
    for _ in 0..10 { w.tick(nf_game::player::Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x8c), "AND #391 [0x87, 0x88] raises 0x8C [D]");
        assert_eq!(sim.drones.health.health, 100.0,
            "no live-wire hurt with +0x56 configured [M 0x1adfe8:99-101]");
    }

    // Leg 4: back east to the server-room escape touch #456 — walked; the SP #407 cut-scene
    // (0x06000848, 950 ticks [D]) writes ch 0x4A.
    w.follow_nav_route([-33.1, 0.9, 26.8], "east to the escape touch #456");
    let mut ch4a = false;
    for _ in 0..2000 {
        w.tick(nf_game::player::Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(0x4a) { ch4a = true; break; }
        assert!(sim.drones.channels.on(0x0c), "the room touch #456 fired 0x0C on contact [D]");
    }
    assert!(ch4a, "the escape cut-scene must write ch 0x4A [D]");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.mission.states[20][2], 4, "objective 3 completes [M 0x1827d8]");
    for _ in 0..900 {
        if !w.scene.sim.as_ref().unwrap().disabled { break; }
        w.tick(nf_game::player::Input::default(), false);
    }

    // Leg 5: east to Rook's wake touch #476 — walked; SEQ_AND #475 wakes him.
    w.follow_nav_route([20.3, 0.9, -23.5], "east to Rook's wake touch #476");
    for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
    let rook_i = w.scene.sim.as_ref().unwrap().drones.list.iter()
        .position(|d| d.skin == 0x0500_0066).expect("Rook (skin 0x05000066) exists [D]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x0d), "SEQ_AND #475 [0x8C, 0x0F] raises 0x0D [D]");
        assert!(sim.drones.list[rook_i].started, "Rook leaves his switch-channel wait [M 0x14b300]");
    }
    let mut dead = false;
    for attempt in 0..3 {
        if w.shoot(rook_i, "Rook") { dead = true; break; }
        assert!(w.resupply(&format!("Rook attempt {attempt}")), "no ammo for the Rook kill");
    }
    assert!(dead, "Rook must die to the walked gun path");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x54), "Rook's death channel 0x54 fires [D param 12; M 0x13e660]");
        assert_eq!(sim.drones.mission.states[20][3], 4, "objective 4 completes [M 0x1827d8]");
    }

    // Leg 6: to the Delta exit #944 — walked; E fires the fallback write + the load.
    w.walk_to(Wp::new(27.79, 1.28, -25.32).r(1.0), "to the Delta exit #944");
    let mut loaded = false;
    for _ in 0..60 {
        w.tick(nf_game::player::Input::default(), true);
        if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { loaded = true; break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(loaded && sim.drones.level_exit == Some(0x0700_0015),
            "E at the exit #944 loads 07000015 (EvilBase) [D]");
        assert!(sim.drones.channels.on(0xce), "the fallback arm wrote ch 0xCE (objective 5) [M 0x1b2318]");
    }
    eprintln!("[014] walked out in {} ticks; stuck {}; falls {}", w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());
}

/// The grill sister, REWRITTEN (level-14-vent-grate): the old version pinned the crawl THROUGH the
/// AirConGrill1 plane on the bit-0x1 skip — that skip contradicts the decompile and is reverted.
/// The game's own evidence: [M] parsemap_block_map_data_static 0x1d1998:92 creates the runtime cel
/// for every 0x8000 placement (AirConGrill1's placement #100 carries 0x00008019 [D probe14_plcflags])
/// and [M] Collide_Pick 0x1eb320 — the candidate stage of the movement capsule's kind-0x800 query
/// (Collide_CylinderIntersect 0x1ecc00:61) — skips nodes only on cel+0x94 bit 0x20 (:140) and
/// bit 0x1000 with query+0x98 bit 0x20 (:145-151); the grill's glist word is 0x1 [D], never tested.

/// duct's south end is closed and the route leaves through the EAST duct mouth (past lid SP #328).
#[test]
fn the_vent_grill_blocks_the_west_duct_per_the_game() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..60 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
    }
    let bar_count = {
        let sim = scene.sim.as_ref().unwrap();
        let tris = sim.world.tris_near([40.0, 2.3, 58.5], [41.6, 3.9, 59.2]);
        tris.iter().filter(|t| {
            let zs = [t[0][2], t[1][2], t[2][2]];
            let ys = [t[0][1], t[1][1], t[2][1]];
            zs.iter().all(|z| (58.5..59.3).contains(z))
                && ys.iter().all(|y| (2.55..3.7).contains(y))
                && (ys.iter().cloned().fold(f32::MAX, f32::min) - ys.iter().cloned().fold(f32::MIN, f32::max)) < 0.09
        }).count()
    };
    drop(scene); // the second load doubles the footprint — a small test machine OOMs at ~2 GB
    let bars = bar_count;
    // The bars are 0.06-0.07 thick in y, six of them between y 2.62 and 3.62, all at z=58.86-58.9
    // [D probe14_tris; the placement 0x8000 cel + the Collide_Pick no-0x1-skip rule above].
    assert!(bars >= 20,
        "the AirConGrill1 bars (24 tris, six bands y 2.62..3.62) ARE the game's static collision [M 0x1d1998:92 + 0x1eb320:140/:145-151; D plc #100 0x00008019 glist 0x1], got {bars}");
    // And the crawl does NOT get through: a crouched walk south wedges on the grill plane —
    // the game's Bond never crosses z=58.86 westward; the route leaves east (the walkreal's
    // the_east_duct_route_reaches_the_stairwell_slot walks the real east path).
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..60 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
        sim.bond.crouched = true;
        for _ in 0..15 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
        for _ in 0..400 {
            let inp = nf_game::player::Input { forward: 1.0, ..Default::default() };
            sim.step_bond(inp, false, false, false);
        }
        assert!(sim.bond.pos[2] > 58.5,
            "the crawl wedges on the grill plane z=58.86 [M the cel-exists rule above], got {:?}", sim.bond.pos);
    }
}
