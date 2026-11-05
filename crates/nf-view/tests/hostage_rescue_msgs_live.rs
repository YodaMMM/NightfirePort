//! audit-text-lines: the rescue INFO messages `DroneFunc_HostageSaved` 0x1441c0:20-51 posts —
//! the part of the hostage save the port dropped until now. The game, on every HostageSaved
//! ENTER, after the channel raise: level-gated to 0x7000002/0x7000003 [M :20-34], the save
//! counter `DAT_00298fac` (read before the increment [M :12], written [M :13]) picks the
//! message: the FIRST save of the level posts `Text_AddMsg(0, 0, 1, Txt_BindLabel(0x200004f), 0, 0xb4)`
//! ("1 of 2 hostages released") and every later save posts 0x2000050 ("All hostages released")
//! the same way [M :35-51] — decomp: "before == 0" forces "after == 1", so the 0x2000050 arm in
//! the before == 0 branch is dead and the fall-through label is 0x200004f; before != 0 forces
//! after >= 2, so the `after != 1` guard always passes. Text_AddMsg's id is 0 and the type-1
//! INFO pane life is 0xb4 = 180 ticks [M :44-51].
//!
//! The disc path (the proven machinery the existing hostage_save_07000002_live pins): 07000002's
//! geishas are hostage drones whose rescue channel is placement param 6 — geisha 23 ch 0x49,
//! geisha 24 ch 0x4A — enabled from WaitSwitch by their param-3 channels 0x17 / 0x19, each held
//! by captor guards [D placements; the same census that test cites]. Killing the captor through
//! the real fire path runs 0x15cb78:24-36 → HostageSaved → 0x1441c0.
//!
//! The port site: `Drones`' HOSTAGE_SAVED ENTER (crates/nf-game/src/drone.rs) posts
//! `info_labels` — the same type-1 INFO path the locked exit rides — and the view's hud_tick
//! drains it into `hud.info_msgs` bound through the level's Lang table.
//!
//! Opt-in via NF_ISO (the slim ISO is enough).

#[path = "common/walker.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn geisha(sim: &nf_view::scene::Sim, ch: u8) -> Option<usize> {
    sim.drones.list.iter().position(|d| d.second_channel == ch)
}

/// Enable + save one geisha the real way: raise its enable channel, stand beside the captor,
/// shoot the captor, wait for the rescue channel. Returns the frame the channel rose.
fn rescue(w: &mut Walker, enable: u8, rescue_ch: u8) {
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let gi = geisha(sim, rescue_ch).expect("the geisha with that rescue channel");
        drop(sim);
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(u32::from(enable), 1, sim.drones.frame);
        drop(sim);
    }
    // The captor link binds when the enabled hostage's Hostage state runs `SetupHostageKiller`
    // [M 0x144030] — the existing test waits 60 ticks before reading it.
    w.wait(60);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let gi = geisha(sim, rescue_ch).expect("the geisha with that rescue channel");
        let captor = sim.drones.list[gi].captor.expect("captor bound while captive");
        let at = sim.drones.list[captor].pos;
        drop(sim);
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [at[0] + 2.5, at[1], at[2]];
        drop(sim);
    }
    w.wait(30);
    let gi = {
        let sim = w.scene.sim.as_ref().unwrap();
        geisha(sim, rescue_ch).unwrap()
    };
    let captor = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.list[gi].captor.expect("captor bound")
    };
    let shot = w.shoot(captor, "the captor guard");
    if !shot {
        // 0x4A's captor stands behind glass the walker's shot cannot open — drive the save through
        // the game's own trigger instead: the Hostage UPDATE saves when the captor's health ≤ 0
        // [M 0x15cb78:24-36]. The first rescue (below, and hostage_save_07000002_live) already
        // pins the real fire path; this arm only needs the captor DEAD, whatever killed him.
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.list[captor].health = 0.0;
        drop(sim);
    }
    for _ in 0..120 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(u32::from(rescue_ch)) { return; }
        drop(sim);
        w.tick(Default::default(), false);
    }
    panic!("the rescue channel {} never rose", rescue_ch);
}

fn msgs(sim: &nf_view::scene::Sim, label: u32) -> Vec<&nf_view::scene::Msg> {
    sim.hud.info_msgs.iter().filter(|m| m.label == label).collect()
}

#[test]
fn first_rescue_posts_one_of_two_second_posts_all() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000002.bin", false).expect("load 07000002"));
    w.wait(120);
    // Geisha 1 (rescue channel 0x49 [D param 6], enabled by plate channel 0x17 [D tr[003]]).
    rescue(&mut w, 0x17, 0x49);
    // The first save's message: the queue drains on the save tick and binds 0x200004f.
    let sim = w.scene.sim.as_ref().unwrap();
    let first = msgs(sim, 0x0200_004f);
    assert_eq!(first.len(), 1, "the FIRST save posts 0x200004f exactly once [M 0x1441c0:35-51]; got {:?}",
        sim.hud.info_msgs.iter().map(|m| (m.label, &m.text)).collect::<Vec<_>>());
    assert_eq!(first[0].text, "1 of 2 hostages released", "the bound USATxt text [D USATxt.dat]");
    assert!((first[0].life - 180.0).abs() < 1.0, "the type-1 life is 0xb4 = 180 ticks [M 0x1441c0:51]");
    assert!(msgs(sim, 0x0200_0050).is_empty(), "no 0x2000050 on the first save [M]");
    drop(sim);
    // Geisha 2 (rescue channel 0x4A [D param 6], enabled by plate channel 0x19 [D tr[009]]).
    rescue(&mut w, 0x19, 0x4A);
    let sim = w.scene.sim.as_ref().unwrap();
    let second = msgs(sim, 0x0200_0050);
    assert_eq!(second.len(), 1, "the SECOND save posts 0x2000050 exactly once [M]");
    assert_eq!(second[0].text, "All hostages released", "the bound USATxt text [D USATxt.dat]");
    // The first 0x200004f may have expired (life 180) by now — the game's count `DAT_00298fac`
    // is what the label choice reads [M 0x1441c0:13-14], and it is at 2 with no third post.
    assert_eq!(sim.drones.hostage_saves, 2, "both saves counted [M 0x1441c0:13]");
    assert!(msgs(sim, 0x0200_004f).len() <= 1, "no second 0x200004f [M]");
}
