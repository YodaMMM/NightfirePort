//! agent-pane-solid: a live pane is solid to Bond, a broken one is not, on 07000006.
//!
//! Pane #376 [D placements]: kind 2, health 0, glist flags 0x1201, material 0x10, a 1.55 x 1.55
//! sheet at z -16.3265 filling the hatch in the wall at z -16.5 (floor 40.0 on the north side,
//! the sill 40.65). `Break_Create` 0x12c230 links the pane into its room as an object with its
//! own collision, and `Collide_PickObj` 0x1eae80 admits it to every query whose HITTEST+0x98 does
//! not filter it: Bond's walk carries 0 [M Player_Collision 0x194df0:88]. `Break_Kill` 0x12b970:60
//! takes it out of the world (obj+0xFE |= 1).
use nf_game::player::Input;

const PANE: usize = 376;
const PLANE_Z: f32 = -16.3265;

/// Bond crouched on the north side of the hatch, walking south (+z) into it for six seconds.
/// Returns the furthest z he reached.
fn crouch_into_the_hatch(kill: bool) -> f32 {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut s = nf_view::load(&iso, "07000006.bin", false).expect("load 06");
    let sim = s.sim.as_mut().unwrap();
    for _ in 0..5 { sim.step_bond(Input::default(), false, false, false); }
    let i = sim.drones.breaks.list.iter().position(|b| b.placement == PANE).expect("pane #376 is placed [D]");
    assert!(!sim.drones.breaks.list[i].dead, "the pane starts whole");
    if kill {
        let c = [30.125, 41.524, PLANE_Z];
        assert!(sim.drones.breaks.kill(i, c, c, &mut sim.drones.rng).is_some() || sim.drones.breaks.list[i].dead);
        // The kill drops the part on the next tick's set_dynamic.
        sim.step_bond(Input::default(), false, false, false);
    }
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, [30.1, 41.0, -18.0], 0.0);
    for _ in 0..10 { sim.step_bond(Input { crouch: true, ..Default::default() }, false, false, false); }
    let mut max_z = f32::NEG_INFINITY;
    for _ in 0..360 {
        sim.step_bond(Input { forward: 1.0, crouch: true, ..Default::default() }, false, false, false);
        max_z = max_z.max(sim.bond.pos[2]);
    }
    max_z
}

#[test]
fn the_live_pane_stops_bond_in_the_hatch() {
    if std::env::var_os("NF_ISO").is_none() { return; }
    let z = crouch_into_the_hatch(false);
    assert!(z < PLANE_Z, "the live pane stops Bond short of its plane (reached z {z:.3}, plane {PLANE_Z})");
}

#[test]
fn the_broken_pane_lets_bond_through_the_hatch() {
    if std::env::var_os("NF_ISO").is_none() { return; }
    let z = crouch_into_the_hatch(true);
    assert!(z > PLANE_Z + 2.0, "with the pane broken Bond crawls through the hatch (reached z {z:.3})");
}
