//! ai-cover-anims: the Hostage case-5 arm through the real message path. `NDrone2_DSTATE_Hostage`
//! (0x15cb78) case 5 (msg 5 = ANIM_DONE) is `DroneAnim_SetDAnimNow` (0x13dd78) of danim 0xf4 with
//! blend 0 [M 0x15cb78:38-41]: the completion action clears (+0x596/+0x598), `DroneAnim_SetDAnimInternal`
//! runs, and a started script marks the call reached (+0x57B). The 07000002 geishas are the ported
//! Hostage drones (mode 5 → `DroneTypeSettings[5].start` = 0x0A [L 0x29aee8+5·12]); no ported sender
//! routes msg 5 to a hostage (the ENTER call's completion message is 0 [M 0x15cb78 case 1]), so the
//! test feeds msg 5 through the drone's real delayed-message queue — the same `Drones`-tick `send`
//! route the patrol idle timeout rides — the way the ElevatorJumper tests drive their writer-less
//! state. Sisters: the pre-delivery danim is NOT 0xf4 (fails if the ENTER pose changes under it),
//! the state stays Hostage (fails on a misroute), and a fresh script restart (time <= 2, only when
//! a script exists for 0xf4 — without one the game's `SetDAnimInternal` returns 0 and nothing
//! restarts [M 0x13aee8 tail]) fails if the arm never ran.

#[path = "common/walker.rs"]
mod walker;

use nf_game::drone::{msg, state};
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn geisha(sim: &nf_view::scene::Sim, ch: u8) -> Option<usize> {
    sim.drones.list.iter().position(|d| d.second_channel == ch)
}

#[test]
fn the_hostage_anim_done_arm_reasserts_danim_f4() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000002.bin", false).expect("load 07000002"));
    w.wait(120);
    let sim = w.scene.sim.as_ref().unwrap();
    let gi = geisha(sim, 0x49).expect("geisha 1 (param 6 = 0x49) on the disc");
    drop(sim);
    // Enable her through her plate channel (tr[003] out 0x17 [D]) — the proven enable path.
    let sim = w.scene.sim.as_mut().unwrap();
    sim.drones.channels.set(0x17, 1, sim.drones.frame);
    drop(sim);
    w.wait(60);
    let sim = w.scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.list[gi].state, state::HOSTAGE,
        "the enabled geisha sits in Hostage 0x0A [L DroneTypeSettings[5].start]");
    // Sister: the ENTER pose is playing — 0xf4 is the case-5 re-assert danim, not the ENTER one.
    let before = sim.drones.list[gi].anim.danim;
    drop(sim);
    assert_ne!(before, 0xf4,
        "the ENTER pose danim is not 0xf4 [M 0x15cb78 case 1 CallAnim(0x21) vs case 5 SetDAnimNow(0xf4)]");
    // Feed msg 5 through the real delayed-message route (the Drones tick's send → the state switch).
    let sim = w.scene.sim.as_mut().unwrap();
    let at = sim.drones.frame + 1;
    sim.drones.list[gi].delayed = Some((at, msg::ANIM_DONE));
    drop(sim);
    w.wait(3);
    let sim = w.scene.sim.as_ref().unwrap();
    let g = &sim.drones.list[gi];
    assert_eq!(g.state, state::HOSTAGE, "case 5 keeps the state [M 0x15cb78:38-41]");
    assert_eq!(g.anim.danim, 0xf4,
        "SetDAnimNow(0xf4, 0, 0) re-asserts danim 0xf4 [M 0x15cb78:39 → 0x13dd78 → +0x56C = 0xf4]");
    if g.anim.script.is_some() {
        assert!(g.anim.time <= 2,
            "the re-asserted danim restarted fresh (a started script marks +0x57B) [M 0x13dd78], got time {}",
            g.anim.time);
    }
}
