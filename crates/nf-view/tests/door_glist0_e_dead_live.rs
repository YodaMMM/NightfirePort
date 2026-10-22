//! Real-path proof for door-glist0-use-standin: the deleted glist-0 stand-in. Runs through the REAL
//! Sim (`nf_view::load` + `Sim::step_bond`), on 07000001's slide door — the same stand the
//! `level_end_live::door_e_opens_then_e_closes_on_07000001` test opens from (1.6 out on the through
//! axis, inside the old stand-in's 2.5 of the placement).
//!
//! Game rule: a door without a resolved glist (obj+0xd8 == 0) is never admitted to the use pick
//! [M Collide_PickObj 0x1eae80:88-90] and never even created [M parsemap_block_map_data_dynamic
//! 0x1d1b68:93-95]. The port's create fallback can still produce such a record if the by_hash map
//! breaks; breaking the glist here reproduces that record on the REAL path and pins the E-dead
//! behaviour. The old stand-in opened this door from this stand (1.6 < 2.5) — this test FAILED on it.
//!
//! Sister: the unbroken door opens from the same stand (the sphere rule untouched).


fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load(iso: &str) -> nf_view::scene::Scene {
    nf_view::load(iso, "07000001.bin", false).expect("load 07000001")
}

fn step_at(scene: &mut nf_view::scene::Scene, pos: [f32; 3], yaw: f32, use_key: bool) {
    if let Some(sim) = scene.sim.as_mut() {
        sim.bond.pos = pos;
        sim.bond.yaw = yaw;
        sim.bond.pitch = 0.0;
        sim.step_bond(Default::default(), false, false, use_key);
    }
}

/// The slide door + the 1.6-out stand on its through axis (the level_end_live E stand).
fn door_and_stand(scene: &nf_view::scene::Scene) -> (usize, [f32; 3], f32) {
    let (idx, pos) = scene.sim.as_ref().unwrap().drones.doors.list.iter().enumerate()
        .find(|(_, d)| d.kind == 2)
        .map(|(i, d)| (i, d.pos))
        .expect("a slide door exists on 07000001");
    (idx, [pos[0] + 1.6, pos[1], pos[2]], std::f32::consts::FRAC_PI_2 * -1.0)
}

#[test]
fn a_broken_glist_door_is_e_dead_on_the_real_path() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso);
    let (idx, stand, yaw) = door_and_stand(&scene);
    // Break the glist: the create-fallback record the game would never make [M 0x1d1b68:93-95].
    scene.sim.as_mut().unwrap().drones.doors.list[idx].glist = 0;
    step_at(&mut scene, stand, yaw, true);
    let d = &scene.sim.as_ref().unwrap().drones.doors.list[idx];
    assert!(d.state & 0xc == 0,
        "an obj+0xd8==0 door is not admitted to the use pick [M 0x1eae80:88-90] — got state 0x{:x}", d.state);
    eprintln!("[glist0] broken-glist door E-dead at the 1.6 stand (state 0x{:x}) — the stand-in would have opened it", d.state);
}

/// Sister: the same door with its REAL glist opens from the same stand (the sphere rule intact).
#[test]
fn the_same_door_still_opens_on_e_with_its_glist() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso);
    let (idx, stand, yaw) = door_and_stand(&scene);
    assert_ne!(scene.sim.as_ref().unwrap().drones.doors.list[idx].glist, 0,
        "the door has a resolved glist (the census pins this)");
    step_at(&mut scene, stand, yaw, true);
    let d = &scene.sim.as_ref().unwrap().drones.doors.list[idx];
    assert!(d.state & 0xc != 0, "the resolved door opens on E (state 0x{:x})", d.state);
    eprintln!("[glist0] sister: same door, same stand, E opens (state 0x{:x})", d.state);
}
