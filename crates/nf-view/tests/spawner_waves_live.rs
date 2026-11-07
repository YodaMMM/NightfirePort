//! Spawner waves on the real levels: a template's record outlives its guards, the alive scan frees
//! a slot the moment its guard is dead, the mode-3 permission stays set across a wave, and p8 is an
//! integer distance [M DroneSpawner_Create 0x1378d0, DroneSpawner_Control 0x137e68,
//! DroneSpawner_SpawnDrone 0x137cb0; D the placements]. Opt-in: NF_ISO.

use nf_game::drone::state;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Far from every group-6 spot on 0700000d: the level start [D #1].
const CLEAR_0D: [f32; 3] = [179.53, 11.09, -205.42];

fn step(scene: &mut nf_view::scene::Scene, n: u32) {
    for _ in 0..n { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
}

fn spawner(sim: &nf_view::scene::Sim, group: u32) -> &nf_game::spawners::Spawner {
    sim.drones.spawners.iter().find(|s| s.group == group).expect("the spawner")
}

/// The guards of a spawner that are up: present, not a corpse, health above zero.
fn up(sim: &nf_view::scene::Sim, group: u32) -> Vec<usize> {
    spawner(sim, group).spawned.iter().copied()
        .filter(|&i| { let d = &sim.drones.list[i]; d.present && !d.corpse && d.health > 0.0 }).collect()
}

fn kill(sim: &mut nf_view::scene::Sim, i: usize) {
    let d = &mut sim.drones.list[i];
    d.health = 0.0;
    d.set_state(state::DEATH_ANIM);
}

fn first_wave_0d() -> nf_view::scene::Scene {
    let mut scene = nf_view::load(&iso(), "0700000d.bin", false).expect("load");
    step(&mut scene, 900); // the intro
    scene.sim.as_mut().unwrap().drones.set_channel(5);
    for _ in 0..300 {
        step(&mut scene, 1);
        if up(scene.sim.as_ref().unwrap(), 6).len() == 5 { break; }
    }
    assert_eq!(up(scene.sim.as_ref().unwrap(), 6).len(), 5, "the first wave");
    scene
}

/// Every spawner's distance is the integer p8 as a float, 0 meaning 10 [M 0x1379c0-0x1379d4 cvt.s.w;
/// 0x1378d0:35-37]. 0700000d groups 2/4/5 carry 10, groups 6/7/8 carry 5; 07000011's group 7
/// carries 1 and group 5 carries 5 [D].
#[test]
fn p8_is_an_integer_distance() {
    let scene = nf_view::load(&iso(), "0700000d.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    for (g, want) in [(2, 10.0), (4, 10.0), (5, 10.0), (6, 5.0), (7, 5.0), (8, 5.0)] {
        assert_eq!(spawner(sim, g).min_dist, want, "0700000d group {g}");
    }
    let scene = nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 2).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    for (g, want) in [(1, 10.0), (5, 5.0), (7, 1.0)] {
        assert_eq!(spawner(sim, g).min_dist, want, "07000011 group {g}");
    }
}

/// A screened spawn keeps clear of Bond: with him standing on a group-6 spot on 0700000d, that spot
/// stays empty while the rest of the group comes in.
#[test]
fn a_screened_spawn_never_lands_on_bond() {
    let mut scene = nf_view::load(&iso(), "0700000d.bin", false).expect("load");
    step(&mut scene, 900);
    let (spot, t0) = {
        let sim = scene.sim.as_mut().unwrap();
        let t = spawner(sim, 6).templates[0];
        let spot = sim.drones.list[t].pos;
        sim.drones.set_channel(5);
        (spot, t)
    };
    for _ in 0..240 {
        { let sim = scene.sim.as_mut().unwrap(); sim.bond.pos = spot; sim.bond.vel = [0.0; 3]; }
        step(&mut scene, 1);
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(!up(sim, 6).is_empty(), "sister: the group was released");
    assert!(!sim.drones.list[t0].present, "the spot under Bond stays empty [M 0x137cb0:28-31]");
}

/// 07000011 group 1 (mode 3, 2 alive, 2 waves, start 0x15 [D #184]): both guards of a wave come in
/// together, and the second pair follows the first pair's deaths while the corpses still lie there.
#[test]
fn a_mode3_wave_fills_together_and_the_second_wave_comes() {
    // The group's four templates are level-tag 2 [D]: Agent and up.
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 2).expect("load");
    step(&mut scene, 900); // the intro
    {
        let sim = scene.sim.as_mut().unwrap();
        assert!(up(sim, 1).is_empty(), "sister: the group holds before its channel");
    }
    let mut first = Vec::new();
    for _ in 0..240 {
        // The touch #182 re-fires only while it is stood on [D].
        { let sim = scene.sim.as_mut().unwrap(); sim.bond.pos = [8.25, 47.9, -2.3]; sim.bond.vel = [0.0; 3]; }
        step(&mut scene, 1);
        first = up(scene.sim.as_ref().unwrap(), 1);
        if first.len() == 2 { break; }
    }
    assert_eq!(first.len(), 2, "the second guard does not wait for the first to die [M 0x137e68:176-203]");
    { let sim = scene.sim.as_mut().unwrap(); for &i in &first { kill(sim, i); } }
    let mut second = Vec::new();
    for _ in 0..240 {
        step(&mut scene, 1);
        second = up(scene.sim.as_ref().unwrap(), 1);
        if second.len() == 2 { break; }
    }
    assert_eq!(second.len(), 2, "the second wave comes while the corpses still lie there [M 0x137e68:96-124]");
    let sim = scene.sim.as_ref().unwrap();
    for &i in &second {
        let d = &sim.drones.list[i];
        assert!(!d.corpse && d.health > 0.0 && d.enabled, "guard {i} is whole");
        assert!(!matches!(d.state, state::DEAD | state::DEATH_ANIM | state::FADE | state::FADE_FAST), "guard {i} is out of the death states: {}", d.state);
        assert_eq!(sim.drones.llist.iter().filter(|&&k| k == i).count(), 1, "guard {i} updates once a tick");
    }
    assert_eq!(spawner(sim, 1).wave, 1, "the second wave");
}

/// 0700000d group 6 (mode 2, 5 templates, 3 waves, start 5 [D]): each death brings a guard of the
/// next wave while the others stand. Three waves are fifteen guards.
#[test]
fn three_waves_are_three_waves() {
    let mut scene = first_wave_0d();
    let mut killed = 0;
    for _ in 0..6000 {
        step(&mut scene, 1);
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = CLEAR_0D;
        if let Some(&i) = up(sim, 6).first() {
            kill(sim, i);
            killed += 1;
        }
        if spawner(sim, 6).gone { break; }
    }
    assert_eq!(killed, 15, "three waves of five [D p3 = 3; M 0x137e68:145-172]");
}

/// A guard added beside a template still in use draws with a body of his skin.
#[test]
fn a_guard_beside_a_live_template_is_drawn() {
    let mut scene = first_wave_0d();
    let n0 = scene.sim.as_ref().unwrap().drones.list.len();
    for _ in 0..600 {
        {
            let sim = scene.sim.as_mut().unwrap();
            sim.bond.pos = CLEAR_0D;
            let live = up(sim, 6);
            if live.len() == 5 { kill(sim, live[0]); }
        }
        step(&mut scene, 1);
        if scene.sim.as_ref().unwrap().drones.list.len() > n0 { break; }
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.list.len() > n0, "a later-wave guard was added");
    let chars: Vec<usize> = (0..sim.char_drone.len()).filter(|&c| sim.char_drone[c] == Some(n0)).collect();
    assert!(!chars.is_empty(), "the added guard has a body");
    assert!(chars.iter().all(|&c| !scene.char_hidden(c, Default::default())), "and it is not hidden");
    assert_eq!(sim.drones.list[n0].index, n0);
}
