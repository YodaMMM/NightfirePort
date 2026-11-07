//! The 07000014 generator hall goes up in stages. One write of ch 0x51 starts the chain; each
//! leaf smashes when its watch channel rises and writes its own channel after its countdown
//! [M Destroy_Update 0x12c4f8:0x12c518-40; D p6 on the placements; L −30 per p6 at rest].
//! The offsets are the game's, read frame by frame on a fresh load with one poke of 0x51 [L].
//!
//! Opt-in via NF_ISO.

const STAGES: [(u32, u32); 9] = [(0x52, 31), (0x4e, 62), (0x53, 122), (0x5c, 153), (0xaf, 213),
    (0x5a, 274), (0xc9, 304), (0x5b, 335), (0x85, 515)];

#[test]
fn the_hall_chain_rises_on_the_games_frames() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    for _ in 0..400 {
        if !scene.sim.as_ref().unwrap().disabled { break; }
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    }
    let sim = scene.sim.as_mut().unwrap();
    let frame = sim.drones.frame;
    sim.drones.channels.set(0x51, 1, frame);
    let mut rose = [None; 9];
    for t in 1..=560u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.health.health = 100.0;
        sim.step_bond(Default::default(), false, false, false);
        for (k, &(c, _)) in STAGES.iter().enumerate() {
            if rose[k].is_none() && sim.drones.channels.on(c) { rose[k] = Some(t); }
        }
    }
    for (k, &(c, at)) in STAGES.iter().enumerate() {
        let t = rose[k].unwrap_or_else(|| panic!("ch {c:#x} never rose: {rose:?}"));
        assert!(t.abs_diff(at) <= 3, "ch {c:#x} rose at +{t}, the game's +{at} [L]: {rose:?}");
    }
}
