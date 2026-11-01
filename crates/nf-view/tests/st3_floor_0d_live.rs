//! 0700000d, the ST-3 interior: Bond placed in the air at (137, 10.5, −300) falls to the floor.
//! The game, placed there, lands him at y 5.02 (the ST-3 floor is the static y 3.985 face [D];
//! 5.02 = 3.985 + the stand height cc 1.0347 [M Player_FeetOnPoint 0x194188:106 band]). Nothing
//! solid sits between 10.5 and the floor: the only part over the spot is the stair-top dynamic
//! object (box y 8.35..14.32), whose faces at x 137 lie at z −293, seven metres north.
//! Sister: placed on that floor he stays there, grounded.

use nf_game::player::{Bond, Input};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// 0d loaded and run past its intro cut-scene (the script camera holds Bond while it runs).
fn load_past_intro(iso: &str) -> nf_view::scene::Scene {
    let mut scene = nf_view::load(iso, "0700000d.bin", false).expect("load 0700000d");
    for c in [0x1bu8, 0x1c] { scene.sim.as_mut().unwrap().drones.set_channel(c); }
    let (mut seen, mut quiet) = (false, 0);
    for _ in 0..6000 {
        let sim = scene.sim.as_mut().unwrap();
        let busy = sim.disabled || sim.script_camera().is_some();
        seen |= busy;
        if seen && !busy { quiet += 1; if quiet > 60 { break; } }
        sim.step_bond(Input::default(), false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(seen && !sim.disabled && sim.script_camera().is_none(), "the intro must end");
    scene
}

fn settle(scene: &mut nf_view::scene::Scene, p: [f32; 3], ticks: u32) -> (Vec<[f32; 3]>, bool) {
    let sim = scene.sim.as_mut().unwrap();
    sim.bond = Bond::new(p, std::f32::consts::PI);
    sim.bond.cel = sim.world.find_cel(p);
    let mut path = Vec::new();
    for _ in 0..ticks {
        sim.step_bond(Input::default(), false, false, false);
        path.push(sim.bond.pos);
    }
    (path, sim.bond.grounded)
}

#[test]
fn bond_placed_above_the_st3_floor_falls_to_it() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut scene = load_past_intro(&iso);
    {
        let sim = scene.sim.as_ref().unwrap();
        let from = [137.0, 11.0, -300.0];
        let to = [137.0, 2.5, -300.0];
        let (t, n) = sim.world.segment_hit(from, to).expect("a floor under the spot");
        let y = from[1] + t * (to[1] - from[1]);
        assert!((y - 3.985).abs() < 0.01 && n[1] > 0.99, "the first face under (137, −300) is the 3.985 floor, got y {y} n {n:?}");
    }
    let (path, grounded) = settle(&mut scene, [137.0, 10.5, -300.0], 150);
    let end = *path.last().unwrap();
    assert!(path[40][1] < 9.0, "he is falling 40 ticks in, got y {}", path[40][1]);
    assert!(grounded, "he lands");
    assert!((end[1] - 5.02).abs() < 0.05, "he lands at the game's y 5.02, got {end:?}");
    assert!((end[0] - 137.0).abs() < 0.05 && (end[2] + 300.0).abs() < 0.05, "a straight drop, got {end:?}");
}

#[test]
fn bond_placed_on_the_st3_floor_stays_there() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut scene = load_past_intro(&iso);
    for p in [[137.0f32, 5.02, -300.0], [133.0, 5.02, -298.0]] {
        let (path, grounded) = settle(&mut scene, p, 90);
        let end = *path.last().unwrap();
        assert!(grounded, "grounded on the floor at {p:?}");
        assert!((end[1] - 5.02).abs() < 0.05, "held at 5.02 from {p:?}, got {end:?}");
        assert!(path.iter().all(|q| q[1] > 4.9), "never sinks through the floor from {p:?}");
    }
}
