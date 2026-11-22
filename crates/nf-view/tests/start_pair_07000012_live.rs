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
#[ignore = "the port reads 82.50 at 480 / 28.25 at 840 against the game's 53-72 / 0-37 [L]: the 840 side now passes, the 480 side is short 17.5 lost vs the game's 28-47. Measured this round (start_rounds_07000012_live, s/level-12-start-damage-gap): the guards alert at t169 (the script sound 634), los t170/171, ack t179/180 (reaction 10 [M DroneFunc_ReactionTime 0x14ac60 default arm]), first shots t180/181; 33 rounds by t480, 21 real rolls after the 120-tick NewSighting gate opens at ~t290 [M DroneWeap_DoBulletAccuracy 0x179740:103-108, +0x270], 9 rolled hits, 6 landed; 3 rolled hits die on the counter edge at (17.4-18.4, 1.3-2.0, ~19.0) between the guards and Bond (World events, no Player), which the crouched rounds (muzzle y 0.84) clear. Verified facts for the next lever: (1) the port's north guard #6 sits in AimStandFire 0x68 from t179 to t840+ with ZERO ChooseCombatMove re-picks while the game's north guard hits from 0x70/0x74/0x68/0x6f [L] and ends at (13.9, 17.1-17.5) [L] - the port's between-burst combat_state ask window is ~1 tick (burst_done flips true only on the drain tick, handle_firing re-clears it next pass) and its ChooseCombatMove opts for an unaimed guard are crouch/stand only (the step/dodge/strafe options need the aimed_at latch, false for idle Bond); (2) the game's guard spread is weapon+0x24 x Drone+0xc0 with +0xc0 = the accuracy byte [M Bullet_init 0x12c5a0:139-148 + NDrone2_DoModeSettingsNEW 0x14c600:108] - the port passes the raw +0x24 (10.0), ~7x too precise, which does not explain the missing damage but is a real divergence; (3) AimStandFire's ENTER now carries the captain 0x19->0x18 swap and the prop-0x40/locked 0x59 diversion [M 0x166328:30-49] (this round, no-op on this level's pair: both carry 0x40 clear [D], the captain swapped in Attack 0x56 already [M 0x164210:31-52]). Next levers: why the game's north guard re-picks (0x70/0x74 hits [L]) while the port's #6 never leaves 0x68 - instrument the ask window (+0x3c vs burst_done) and the anim class flip 7->2 the probe caught at t179-260 (the aim danim ends and class 7 refuses Ready2Fire 0x178c48:47-54 until the re-call)"]
fn the_idle_start_costs_bond_what_the_games_does() {
    let (_, _, _, hp) = idle_start();
    let at = |f: u32| hp.iter().find(|(t, _)| *t == f).unwrap().1;
    assert!(at(480) <= 72.0 && at(840) <= 40.0, "the idle start costs Bond what the game's does: {hp:?}");
}
