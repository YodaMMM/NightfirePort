//! walk-0a-de-teleport: the recorded Operative real-damage walk of 0700000a
//! (`walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch_real_damage`, which writes
//! tests/replays/0700000a_operative.bin with NF_REC_0A_WRITE=1 and otherwise asserts its inputs
//! still match it) replays HEADLESS on the same warm load to the same hatch exit: the walk's
//! inputs alone carry it, nothing else touched the sim. NF_ISO-gated.

#[test]
fn replay_the_operative_walk_headless_to_the_hatch_exit() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/replays/0700000a_operative.bin");
    let replay = nf_view::replay::Replay::load(path).expect("the committed Operative recording");
    assert_eq!(replay.header.level, "0700000a.bin");
    let ticks = replay.tick_count();
    assert!(ticks > 10000, "the full-mission recording carries the whole walk, got {ticks}");

    // The walk's own load: the warm carry from the elevator chain (ch 0x46 rides the RamSave
    // [M 0x195670:13]), harvested from a cold load exactly as walkreal_0700000a's load_carry.
    let cold = nf_view::load_with_carry(&iso, "0700000a.bin", false, None).expect("load 0700000a cold");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut scene = nf_view::load_with_carry(&iso, "0700000a.bin", false, Some(&save)).expect("load 0700000a warm");
    let states = nf_view::replay::replay_headless(&mut scene, &replay)
        .expect("the headless replay (the header's RNG/pose/weapon checks are the load-determinism proof)");
    assert_eq!(states.len() as u32, ticks + 1, "one snapshot per tick plus the starting state");
    if let Some(live) = std::env::var_os("NF_REC_0A_STATES").and_then(|p| std::fs::read_to_string(p).ok()) {
        let first = live.lines().zip(states.iter().skip(1)).position(|(l, s)| {
            let v: Vec<f32> = l.split_whitespace().filter_map(|x| x.parse().ok()).collect();
            v.len() == 3 && (0..3).any(|k| (v[k] - s.bond_pos[k]).abs() > 1e-3)
        });
        eprintln!("[0a-parity] first divergent tick: {first:?}");
        if let Some(i) = first { eprintln!("[0a-parity] live {} replay {:?}", live.lines().nth(i).unwrap_or(""), states[i + 1].bond_pos); }
    }
    let sim = scene.sim.as_ref().unwrap();
    // The hatch (21.72, 1.12, 19.79) loads 0700000b [D] — the walk's own end assert.
    assert_eq!(sim.drones.level_exit, Some(0x0700_000b), "the replayed walk reaches the hatch exit into 0700000b");
    assert!(sim.drones.health.health > 0.0, "Bond is alive at the end");
}
