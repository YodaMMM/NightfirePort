//! 07000007: Drake's reveal played from a Sim-level test (the real path's order before plate #412).

use nf_game::player::Input;

/// Drake's reveal first, the real path's order: the library door pair #404/#417 stands in front of
/// TouchOnce #412 and its zone-3 band covers the plate (Bond there reads the 0xc mission fail
/// [M NDrone2_DSTATE_CivilianDoorGuard 0x160648:89-94]) until ch 21 takes the pair off [D p6 = 21].
/// Bond is put on the balcony plate #356 (ch 0x15), and the fly-through 0x06000072 is skipped with
/// the game's skip action, as a player does; it writes ch 21.
pub fn drakes_reveal(sim: &mut nf_view::scene::Sim) {
    let c = sim.drones.triggers.iter()
        .find(|t| t.kind == nf_game::triggers::kind::TOUCH_ONCE && u32::from(t.out) == 0x15)
        .map(|t| t.centre()).expect("TouchOnce #356 exists");
    sim.bond.pos = [c[0], c[1], c[2]];
    for _ in 0..120 { sim.step_bond(Input::default(), false, false, false); if sim.drones.channels.on(0x15) { break; } }
    assert!(sim.drones.channels.on(0x15), "the reveal plate #356 fired ch 0x15 [D]");
    for _ in 0..4200 {
        if sim.disabled { sim.drones.scripts.skip_request = true; }
        sim.step_bond(Input::default(), false, false, false);
        if !sim.disabled && sim.drones.channels.on(21) { break; }
    }
    assert!(!sim.disabled && sim.drones.channels.on(21), "the reveal ran and wrote ch 21");
    assert!(!sim.drones.list.iter().any(|g| matches!(g.placement, 404 | 417) && g.present), "the library door pair left on ch 21");
}
