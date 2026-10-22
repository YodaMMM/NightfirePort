//! unported-guard-ai-other-3 (856): 07000007's prop-0x1A guard (skin 0x13 at (110.2, 44.1, -60.8)
//! [D placement words; switch channel param 3 = 0x93]) wakes when Bond crosses the doorway touch
//! #455 (kind 1, mask 8 = the player, out 0x93, (102.8..107.9, 43.2..48.0, -58.4..-57.6) [D]) and,
//! on his first sight, takes `NDrone2_DSTATE_Interogate` 0x161a40 [M `DroneFunc_FirstSightState`
//! 0x147c48:92-100 -> 0x3c] — not Combat. The interrogation latches the party
//! (`DroneVision_BeginInterogate` 0x1782a8: DAT_0029aa86 = 1), and on 07000007 the timer beats
//! run 60 -> (anim 0x0A + line 0x51) -> 180 -> Attack [M 0x161a40 case 0xc].

#[path = "common/walker.rs"]
mod walker;

use nf_game::drone::state;
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn guard(sim: &nf_view::scene::Sim) -> usize {
    sim.drones.list.iter().position(|d| d.skin == 0x0500_0013 && (d.pos[0] - 110.16).abs() < 1.0 && (d.pos[2] + 60.78).abs() < 1.0)
        .expect("the prop-0x1A guard (skin 0x13 at (110.2, 44.1, -60.8))")
}

/// Bond walks south through the doorway touch (-z, toward the guard), then either stands
/// (`draw = false`) or draws his gun 30 ticks in (`draw = true`). Returns (tick the guard entered 0x3c,
/// tick he left it, the state he left into, the party latch after entering, ch 0x93 seen).
fn run(draw: bool) -> (Option<u32>, Option<u32>, u8, bool, bool) {
    let mut w = walker::Walker::new(nf_view::load(&iso(), "07000007.bin", false).expect("load 07000007"));
    w.wait_enabled(6000, "level start");
    w.wait(60);
    let g = guard(w.scene.sim.as_ref().unwrap());
    {
        let sim = w.scene.sim.as_mut().unwrap();
        assert!(!sim.drones.list[g].enabled, "sister: the guard sleeps until ch 0x93");
        sim.bond = nf_game::player::Bond::new([105.3, 44.2, -55.0], std::f32::consts::PI);
    }
    let (mut entered, mut left, mut into, mut latch, mut ch93) = (None, None, 0u8, false, false);
    for t in 0..1200u32 {
        let forward = match entered { None => if t < 400 { 0.6 } else { 0.0 }, Some(_) => 0.0 };
        // Drawing: 30 ticks into the interrogation Bond selects the P99 he carries (weapon 2 — the
        // party start holds the fists, 0x1) through the weapon-select request.
        if draw && entered.is_some_and(|e| t == e + 30) {
            let sim = w.scene.sim.as_mut().unwrap();
            assert!(sim.drones.inventory.owned[2], "Bond carries the P99 on 07");
            sim.gun.switch_request = 2;
        }
        w.tick(Input { forward, ..Default::default() }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        ch93 |= sim.drones.channels.on(0x93);
        let st = sim.drones.list[g].state;
        if entered.is_none() && st == state::INTEROGATE { entered = Some(t); }
        if entered.is_some() { latch |= sim.drones.party_loud; }
        if entered.is_some() && left.is_none() && st != state::INTEROGATE { left = Some(t); into = st; break; }
    }
    (entered, left, into, latch, ch93)
}

#[test]
fn the_doorway_guard_interrogates_then_attacks_on_the_games_beats() {
    let (entered, left, into, latch, ch93) = run(false);
    eprintln!("entered {entered:?} left {left:?} into {into:#x} latch {latch} ch93 {ch93}");
    assert!(ch93, "Bond's walk through the doorway raises ch 0x93 (touch #455 [D])");
    let e = entered.expect("the woken guard's first sight takes Interogate 0x3c [M 0x147c48:92-100]");
    assert!(latch, "BeginInterogate latches the party (DAT_0029aa86 = 1) [M 0x1782a8]");
    let l = left.expect("the interrogation ends");
    // Timer C at +60, then +180 -> Attack [M 0x161a40 case 1 (+60) and case 0xc step 0 (+180), step 1].
    assert!((238..=242).contains(&(l - e)), "Bond standing still: Attack on the 60 + 180 beats, got {} ticks", l - e);
    assert!(!matches!(into, state::INTEROGATE | state::CIVILIAN_GUARD), "the guard turns hostile, got {into:#x}");
}

/// Sister: Bond unarmed when the interrogation began (`+0x228 & 0x10` clear) who then draws a gun
/// breaks it at once: `PlaySFX(0x4a)` + Attack [M 0x161a40 case 3, LAB_001621d0].
#[test]
fn drawing_a_gun_ends_the_interrogation_early() {
    let (entered, left, into, _latch, _ch93) = run(true);
    let e = entered.expect("Interogate entered");
    let l = left.expect("the interrogation ends");
    assert!(l - e < 120, "drawing the gun breaks the interrogation before the timer beats ({} ticks)", l - e);
    assert!(!matches!(into, state::INTEROGATE | state::CIVILIAN_GUARD), "the guard turns hostile, got {into:#x}");
}
