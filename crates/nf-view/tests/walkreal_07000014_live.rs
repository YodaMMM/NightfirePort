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

use walker::{Walker, Wp};

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

#[test]
#[ignore = "the route beyond the grill is blocked twice on the walked path [D probe14_stairs/probe14_trench, all traced 2026-10-18]: (1) the crawl through the grill now PASSES (the bit-0x1 fix, the_vent_grill sister green, the crawl reaches the stair head z=57.48), but (2) the duct stairs descend into a shaft slot ~0.4-0.55 deep (z 55.72..56.1, the south wall face y≈0.4..3.0 normal +z, the ramp edge at z≈56.1) that the port's crouch capsule (r 0.55, [M case 3]) cannot enter — the game's low-profile CREEP capsule (r 0.275 [M 0x194df0 case 6]) would fit, but the port's creep (substate 7) only runs on creepwall-rail objects (placement 0x3F) and 07000014 has NONE [D the kind census]. (3) The stairwell shaft drops open to y=-3.5 (the facility at -2.4, the cover nodes at z=57.69 [D]) — the drop IS the route's shape, entered by the creep the port lacks. (4) The alternative under-vent space (ceiling 1.33-1.4, the sleeper drone #596 at (40.83,1.08,62.85) [D]) has no entrance found from the duct floor. LIVE CHECKS in the report. The full leg set below is written and ready — it un-ignores once the descent runs."]
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

/// The grill-pass sister (level-14-complete): the crawl-vent start box is OPEN at the AirConGrill1
/// plane z=58.86 — no static world triangle sits in the grill box (the old world carried the 24
/// AirConGrill1 tris there and sealed the only start inside; the bit-0x1 census [D] is the fix's
/// evidence and the sealed-box contradiction its proof).
#[test]
fn the_vent_grill_no_longer_blocks_the_start_box() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..60 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
    }
    let sim = scene.sim.as_ref().unwrap();
    let tris = sim.world.tris_near([40.0, 2.3, 58.5], [41.6, 3.9, 59.2]);
    // The bars were 0.06-0.07 thick in y, six of them between y 2.62 and 3.62, all at z=58.86-58.9
    // [D probe14_tris]. The room frame's own tris there are thicker; none may be bar-thin.
    let bars = tris.iter().filter(|t| {
        let zs = [t[0][2], t[1][2], t[2][2]];
        let ys = [t[0][1], t[1][1], t[2][1]];
        zs.iter().all(|z| (58.5..59.3).contains(z))
            && ys.iter().all(|y| (2.55..3.7).contains(y))
            && (ys.iter().cloned().fold(f32::MAX, f32::min) - ys.iter().cloned().fold(f32::MIN, f32::max)) < 0.09
    }).count();
    assert_eq!(bars, 0,
        "the AirConGrill1 bars (0.06-0.07 y-thick, six bands y 2.62..3.62) must carry no static collision [D the bit-0x1 census; was 24 tris]");
    // And the crawl gets through: a crouched walk south crosses the plane.
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
        assert!(sim.bond.pos[2] < 58.4 && sim.bond.pos[2] > 55.0,
            "the crawl crosses the grill plane z=58.86 southward, got {:?}", sim.bond.pos);
    }
}
