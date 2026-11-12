//! The Exchange's castle exit (07000005) is closed to Bond on foot: the LoadLevel trigger #118's
//! door mesh (Door_Gate8, x 27.4-32.6, y 36-41.7, z 38.0-38.4 [D]) is solid.
//! [L PCSX2-gdb 2026-11-12, level loaded by the game's own loader, Bond placed at
//! (30.0, 37.2, 41.5) facing -z, forward held 360 frames]: he stops at z 38.89 and stays.
//! Before this the port let him walk through and slide down the gorge to y -9.19.
//! NF_ISO-gated.
use nf_view::scene::Sim;

fn walk_into_the_exit() -> [f32; 3] {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    let mut sim: Sim = scene.sim.take().expect("sim");
    for _ in 0..2000 {
        if !sim.disabled { break; }
        sim.drones.scripts.skip_request = true;
        sim.step_bond(Default::default(), false, false, false);
    }
    sim.bond = nf_game::player::Bond::new([30.0, 37.2, 41.5], std::f32::consts::PI);
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    let walk = nf_game::player::Input { forward: 1.0, ..Default::default() };
    for _ in 0..360 { sim.step_bond(walk, false, false, false); }
    sim.bond.pos
}

#[test]
fn the_castle_exit_door_stops_bond_like_the_game() {
    let p = walk_into_the_exit();
    assert!(p[2] > 38.4 && p[2] < 39.3, "Bond stops at the door like the game's z 38.89 [L], got {p:?}");
    assert!(p[1] > 36.0, "Bond stays on the gate floor, got {p:?}");
}
