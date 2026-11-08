//! When a dying guard's death channel lands. `NDrone2_DSTATE_Death_Anim` 0x1637f8 raises it from
//! msg 2, the state EXIT (`Drone_SM_RouteMsgDCV` 0x171df8), so it rides the fall anim's end, never
//! the first update. 07000002 alone also arms timer C at +90 ticks in the ENTER [M :55-61], and
//! case 0xC raises the channel there. Case 3 removes a still-falling guard whose channel is
//! 0x88/0x29/0x2a once ch 0x35 is on [M :76-90]. All through the real fire path on the real level.

#[path = "common/walker.rs"]
mod walker;

use nf_game::drone::state;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Wake the guard with death channel `ch` by his enable channel, stand Bond beside him and shoot
/// him down; returns (the walker, the guard's index) on the tick he enters the fall.
fn shot_down(level: &str, enable: u32, ch: u8) -> (Walker, usize) {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, level, false).expect("load"));
    w.wait(120);
    let sim = w.scene.sim.as_mut().unwrap();
    let g = sim.drones.list.iter().position(|d| d.death_channel == ch).expect("the guard [D param 12]");
    sim.drones.channels.set(enable, 1, sim.drones.frame);
    w.wait(60);
    let sim = w.scene.sim.as_mut().unwrap();
    let at = sim.drones.list[g].pos;
    sim.bond.pos = [at[0] + 2.5, at[1], at[2]];
    w.wait(30);
    w.shoot(g, "the guard");
    for _ in 0..240 {
        let sim = w.scene.sim.as_ref().unwrap();
        if matches!(sim.drones.list[g].state, state::DEATH_ANIM | state::DEATH_BY_EXPLOSION) { return (w, g); }
        if sim.drones.list[g].state == state::DEAD { panic!("the fall was missed"); }
        w.tick(Default::default(), false);
    }
    panic!("the guard never fell");
}

#[test]
fn on_07000001_the_channel_waits_for_the_end_of_the_fall() {
    // #166 (5.42, 6.42, 14.44): enable ch 14, death ch 19, the hall AND's input [D].
    let (mut w, g) = shot_down("07000001.bin", 14, 19);
    let mut ticks = 0;
    loop {
        let sim = w.scene.sim.as_ref().unwrap();
        let dead = sim.drones.list[g].state == state::DEAD;
        let on = sim.drones.channels.on(19);
        assert_eq!(on, dead, "tick {ticks}: ch 19 lands with the fall's end, not before [M 0x1637f8 case 2]");
        if dead { break; }
        ticks += 1;
        assert!(ticks < 900, "the fall ends");
        w.tick(Default::default(), false);
    }
    assert!(ticks > 5, "the fall takes time ({ticks} ticks)");
    eprintln!("07000001 fall {ticks} ticks");
}

#[test]
fn on_07000002_the_timer_caps_the_wait_at_90_ticks() {
    // #449 (−46.56, 5.83, 7.45): enable ch 0x17, death ch 0x29 [D].
    let (mut w, g) = shot_down("07000002.bin", 0x17, 0x29);
    let mut ticks = 0;
    loop {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(0x29) { break; }
        ticks += 1;
        assert!(ticks <= 91, "timer C raises it by +90 [M 0x1637f8:55-61 + case 0xC]");
        w.tick(Default::default(), false);
    }
    assert!(ticks > 5, "not on the first update ({ticks} ticks)");
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("07000002 channel after {ticks} ticks, state {:#x}", sim.drones.list[g].state);
}

#[test]
fn the_bathhouse_scene_removes_a_guard_still_falling() {
    let (mut w, g) = shot_down("07000002.bin", 0x17, 0x29);
    w.tick(Default::default(), false);
    let sim = w.scene.sim.as_mut().unwrap();
    assert!(sim.drones.list[g].present, "still on screen");
    sim.drones.channels.set(0x35, 1, sim.drones.frame);
    w.tick(Default::default(), false);
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!sim.drones.list[g].present, "gone with ch 0x35 on [M 0x1637f8:76-90]");
    assert!(sim.drones.channels.on(0x29), "his channel lands with him [M :80-82]");
}

#[test]
fn another_guard_keeps_falling_through_ch_0x35() {
    // Sister: #443's death channel is 138, not one of the scene's three [M :78-80].
    let (mut w, g) = shot_down("07000002.bin", 21, 138);
    let sim = w.scene.sim.as_mut().unwrap();
    sim.drones.channels.set(0x35, 1, sim.drones.frame);
    w.tick(Default::default(), false);
    assert!(w.scene.sim.as_ref().unwrap().drones.list[g].present);
}
