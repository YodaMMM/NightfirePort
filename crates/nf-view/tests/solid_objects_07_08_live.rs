//! Placed objects Bond cannot walk through on 07000007 and 07000008. `DynamicObject_Create`
//! 0x18dbf8 and `Break_Create` 0x12c230 both give the object its glist and link it into the room's
//! cel chain, so `Collide_PickObj` 0x1eae80 admits it for the body query (HITTEST+0x98 = 0) while
//! obj+0xF0 lacks bit 0x20: the party's rope barriers and the control-room glass stop him.

use nf_game::player::{Bond, Input};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn walk(sim: &mut nf_view::scene::Sim, from: [f32; 3], yaw: f32, ticks: u32) -> [f32; 3] {
    sim.bond = Bond::stand_at(&sim.world, from, yaw);
    for _ in 0..5 { sim.step_bond(Input::default(), false, false, false); }
    for _ in 0..ticks { sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false); }
    sim.bond.pos
}

/// #466 (126.7, 43, −63.8) [D] stands 0.16 thick across the walk south.
#[test]
fn the_party_rope_barrier_stops_a_walk() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    let end = walk(sim, [126.7, 44.5, -61.8], std::f32::consts::PI, 120);
    assert!(end[2] > -63.45, "the barrier holds Bond off its face: {end:?}");
}

/// #495 (80.48, 43, −50.49) is created hidden behind ch 21 [D p0; M 0x18dbf8:27-29] and takes its
/// collision with its picture when the channel rises [M 0x18d3b8 gate arm].
#[test]
fn the_gated_rope_barrier_turns_solid_with_its_channel() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    let hits = |sim: &nf_view::scene::Sim| (0..30).filter(|k| {
        let (y, z) = (43.2 + 0.1 * *k as f32, -50.5);
        sim.world.segment([81.5, y, z], [80.3, y, z]).is_some()
    }).count();
    for _ in 0..3 { sim.step_bond(Input::default(), false, false, false); }
    assert_eq!(hits(sim), 0, "hidden behind ch 21 the barrier is not there");
    let frame = sim.drones.frame;
    sim.drones.channels.set(21, 1, frame);
    for _ in 0..3 { sim.step_bond(Input::default(), false, false, false); }
    assert!(hits(sim) > 0, "ch 21 stands it up");
}

/// Pane #176 (z −122, x 105.25..109.25) [D]: the walk north of it stops at the glass; with the
/// pane gone the same walk goes out through the frame.
#[test]
fn the_control_room_glass_stops_a_walk_until_it_breaks() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    let mut scene = nf_view::load(&iso, "07000008.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    let from = [107.2, 35.0, -120.4];
    let end = walk(sim, from, std::f32::consts::PI, 120);
    assert!(end[2] > -121.7, "the whole pane stops Bond: {end:?}");
    let pane = sim.drones.breaks.list.iter_mut().find(|b| b.placement == 176).expect("#176");
    pane.dead = true;
    pane.dead_frame = 0;
    let end = walk(sim, from, std::f32::consts::PI, 120);
    assert!(end[2] < -122.5, "a broken pane leaves the frame open: {end:?}");
}
