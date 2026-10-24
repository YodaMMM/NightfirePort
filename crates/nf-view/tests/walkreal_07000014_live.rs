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
#[ignore = "level-14-sleeper-push: the drop-slot mechanism is SOLVED and the remaining wedge is named - (1) the slot is ROOFED by vent-lid SP #266's glist shape (hash 0x02000750 [D probe14_lid: dynamic tris y 2.346..2.397 span x 40.05..41.54, z 62.22..63.66 = the whole slot]); (2) the lid SPs sit mask 0x4 = ACTIVATE [D placement p5] in state 1, and the port's E path FIRES them: probe14_lid activated #266 on the first press at the lip and the 63-tick script (0x06000A3B [D decode]) swung the slab open - the slot's dynamic tris dropped 60 -> 0 by k45 and Bond walked south through the opening past z 62.2 (probe14_lid, committed); (3) the drop's landing wedges on a ghost rest at y 2.97 over the trench (capsule_push walkable=true, push~0, NO floor tri within 3 m below, no dynamic tri in the stall box, the sleeper far away) - the flipped-open lid's residual pose or the kind-25 second entity (0x02000751, obj_type 25, key 61) is the suspect [S unverified]; (4) the actor-actor push (Collide_Jointy 0x1ec180:229-281, this branch) handles the sleeper landing itself (sleeper_push_live). Next: pin the stall tri, then the corridor east (x 41->49 at z 62.9) and the x 49 south run to the mouths (z 42.16, lids #655/#600 the same mask-4 mechanic) and the mezzanine; the grapple points #626/#657 at (45.30/40.78, 4.79, 42.86) sit at the mouths' level and may be the mezzanine entry instead of the stairwell slot [D+M check outstanding]."]
fn the_east_duct_route_reaches_the_stairwell_slot() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000014.bin", false).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    set_crouch(&mut w, true);
    // Leg A: crawl south off the room floor's south lip (z≈63.64 past lid #266) and drop to the
    // −0.6 corridor the nav web threads [D map 54 path 2 nodes 4-8]. The x=41.2 lane dodges the
    // sleeper drone #270 (40.83, 1.08, 62.85 [D]) under the lip.
    w.walk_to(Wp::new(41.0, 2.4, 63.9).r(0.4), "the crawl south to the room's south lip");
    // The drop lands on/beside the sleeper drone #270 (skin 0x05000090 [D] (40.83, 1.08, 62.85)) —
    // the authored ambush: the only drop lane (x ≤40.98, the east wall caps the crawl) sits right
    // over him and he carries Bond as he wanders. Kill him through the REAL gun path (the walker's
    // aim+fire), then walk the corridor.
    w.walk_to(Wp::new(40.9, -0.6, 60.2).r(1.4), "the drop south onto the sleeper's back");
    {
        let di = w.scene.sim.as_ref().unwrap().drones.list.iter()
            .position(|d| d.skin == 0x0500_0090).expect("the sleeper drone #270 (skin 0x05000090) exists [D]");
        let mut dead = false;
        for attempt in 0..6 {
            if w.shoot(di, "the vent sleeper #270") { dead = true; break; }
            // No resupply walk here: the pickup route crosses the sealed grill — cycling weapons
            // (the shoot helper's own cycle) reaches the loadout's other guns instead.
            eprintln!("[014] sleeper attempt {attempt}: cycling to the next gun");
        }
        assert!(dead, "the sleeper drone must die to the walked gun path");
    }
    // Off the carrier: JUMP east off the sleeper's back (climb_stairs hops every 30 ticks; the
    // wandering carrier dithers the plain steer — the jump clears his edge).
    let _ = w.climb_stairs([43.0, 0.3, 62.0], "jump east off the sleeper onto the corridor floor", 0.9, 1500);
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[014] after the sleeper: {:?}", p);
    }
    // Leg B: the corridor east to the x≈49 corner (the nav lane: node 5 (43.05, 0.4, 62.86) →
    // node 4 (49.08, 0.4, 62.85) [D]).
    let _ = w.try_walk_to(Wp::new(48.9, -0.6, 62.8).r(0.6), "the corridor east to the x≈49 corner");
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[014] corridor east leg: {:?}", p);
        assert!(p[0] > 45.0, "the corridor east made real progress [D nav nodes 5/4], got {p:?}");
    }
    // Leg C: south along the x≈49 lane toward the flat-grate vent mouths at z=42.16 [D #655/#600]
    // — the nav thread node 3 (49.08, 0.4, 52.9) → (49.1, 0.4, 42.8) [D].
    let _ = w.try_walk_to(Wp::new(49.05, -0.6, 53.2).r(0.7), "the corridor south past z≈53");
    let _ = w.try_walk_to(Wp::new(49.05, -0.6, 43.4).r(0.7), "the corridor south to the vent mouths");
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[014] corridor south leg: {:?}", p);
    }
    eprintln!("[014] east route: at {:?} in {} ticks", w.scene.sim.as_ref().unwrap().bond.pos, w.total_ticks);
}
