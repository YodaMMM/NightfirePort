//! unported-misc-5: `Check_Target`'s class-0x1D trigger arm — the exit aim hint (0x2000009) and
//! `Trigger_Locked` (0x1b2fc8) — through the real path (`nf_view::load` + `Sim::step_bond` inputs;
//! the position/aim fixture follows the merged door-text-labels pattern, a556ce10 — the hint, the
//! cooldown and the lock state themselves are never poked).
//!
//! Game rules [M]:
//! - `Check_Target` 0x1a7810 (every 6th tick) decrements `LoadMsg` (0x30cd08) on EVERY invocation
//!   (:78-79); the class-0x1D arm (:193-208) requires `Trigger_Type` (0x1b2fa8) == 10 — a
//!   LOAD_LEVEL exit — hit dist < 5.0, `LoadMsg == 0`, level != 0x7000009, and re-arms
//!   `LoadMsg = FRAME_RATE_INT * 0x14` = 1200 whether or not the hint shows (:195-196).
//! - The hint itself needs `!Trigger_Locked` (0x1b2fc8): for a type-10 trigger the key gate channel
//!   (ext+0x26) off; a non-10 trigger's lock channel (ext+0x24) on locks it. Unlocked →
//!   `Txt_BindLabel(0x2000009, 0)` for 0xb4 = 180 ticks (:201-207).
//! - The fixture exit: 07000005's LOAD_LEVEL exit carries real collision — `Player_Activate`
//!   0x1a5170's E pick (a collision sphere cast) reaches it there (routes b/c) [M], unlike 01's
//!   mesh-less #18 [D placements, exit_aim_01_live]. The same collision is what the aim ray hits.
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere (the ISO tests run under the suite's silent rule).

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn settle(scene: &mut nf_view::scene::Scene, cap: u32) {
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..cap {
        sim.step_bond(Default::default(), false, false, false);
        if !sim.disabled { break; }
    }
}

/// Yaw/pitch (the port's units: pitch is in π/2 turns) so `bond.look()` points at `target`.
fn face(sim: &mut nf_view::scene::Sim, target: [f32; 3]) {
    let head = sim.bond.head();
    let l = [target[0] - head[0], target[1] - head[1], target[2] - head[2]];
    let len = (l[0] * l[0] + l[1] * l[1] + l[2] * l[2]).sqrt();
    let l = [l[0] / len, l[1] / len, l[2] / len];
    sim.bond.yaw = l[0].atan2(l[2]);
    sim.bond.pitch = (l[1].asin()) / (std::f32::consts::FRAC_PI_2);
}

fn raised(sim: &nf_view::scene::Sim, label: u32) -> Vec<f32> {
    sim.hud.info_msgs.iter().filter(|m| m.label == label).map(|m| m.life).collect()
}

/// A collision-backed LOAD_LEVEL exit plus a clear vantage 2.5 units out, found through the real
/// scene: the four horizontal sides are probed with the world's own segment (the same occluder the
/// Check_Target ray uses), so the fixture stands in open air.
fn exit_fixture(sim: &nf_view::scene::Sim) -> (usize, [f32; 3]) {
    let exits: Vec<usize> = sim.drones.triggers.iter().enumerate()
        .filter(|(_, t)| t.kind == nf_game::triggers::kind::LOAD_LEVEL && !t.tris.is_empty())
        .map(|(i, _)| i).collect();
    assert!(!exits.is_empty(), "the level has a collision-backed LOAD_LEVEL exit (the zip-wire exit [M 0x1a5170 route c])");
    for &ti in &exits {
        let c = sim.drones.triggers[ti].centre();
        for d in [[1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]] {
            let feet = [c[0] + d[0] * 2.5, c[1], c[2] + d[1] * 2.5];
            let head = [feet[0], feet[1] + 1.6, feet[2]];
            if sim.world.segment(head, c).is_none() { return (ti, feet); }
        }
    }
    panic!("no open-air vantage for any exit of {:?}", exits);
}

/// The real path raises the 0x2000009 exit hint once: aim at the exit through `step_bond` inputs —
/// fails on the old stand-in (no trigger arm at all → no label ever) — and the sister asserts the
/// game's cooldown: `LoadMsg` re-arms to 1200 (invocations), so the label does NOT double-post while
/// it runs [M 0x1a7810:195-196].
#[test]
fn aiming_at_a_load_level_exit_raises_the_hint_once_and_arms_the_cooldown() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    settle(&mut scene, 3000);
    let (ti, feet) = exit_fixture(scene.sim.as_ref().unwrap());
    let sim = scene.sim.as_mut().unwrap();
    // Premise [M 0x1a7810:201-207 with 0x1b2fc8]: the fixture exit is UNLOCKED (its key gate is
    // off) — otherwise the game re-arms LoadMsg but shows nothing.
    assert!(!sim.drones.triggers[ti].locked(&sim.drones.channels),
        "fixture premise: the exit's key gate is off (unlocked)");
    sim.bond.pos = feet;
    face(sim, sim.drones.triggers[ti].centre());
    let mut hits = Vec::new();
    for k in 0..30 {
        sim.step_bond(Input::default(), false, false, false);
        hits = raised(sim, 0x200_0009);
        if !hits.is_empty() { break; }
    }
    assert_eq!(hits, vec![180.0], "the aim ray raised the 0x2000009 exit hint for 180 ticks — Text_AddMsg(…, 0xb4) [M 0x1a7810:201-207]; other info msgs {:?}",
        sim.hud.info_msgs.iter().map(|m| (m.label, m.life)).collect::<Vec<_>>());
    // Sister: the cooldown re-armed to 1200 invocations (60 * 0x14) [M 0x1a7810:196] and counts
    // down one per 6th-frame invocation [M 0x1a7810:78-79] — so it stays > 0 (no second raise).
    assert_eq!(sim.hud.load_msg, 1200, "LoadMsg re-armed to FRAME_RATE_INT * 0x14 [M 0x1a7810:196]");
    for _ in 0..60 {
        sim.step_bond(Input::default(), false, false, false);
        assert!(sim.hud.load_msg > 0, "the cooldown must outlast a 60-tick aim window [M 0x1a7810:78-79: one decrement per 6th frame]");
        assert_eq!(raised(sim, 0x200_0009).len(), 1, "no second raise while LoadMsg runs [M 0x1a7810:195]");
    }
}

/// `Trigger_Locked` 0x1b2fc8's two arms, pinned directly on the port's fields: a type-10 trigger
/// locks on its KEY GATE off (ext+0x26 [M 0x1b2fc8:14-22, the LoadLevelCreate-shifted channel]),
/// a non-10 trigger locks on its LOCK channel on (ext+0x24 [M 0x1b2fc8:24-29]), and a zero channel
/// never locks either arm. Sisters inside: each assertion has an on/off pair.
#[test]
fn trigger_locked_arms_match_the_decompile() {
    use nf_game::triggers::{Channels, Trigger, kind};
    let mk = |kind: u8, key_gate: u8, lock: u8| Trigger {
        placement: 0, kind, out: 0, mask: 2, list: Vec::new(), lock, hide: false, glist_0x10: true,
        filter_gate: 0, tris: Vec::new(), lo: [0.0; 3], hi: [0.0; 3], gone: false, script: None,
        next_level: 0x0700_0006, gate: 0, key_gate, success: 0, fallback: 0, movie_code: 0, movie_fired: false,
    };
    let mut ch = Channels::default();
    // Type 10: key gate set + channel OFF → locked [M 0x1b2fc8:14-22: `switch_channels[ch] == 0` → 1];
    // channel ON → not locked [M 0x1b2fc8:19-21]. The fresh channels are all OFF.
    assert!(mk(kind::LOAD_LEVEL, 5, 0).locked(&ch), "type-10: key gate off → locked [M 0x1b2fc8:14-22]");
    ch.set(5, 1, 0);
    assert!(!mk(kind::LOAD_LEVEL, 5, 0).locked(&ch), "type-10: key gate on → not locked [M 0x1b2fc8:19-21]");
    ch.set(5, 0, 0);
    assert!(!mk(kind::LOAD_LEVEL, 0, 0).locked(&ch), "type-10: zero key gate never locks [M 0x1b2fc8:15]");
    // Non-10: lock channel set + ON → locked [M 0x1b2fc8:24-29]; OFF → not locked.
    ch.set(7, 1, 0);
    assert!(mk(kind::TOUCH, 0, 7).locked(&ch), "non-10: lock channel on → locked [M 0x1b2fc8:24-29]");
    ch.set(7, 0, 0);
    assert!(!mk(kind::TOUCH, 0, 7).locked(&ch), "non-10: lock channel off → not locked [M 0x1b2fc8:27-29]");
    assert!(!mk(kind::TOUCH, 0, 0).locked(&ch), "non-10: zero lock channel never locks [M 0x1b2fc8:24]");
}
