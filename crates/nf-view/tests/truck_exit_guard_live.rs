//! level-05-truck-exit: the courtyard guard does NOT block the truck exit on the real route, per difficulty.
//!
//! The question (task 07000005): the exit by truck waits while a guard aware of Bond is within 30 u
//! (ch 0x98, the EndLevel event 0x11's test channel [D]). Walk the real route to the truck with real
//! input on each difficulty and check whether the courtyard guard notices Bond there.
//!
//! The mechanism, all evidenced: the EndLevel script 0x06000A23's event 0x11 [0x98, 0x98, 0x1D, 1]
//! writes ch 0x1D (the LOAD_LEVEL gate) only while ch 0x98 is clear [M Script_EventHandler 0x1d94c0
//! case 0x11]; ch 0x98 is `Drone_InitComms` 0x139520's chatter gate, written EVERY frame: 1 while any
//! living ENABLED drone at alert ≥ 1.0 stands within 30 u of its opponent, not team 2 [M 0x139520
//! :173-179, :198-201]. The game's own ride-time snapshot agrees the post guards never chatter there
//! [L ee_snapshot.bin: the two post guards at (32.4/27.6, 37.0, ~49.8) sit with flags bit 0x100 CLEAR
//! (disabled, the WaitSwitch pair) and alert 0.000, ch 0x98 = 0, ch 0x97 = 1 (the chatter latched
//! once earlier in that run), switch_channels = {0x45, 0x73, 0x97}; the four enabled gate guards at
//! (61-64, 93-104)/(150, 17, 101) hold Bond (the cold start) as their opponent at 42-47 u — past the
//! 30 u test].
//!
//! This walk (the walkreal route, real input only: cold start → the gatehouse porch → the hand-off
//! cut-scene → the crouched ride → the park) pins, on EVERY difficulty (1..=4, the front-end's
//! launch path `load_with_carry_difficulty`): the guard chain completes (ch 0x22 mid-ride, the post
//! touch ch 0x72, the park ch 0x70, mux ch 0x65), NO aware guard ever raises ch 0x98, and the level
//! exits into 07000006 — the guard does not block the exit on any difficulty, matching the game.
//!
//! Run with: `NF_ISO=… cargo test -p nf-view --test truck_exit_guard_live -- --nocapture --test-threads=1`

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The real-input walk per difficulty. Returns the exit tick and the park-moment guard lines.
fn walk_route(difficulty: u32) -> (u32, Vec<String>) {
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso(), "07000005.bin", false, None, difficulty)
        .expect("load 07000005"));
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - 104.95).abs() < 2.0 && (p[2] - 108.83).abs() < 2.0, "cold start #26 [D], got {p:?}");
        assert_eq!(sim.drones.difficulty, difficulty, "the front-end's difficulty rides the load");
    }
    w.wait(30);
    w.walk_nav_to([98.6, 26.0, 106.5], "to the gatehouse", 1.6);
    let mut prev_dis = w.scene.sim.as_ref().unwrap().disabled;
    let mut crouch_armed = false;
    let mut input = Input::default();
    let mut exit_tick = 0u32;
    let mut ch98_ticks = 0u32;
    let mut ch97_latched = false;
    let mut saw_porch = false;
    let mut saw_ch72 = false;
    let mut guard_lines = Vec::new();
    for t in 0..12000u32 {
        let dis = w.scene.sim.as_ref().unwrap().disabled;
        if !dis && prev_dis && w.scene.sim.as_ref().unwrap().bond.pos[0] < 95.0 { crouch_armed = true; }
        prev_dis = dis;
        input.crouch = if crouch_armed && !w.scene.sim.as_ref().unwrap().bond.crouched { crouch_armed = false; true } else { false };
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(0x5c) { saw_porch = true; }
        if sim.drones.channels.on(0x72) { saw_ch72 = true; }
        if sim.drones.channels.v[0x98] != 0 { ch98_ticks += 1; }
        if sim.drones.channels.on(0x97) { ch97_latched = true; }
        // The park moment: ch 0x70 on and Bond on the bed — log the guards once, near the park.
        if sim.drones.channels.on(0x70) && guard_lines.is_empty() {
            for (i, dr) in sim.drones.list.iter().enumerate() {
                if dr.corpse || dr.health <= 0.0 { continue; }
                let dx = dr.pos[0] - sim.bond.pos[0];
                let dz = dr.pos[2] - sim.bond.pos[2];
                let d = (dx * dx + dz * dz).sqrt();
                if d > 35.0 { continue; }
                guard_lines.push(format!(
                    "  d{difficulty} park guard[{i}] skin {:08x} state {:02x} team {} en {} alert {:.3} floor {:.3} dist_to_bond {d:.1} pos {:?}",
                    dr.skin, dr.state, dr.team, dr.enabled as u8, dr.alert, dr.alert_floor, dr.pos));
            }
        }
        if sim.drones.level_exit.is_some() { exit_tick = w.total_ticks; break; }
    }
    for l in &guard_lines { eprintln!("{l}"); }
    eprintln!("[d{difficulty}] exit tick {exit_tick}, ch98-on ticks {ch98_ticks}, ch97 latched {ch97_latched}, porch {saw_porch}, ch72 {saw_ch72}");
    // The route completed through the real chain on this difficulty.
    assert!(saw_porch, "d{difficulty}: the ch-0x5C porch plate fired (the hand-off ran)");
    assert!(saw_ch72, "d{difficulty}: the gate guard reached his post and touched #318 (ch 0x72) [D placement]");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x70), "d{difficulty}: the ride parked on the ch-0x70 plate");
    // THE FINDING: no aware guard ever raised the chatter gate on the real route — the exit is not blocked.
    assert_eq!(ch98_ticks, 0, "d{difficulty}: ch 0x98 (an aware guard within 30 u) never set on the real route");
    assert!(!ch97_latched, "d{difficulty}: the chatter latch 0x97 never fired on the real route");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.level_exit, Some(0x0700_0006),
        "d{difficulty}: the truck chain ends the level into 07000006 — the courtyard guard does not block the exit");
    (exit_tick, guard_lines)
}

/// The task's walk, every difficulty. Green = the exit works everywhere; the post guards sit at
/// their floor alert (they never see Bond on the bed), exactly as the game's ride-time snapshot
/// shows them (disabled, alert 0.000, ch 0x98 = 0 [L ee_snapshot.bin]).
#[test]
fn the_truck_exit_works_on_every_difficulty_the_courtyard_guard_never_blocks_it() {
    let mut all_lines = Vec::new();
    for d in [1u32, 2, 3, 4] {
        let (_tick, lines) = walk_route(d);
        all_lines.extend(lines);
    }
    // The post guards the walk logged at the park: every alive guard still standing near the park
    // is at (or under) his floor alert — nobody is aware; the 30 u test has no aware input.
    assert!(!all_lines.is_empty(), "the park log should show the guards near the truck");
    for l in &all_lines {
        // Parse the alert out of the log line (kept text so the failure reads like the report).
        let alert = l.split(" alert ").nth(1).and_then(|s| s.split(' ').next())
            .and_then(|s| s.parse::<f32>().ok()).expect("alert in the guard line");
        let floor = l.split(" floor ").nth(1).and_then(|s| s.split(' ').next())
            .and_then(|s| s.parse::<f32>().ok()).expect("floor in the guard line");
        assert!(alert < 1.0 && alert <= floor + 1e-3,
            "an aware guard stood near the park (chatter would block the exit): {l}");
    }
}
