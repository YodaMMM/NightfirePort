//! door-text-labels (G-035 remainder): the doors' two text labels through the real paths.
//!
//! The game's rules, all cited [M] (docs/systems/door-text-labels.md carries the full chain):
//! - `Check_Target` 0x1a7810 (every 6th tick): the 50-unit view ray's first hit object of class
//!   0x1c (a door) within 5.0 of the head shows `Txt_BindLabel(0x200000b, 0)` — "Press Action to
//!   open door" [D USATxt.dat] — through `Door_DoMsg` 0x135f18: once per door (the `+0x74` latch
//!   sets BEFORE the checks, so a failed attempt burns it), lock-channel-down doors fail (a locked
//!   door hints nothing), an OPEN (state 4) door fails, and the boot-global `DoorMsg` 0x30cd04
//!   gates at < 3 with no reset anywhere in ACTION.ELF (the full gp-relative scan). The raise is
//!   `DoorMsg++` + `Text_AddMsg(BL+0x94e, 0, 1, label, 0, 0xb4)` — the type-1 INFO bar, 180 ticks.
//! - `Door_Update` 0x1363c8: the lock channel's rising edge shows `Txt_BindLabel(0x2000003, 0)` —
//!   "Door Unlocked" [D USATxt.dat] — `Text_AddMsg(0, 0, 1, …, 0, 0xb4)`, 180 ticks, BEFORE the
//!   unlock `Door_Activate` + `+0x68 = 0` + busy marker; level-gated off 0x7000014/0x700004a.
//! - Neither label is on the E key: `Player_Activate` 0x1a5170's door arm has no text call [M].
//!
//! The port's `hud_tick` binds `info_labels` into `info_msgs` the same tick (`Txt_BindLabel` runs
//! at post time in the game too [M 0x196e80:41]); the assertions read `info_msgs`' `Msg.label` —
//! the port's provenance field for the game-label pins.
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

/// Aim at a closed, unlocked door → the 0x200000b "Press Action to open door" hint, once, 180
/// ticks, `DoorMsg` 1; the latch holds (no second raise while aiming). Sister inside: the raise
/// count stays 1 (a level-triggered read or a lost latch would fail it).
#[test]
fn aim_at_a_closed_unlocked_door_raises_the_press_action_hint_once() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso, "07000001.bin", false).expect("load 07000001");
    settle(&mut scene, 3000);
    let sim = scene.sim.as_mut().unwrap();
    // 07000001 door #0: a kind-2 slide door, closed, no lock channel, ungrouped [D placements —
    // the probe: ch_lock 0, ch_open 0, group 0, state 0x2, glist 0x20001e1, at (7.9, 5.4, 2.9) r 1.56].
    let di = 0;
    {
        let d = &sim.drones.doors.list[di];
        assert_eq!(d.ch_lock, 0, "the fixture door carries no lock channel");
        assert_eq!(d.state, nf_game::door::CLOSED, "the fixture door starts closed");
        assert!(!d.msg_shown, "the hint latch starts clear");
    }
    let c = sim.drones.doors.list[di].live_sph().0;
    // Stand ~2.5 units from the sphere centre, toward the spawn side (clear line).
    let to_spawn = [sim.bond.pos[0] - c[0], 0.0, sim.bond.pos[2] - c[2]];
    let len = (to_spawn[0] * to_spawn[0] + to_spawn[2] * to_spawn[2]).sqrt();
    sim.bond.pos = [c[0] - to_spawn[0] / len * 2.5, c[1], c[2] - to_spawn[2] / len * 2.5];
    face(sim, c);
    let mut hits = Vec::new();
    for _ in 0..12 {
        sim.step_bond(Input::default(), false, false, false);
        hits = raised(sim, 0x200_000b);
        if !hits.is_empty() { break; }
    }
    assert_eq!(hits, vec![180.0], "the aim ray raised the 0x200000b hint for 180 ticks — the game's Text_AddMsg(…, 0xb4) [M 0x1a7bbc sibling]; other info msgs {:?}",
        sim.hud.info_msgs.iter().map(|m| (m.label, m.life)).collect::<Vec<_>>());
    assert_eq!(sim.hud.door_msg, 1, "DoorMsg counts the raise [M 0x1a7bc0]");
    assert!(sim.drones.doors.list[di].msg_shown, "the +0x74 latch set [M 0x135f18:17]");
    // Keep aiming: the latch holds — one hint per door.
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    assert_eq!(raised(sim, 0x200_000b), vec![180.0], "no second hint while aiming (the latch)");
    assert_eq!(sim.hud.door_msg, 1, "DoorMsg stayed 1");
}

/// Sister: aim at a LOCKED door (its lock channel wired and down) → no 0x200000b hint at all, and
/// the one attempt burns the door's latch [M 0x135f18:16-24]. 07000007 door #11 stands within the
/// 5-unit gate of the spawn [D placements: ch_lock 200, closed, kind 0 at (31.9, 43.0, −69.3)].
#[test]
fn a_locked_door_raises_no_hint_and_burns_its_latch() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07000007");
    settle(&mut scene, 3000);
    let sim = scene.sim.as_mut().unwrap();
    let di = 11;
    {
        let d = &sim.drones.doors.list[di];
        assert_eq!(d.ch_lock, 200, "the fixture door is lock-channelled");
        assert_eq!(d.state, nf_game::door::CLOSED, "the fixture door starts closed");
    }
    assert!(!sim.drones.channels.on(200), "the fixture's lock channel is down at the aim (locked)");
    let c = sim.drones.doors.list[di].live_sph().0;
    let d2 = (c[0] - sim.bond.pos[0]).powi(2) + (c[1] - sim.bond.pos[1]).powi(2) + (c[2] - sim.bond.pos[2]).powi(2);
    assert!(d2.sqrt() < 5.0, "the fixture door stands within the 5-unit gate (its centre is {:.2} away)", d2.sqrt());
    face(sim, c);
    for _ in 0..12 { sim.step_bond(Input::default(), false, false, false); }
    assert!(raised(sim, 0x200_000b).is_empty(), "a locked door raises no hint — info msgs {:?}",
        sim.hud.info_msgs.iter().map(|m| (m.label, m.life)).collect::<Vec<_>>());
    assert!(sim.drones.doors.list[di].msg_shown, "the attempt burned the +0x74 latch [M 0x135f18:16-17]");
    assert_eq!(sim.hud.door_msg, 0, "DoorMsg only counts successes [M 0x1a7ba8-0x1a7bc0: the increment follows the nonzero return]");
}

/// The lock channel's rising edge raises 0x2000003 "Door Unlocked" (180 ticks) and clears the
/// door's lock channel for good [M 0x1363c8:101-107]; holding the channel raises nothing more
/// (sister: the edge, not the level, decides).
#[test]
fn the_lock_channel_rising_edge_raises_door_unlocked_once() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07000007");
    settle(&mut scene, 3000);
    let sim = scene.sim.as_mut().unwrap();
    // Face away from the door pair: the aim ray must stay off them (the unlock arm is the door
    // tick's, not the aim ray's).
    sim.bond.yaw = 0.0;
    sim.bond.pitch = 0.0;
    let ch = sim.drones.doors.list[11].ch_lock;
    assert_eq!(ch, 200, "the fixture doors share lock channel 200 [D placements]");
    let wired = sim.drones.doors.list.iter().filter(|d| d.ch_lock == ch).count();
    assert!(wired >= 2, "the fixture has the paired doors on the channel");
    assert!(!sim.drones.channels.on(u32::from(ch)), "the channel starts down (no edge yet)");
    // The real channel path (the game's switch/script write — `switch_channels[ch] = 1`).
    sim.drones.channels.set(u32::from(ch), 1, sim.drones.frame);
    sim.step_bond(Input::default(), false, false, false);
    let times = raised(sim, 0x200_0003);
    assert_eq!(times.len(), wired, "every rising-edge door raised 'Door Unlocked' once — info msgs {:?}",
        sim.hud.info_msgs.iter().map(|m| (m.label, m.life)).collect::<Vec<_>>());
    assert!(times.iter().all(|t| *t == 180.0), "the message time is the game's 0xb4 [M 0x1363c8:103]");
    // The unlock cleared every wired door's channel field [M 0x1363c8:106] — for good.
    assert!(sim.drones.doors.list.iter().all(|d| d.ch_lock != ch), "the +0x68 field is cleared on every rising-edge door");
    // Sister: holding the channel on raises nothing more.
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    assert_eq!(raised(sim, 0x200_0003).len(), wired, "no second 'Door Unlocked' while the channel stays on");
}
