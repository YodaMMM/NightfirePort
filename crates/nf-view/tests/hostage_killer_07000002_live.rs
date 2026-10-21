//! Review of unported-guard-ai-other-3b (2026-10-21): the HostageKiller pairing and execution on
//! 07000002, the level that places them (placement mode param 4 → +0xC5 = 4 via
//! `NDrone2_GetDTYPENEW` 0x14c938's `switch(param_3)` case 4 [M]; census: k16 = 4 on 07000002 ×2 and
//! 07000003 ×2 only — not 07000012, whose dtype-4 placements carry mode param 0 [D drone-props]).
//! - Pairing `NDrone2_SetupHostageKiller` 0x144030: the nearest mode-5 drone with +0x2B0 == 0 within
//!   5.0, BOTH links set (killer +0x2B0 = hostage, hostage +0x2B0 = killer) [M :22-59].
//! - Execution: HostageKillerAttack 0x15c940 ENTER arms timer 10 at FRAME_RATE; case 10 calls the
//!   execution anim 0x58 with done-msg 5 (CallAnim a3 = 5 → +0x598 [disasm 0x15cab8; M 0x13ab58:100]);
//!   case 5 sends msg 0xb (delay 10) to the hostage [M :61-66], whose case 0xb enters HostageDie 0xb
//!   [M 0x15cb78], and the killer attacks (0x56).

#[path = "common/walker.rs"]
mod walker;

use nf_game::drone::state;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_killer_pairs_both_ways_and_his_execution_kills_the_hostage() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000002.bin", false).expect("load 07000002"));
    w.wait(120);
    let (ki, hi) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let ki = sim.drones.list.iter().position(|d| d.mode == 4 && (d.pos[0] + 45.6).abs() < 0.5 && (d.pos[2] - 10.9).abs() < 0.5).expect("the 0x17 killer [D]");
        let hi = sim.drones.list.iter().position(|d| d.mode == 5 && (d.pos[0] + 45.5).abs() < 0.5 && (d.pos[2] - 12.2).abs() < 0.5).expect("geisha 1 [D]");
        (ki, hi)
    };
    { let sim = w.scene.sim.as_mut().unwrap(); sim.drones.channels.set(0x17, 1, sim.drones.frame); }
    w.wait(60);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let k = &sim.drones.list[ki];
        assert_eq!(k.state, state::HOSTAGE_KILLER, "mode 4 starts in HostageKiller 8 [M 0x14cbd8 case 4]");
        assert_eq!(k.hostage_target, Some(hi), "the killer links the nearest mode-5 drone within 5.0 [M 0x144030]");
        assert_eq!(sim.drones.list[hi].captor, Some(ki), "and the hostage links back to HIM [M 0x144030:58]");
    }
    // The execution chain from HostageKillerAttack (the Global timer-10 route into state 9 is the
    // game's; entered directly here — the subject is the 9 → anim 0x58 → msg 5 → hostage 0xb chain).
    { let sim = w.scene.sim.as_mut().unwrap(); sim.drones.list[ki].set_state(state::HOSTAGE_KILLER_ATTACK); }
    let mut died = None;
    for t in 0..600u32 {
        w.wait(1);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.list[hi].state == state::HOSTAGE_DIE || sim.drones.list[hi].state == state::HOSTAGE_DEAD { died = Some(t); break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(died.is_some(), "the execution anim's done-msg 5 sends msg 0xb → HostageDie — hostage state {:#x}, killer {:#x}",
        sim.drones.list[hi].state, sim.drones.list[ki].state);
    // Case 5 then sets Attack 0x56 (whose ENTER routes on at once — so the state LOG holds it).
    assert!(sim.drones.list[ki].log.iter().any(|(_, s)| *s == state::ATTACK), "case 5 then attacks [M 0x15c940 case 5]");
}

/// Sister: a mode-5 drone farther than 5.0 is never paired — the other killer (0x19's, at
/// (−11.2, −37.8)) takes geisha 2 only, never geisha 1 [M 0x144030 `fVar9 < 5.0`].
#[test]
fn a_hostage_beyond_5_units_is_not_paired() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000002.bin", false).expect("load 07000002"));
    w.wait(120);
    { let sim = w.scene.sim.as_mut().unwrap(); sim.drones.channels.set(0x19, 1, sim.drones.frame); }
    w.wait(60);
    let sim = w.scene.sim.as_ref().unwrap();
    let k2 = sim.drones.list.iter().position(|d| d.mode == 4 && (d.pos[0] + 11.2).abs() < 0.5).expect("the 0x19 killer [D]");
    let g1 = sim.drones.list.iter().position(|d| d.mode == 5 && (d.pos[0] + 45.5).abs() < 0.5).expect("geisha 1");
    assert_ne!(sim.drones.list[k2].hostage_target, Some(g1), "geisha 1 is ~60 units away — never his");
    assert!(sim.drones.list[k2].hostage_target.is_some(), "he pairs geisha 2 within 5.0");
}
