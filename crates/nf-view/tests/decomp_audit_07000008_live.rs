//! decomp-audit-08 (996): the real-path proofs for the 07000008 audit rows.
//!
//! The F5 falling-to-death failure path, end to end: Bond walks off the gondola boarding
//! platform, the plunge TouchOnce #54 fires (ch 0x7C [D #54]), the death NIS SP #307 plays
//! 0x06000991, and the NIS's OWN stream writes the death channel — event 0xD params (62, 1)
//! at the script's tick 89, stream 6 [D the NIS stream, decoded by NF_SCRIPTLOG on the real
//! walk]. `Mission_Update` reads ch 0x62 [M 0x182c60:21] → Music_Event(6,1), MissionState 3,
//! the 1.0 s TimeOut, to_end 0x7000048 [M 0x182c60:35-64] → the 0x40000042 fail/death menu
//! [M 0x182c60:77-81]. The body holds through it, and that is the game's own law: the NIS's
//! event 4 disables the player [M Script_EventHandler 0x1d94c0:119 — `Player_Disable`, and
//! `Player_Update` 0x193600:80-89 returns on the 0 byte], the stream carries NO event 6 [D the
//! decode], so `Script_Free`'s enable — gated on the event-6 flag bit 0x100 [M 0x1da880:44-49]
//! — never runs. The y −24.1 hurt kill floor (docs/levels/07000008.md §7) serves falls the
//! plunge NIS does not catch; through THIS path the NIS scripts the death itself.
//!
//! Real input only: the walk-off is `Sim::step_bond` forward input from the platform (the same
//! spot the hurt test walks); no channel or state pokes anywhere.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

#[test]
fn the_f5_fall_death_nis_writes_0x62_and_opens_the_fail_menu() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000008.bin", false).expect("load 07000008");
    {
        let sim = scene.sim.as_mut().unwrap();
        // The headless sim's auto-restart answers the fail menu the tick it opens (drone.rs's
        // default + scene.rs's MenuOpen arm); the other death tests flip the flag to observe the
        // menu (fix_death_restart_live's pattern). The fall itself stays pure real input.
        sim.drones.auto_restart = false;
        for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
        for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
        // The boarding platform, facing down the cable chasm (the hurt test's walk-off spot).
        sim.bond.pos = [111.4, 24.5, -127.2];
        sim.bond.yaw = std::f32::consts::PI;
    }
    let sim = scene.sim.as_mut().unwrap();
    let mut plunged = false;
    let mut min_y = f32::MAX;
    for t in 1..=1600u32 {
        sim.step_bond(Input { forward: 1.0, ..Input::default() }, false, false, false);
        min_y = min_y.min(sim.bond.pos[1]);
        if sim.drones.channels.on(0x7c) { plunged = true; }
        if plunged && sim.drones.channels.on(0x62) { break; }
    }
    assert!(plunged, "the plunge TouchOnce fired on the walk-off [D #54]");
    assert!(sim.drones.channels.on(0x62), "ch 0x62 rose — the death NIS's own event 0xD (62,1) [D 0x06000991 s6 @t89]");
    assert!(min_y < 20.0, "the fall ran (min y {min_y:.2})");
    // The mission pass reads the channels before the scripts write them in the same tick
    // (drone.rs's fix-objective-banner ordering), so the death arm lands the tick after.
    for _ in 0..10 {
        if sim.drones.mission.state == 3 { break; }
        sim.step_bond(Input::default(), false, false, false);
    }
    assert!(sim.drones.mission.state == 3, "MissionState 3 [M 0x182c60:35-42's ch-0x62 arm]");
    // The 1.0 s TimeOut → the fail/death menu [M 0x182c60:58-64 → case 4's menu; the port's
    // fail_menu_open = mission.menu_open with no choice made yet].
    let mut menu = false;
    for k in 0..240 {
        sim.step_bond(Input::default(), false, false, false);
        if k % 30 == 0 { eprintln!("[f5-dbg] k {k} internal {} state {} menu {} ch62 {}", sim.drones.mission.internal, sim.drones.mission.state, sim.drones.mission.menu_open, sim.drones.channels.on(0x62)); }
        if sim.drones.fail_menu_open() { menu = true; break; }
    }
    assert!(menu, "the fail/death menu opens [M 0x182c60:77-81]");
    // The body holds through the whole sequence — the game's own disable law, not a port gap:
    // the NIS's event 4 disables the player [M 0x1d94c0:119], no event 6 in the stream [D],
    // so Script_Free's 0x100-gated enable never runs [M 0x1da880:44-49].
    let p0 = sim.bond.pos;
    let held = (0..120).all(|_| { let p = sim.bond.pos; sim.step_bond(Input::default(), false, false, false); p == sim.bond.pos });
    assert!(held, "the body holds through the menu (the disable law [M 0x1d94c0:119 + 0x1da880:44-49])");
    assert_eq!(p0, sim.bond.pos, "no drift while held");
    // The hurt kill floor stays silent through this path — the NIS scripts the death itself.
    let hurt_hits: u32 = sim.hurts.iter().map(|h| h.hits).sum();
    assert_eq!(hurt_hits, 0, "the held body never reaches the y −24.1 quads (the NIS path needs no kill floor)");
}
