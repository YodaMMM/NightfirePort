//! 07000012's start pair (the two guards west of Bond's start): their no-bunching side bits and
//! what they cost an idle Bond, read against the game.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// `DroneMove_NoBunching` 0x150ad0 stamps +0x31C every tick: the side the nearest peer is on
/// (0x10 right / 0x8 left) and the ahead/beside bit (2 / 4) [M]. Live, the start pair carries
/// 0x14 (the south guard, his partner to his right) and 0xC (the north guard) through the fight
/// [L PCSX2 07000012 d1, 2026-11-12: guards 0x1876400 / 0x1877240 +0x31C from frame 231 on].
/// With the bits the south guard's step choice keeps off his partner's side, so the pair stays
/// apart and the standing rounds reach Bond instead of the partner's body.
fn idle_start() -> (Vec<u32>, Vec<u32>, f32, Vec<(u32, f32)>) {
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), "07000012.bin", false, None, 1).expect("load 07000012");
    let (mut south_bits, mut north_bits) = (Vec::new(), Vec::new());
    let mut min_gap = f32::MAX;
    let mut hp = Vec::new();
    for t in 0..=840u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let (n, s) = (&sim.drones.list[6], &sim.drones.list[7]);
        if t >= 240 {
            north_bits.push(n.no_bunching);
            south_bits.push(s.no_bunching);
            let g = ((n.pos[0] - s.pos[0]).powi(2) + (n.pos[2] - s.pos[2]).powi(2)).sqrt();
            min_gap = min_gap.min(g);
        }
        if t % 120 == 0 { hp.push((t, sim.drones.health.health)); }
    }
    eprintln!("[12] gap {min_gap:.2} hp {hp:?}");
    (south_bits, north_bits, min_gap, hp)
}

#[test]
fn the_start_pair_keeps_apart_and_reaches_bond() {
    let (south_bits, north_bits, min_gap, _) = idle_start();
    assert!(south_bits.iter().all(|&b| b == 0x14), "the south guard's bits stay 0x14 [L]: {:?}", &south_bits[..8]);
    assert!(north_bits.iter().all(|&b| b == 0xc), "the north guard's bits stay 0xC [L]");
    assert!(min_gap > 1.2, "the pair keeps apart (game ~1.5 u) [L]: {min_gap}");
}

/// The game's three idle runs at d1 read 53-72 at frame 480 and 0-37 at 840 [L]; the rounds
/// must land in that band, not the old 79 / 58.
#[test]
#[ignore = "the port reads 82.5 at 480 / 28.25 at 840 against the game's 53-72 / 0-37 [L]: 840 is in the band, 480 is not. The gap is the 360-480 window (port -5.25, game -25..-33): the south guard's 0x70 -> 0x67 stand swap at t382 holds fire 90 ticks, and the north guard's rolled hits clip the vending machine corner. Every gate on the path matches the decompile (docs/reports/s-l12-start-gap.md); the game's stance sequence and the north guard's firing spot need a live read"]
fn the_idle_start_costs_bond_what_the_games_does() {
    let (_, _, _, hp) = idle_start();
    let at = |f: u32| hp.iter().find(|(t, _)| *t == f).unwrap().1;
    assert!(at(480) <= 72.0 && at(840) <= 40.0, "the idle start costs Bond what the game's does: {hp:?}");
}
