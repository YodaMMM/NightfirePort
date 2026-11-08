//! The security room's second keyboard on 0700000a: `SCRIPTPLAYER_KeyboardLasers` #643 (ch 0x3b, SSys
//! item 2 [D placement param 6]), on a run that really arrived from 07000009.
//!
//! The chain, each link [M]: `Player_Activate` 0x1a5170:121-122 → `SP_Activate` 0x1dc338:44-53 →
//! `Switch_Activate` 0x1b02a8 toggles ch 0x3b (its gate ch 0x3c reads low: nothing writes it [D]);
//! `SSys_Monitor` 0x1ad998:25-38 then sees item 2 armed-and-waiting with item 1 still holding 09's
//! laser channel 0x37 (`SSys_RamLoad` 0x1b0208 carried it), so it writes `switch_channels[0x37] = 1`,
//! retires item 1 and waits out the 8 s `SSysDelayTimer`; ch 0x37 is the lock of beams #606/#607/#621
//! and the second input of the door-beam ORs #632-635 [D triggers], so the whole web drops.
//!
//! Red on the old tree: `load_with_carry` planted 09's item-2 record (channel 0xff) over the
//! `Switch_Create` store, so on the real 09 → 0a path the monitor skipped item 2 and the keyboard
//! changed nothing but its own channel.

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// 07000009 then its real exit save, into a warm 0700000a — the campaign's own hand-over.
fn arrive_from_09() -> nf_view::scene::Scene {
    let a = nf_view::load_with_carry(&iso(), "07000009.bin", false, None).expect("load 09");
    let save = nf_view::warm_exit_save(a.sim.as_ref().unwrap());
    nf_view::load_with_carry(&iso(), "0700000a.bin", false, Some(&save)).expect("load 0a warm")
}

/// Stand at `pos` facing +z and tap the use key.
fn press(sim: &mut nf_view::scene::Sim, pos: [f32; 3]) {
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, pos, 0.0);
    for _ in 0..40 { sim.step_bond(Input::default(), false, false, false); }
    for i in 0..4 { sim.step_bond(Input::default(), false, false, i == 1); }
}

/// The keyboard on a run from 09: the web's lock channel comes up and the item-2 countdown arms
/// after the 8 s delay, with the 09 laser item retired.
#[test]
fn the_keyboard_drops_the_laser_web_on_a_run_from_09() {
    let mut scene = arrive_from_09();
    let sim = scene.sim.as_mut().unwrap();
    assert_eq!(sim.drones.ssys.items[2].channel, 0x3b,
        "#643's Switch_Create store survives the warm load [M 0x1adcc0:29-31 + 0x1b0208:13]");
    assert_eq!(sim.drones.ssys.items[1].channel, 0x37, "09's laser item rides the block [M 0x1b0208]");
    assert!(!sim.drones.channels.on(0x37), "the web is armed on arrival");
    // In the security room, in front of the keyboard's art (sphere (1.12, 1.33, 8.48) r 0.66 [D]).
    press(sim, [1.1, 1.2, 7.2]);
    assert!(sim.drones.channels.on(0x3b), "the use key reaches the laser keyboard from the room");
    for _ in 0..10 { sim.step_bond(Input::default(), false, false, false); }
    assert!(sim.drones.channels.on(0x37), "SSys_Monitor writes the web's lock channel [M 0x1ad998:25-38]");
    assert_eq!(sim.drones.ssys.items[1].channel, 0xff, "item 1 retires [M 0x1ad998]");
    assert_eq!(sim.drones.ssys.items[2].state, 0, "item 2 waits out the delay");
    for _ in 0..(8 * 60 + 30) { sim.step_bond(Input::default(), false, false, false); }
    assert_eq!(sim.drones.ssys.items[2].state, 6, "after SSysDelayTimer item 2 arms its countdown [M 0x1ad998]");
    assert!(sim.drones.channels.on(0x14) && sim.drones.channels.on(0x16) && sim.drones.channels.on(0x19) && sim.drones.channels.on(0x1b),
        "the door-beam ORs #632-635 follow ch 0x37 [D]");
}

/// Sister: no press, nothing moves.
#[test]
fn without_the_keyboard_the_web_stays_armed() {
    let mut scene = arrive_from_09();
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..120 { sim.step_bond(Input::default(), false, false, false); }
    assert!(!sim.drones.channels.on(0x37) && !sim.drones.channels.on(0x3b));
    assert_eq!(sim.drones.ssys.items[1].channel, 0x37);
}
