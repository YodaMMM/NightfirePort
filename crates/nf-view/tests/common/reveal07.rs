//! 07000007: Drake's reveal played from a Sim-level test (the real path's order before plate #412).

use nf_game::player::Input;

/// Drake's reveal first, the real path's order: the library door pair #404/#417 stands in front of
/// TouchOnce #412 and its zone-3 band covers the plate (Bond there reads the 0xc mission fail
/// [M NDrone2_DSTATE_CivilianDoorGuard 0x160648:89-94]) until ch 21 takes the pair off [D p6 = 21].
/// Bond is put on the balcony floor 2 u south of plate #356 (ch 0x15) and walks north into it,
/// and the fly-through 0x06000072 is skipped with the game's skip action, as a player does; it
/// writes ch 21.
///
/// The plate is a thin slab across the balcony (box 92.9–98.0, 49.0–53.8, −79.8–−78.9 [D]). The
/// game's contact is the walk HITTEST segment against the slab's faces from their front side [M
/// Intersect_CylGeom 0x109da0, triggers::cyl_geom_min]; there is no containment contact [M
/// Trigger_Hit 0x1b1fc8 reads only the pick's list], so a Bond dropped at the slab's centre
/// touches nothing. The walk in crosses the south face, as the real route does
/// (walker::drakes_reveal_07).
pub fn drakes_reveal(sim: &mut nf_view::scene::Sim) {
    let c = sim.drones.triggers.iter()
        .find(|t| t.kind == nf_game::triggers::kind::TOUCH_ONCE && u32::from(t.out) == 0x15)
        .map(|t| t.centre()).expect("TouchOnce #356 exists");
    let spot = [c[0], c[1], c[2] + 2.0];
    let floor = sim.world.segment(spot, [spot[0], spot[1] - 30.0, spot[2]]).map(|t| spot[1] - 30.0 * t);
    sim.bond.pos = [spot[0], floor.map_or(spot[1], |f| f + 0.1), spot[2]];
    sim.bond.vel = [0.0; 3];
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    assert!(!sim.drones.channels.on(0x15), "2 u short of the plate, ch 0x15 is off");
    sim.bond.yaw = std::f32::consts::PI; // face −z, the plate
    for _ in 0..180 {
        sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
        if sim.drones.channels.on(0x15) { break; }
    }
    assert!(sim.drones.channels.on(0x15), "walking north into plate #356 fired ch 0x15 [D], bond {:?}", sim.bond.pos);
    for _ in 0..4200 {
        if sim.disabled { sim.drones.scripts.skip_request = true; }
        sim.step_bond(Input::default(), false, false, false);
        if !sim.disabled && sim.drones.channels.on(21) { break; }
    }
    assert!(!sim.disabled && sim.drones.channels.on(21), "the reveal ran and wrote ch 21");
    assert!(!sim.drones.list.iter().any(|g| matches!(g.placement, 404 | 417) && g.present), "the library door pair left on ch 21");
}
