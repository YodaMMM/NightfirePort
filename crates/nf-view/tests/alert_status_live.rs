//! alert-status-vs-alertness (916): `Drone_AlertStatusSet` 0x13a8d8 writes only the status byte
//! +0x51C (+ prev +0x51D, stamp +0x520) [M 0x13a8d8:9-17] — never the alertness +0x50C that
//! scales vision (`visibility()`). The Taser state 0x162b98 calls `AlertStatusSet(2)` and writes
//! no +0x50C [M case 1]; Combat 0x1731b8 writes `+0x50C = 1.0` [M :18].

use nf_game::drone::state;
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn a_status_set_leaves_alertness_alone_and_a_real_write_lands() {
    let mut scene = nf_view::load(&iso(), "07000005.bin", false).expect("load 07000005");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..300u32 { sim.step_bond(Input::default(), false, false, false); }
    let g = sim.drones.list.iter().position(|d| d.present && d.enabled && d.team == 1 && !d.corpse
        && matches!(d.state, state::IDLE | state::PATROL) && d.alert < 0.6)
        .expect("a calm guard on 07000005");
    let a0 = sim.drones.list[g].alert;
    sim.drones.list[g].set_state(state::TASER);
    sim.step_bond(Input::default(), false, false, false);
    let d = &sim.drones.list[g];
    assert_eq!(d.state, state::TASER, "the guard entered Taser");
    assert_eq!(d.alert_status, 2, "Taser's AlertStatusSet(2) sets the status byte [M 0x162b98:42]");
    assert!((d.alert - a0).abs() < 0.01, "the status set leaves alertness alone ({a0} → {}) [M 0x13a8d8]", d.alert);
    // Sister: Combat's real +0x50C write lands.
    let g2 = sim.drones.list.iter().position(|d| d.present && d.enabled && d.team == 1 && !d.corpse
        && matches!(d.state, state::IDLE | state::PATROL) && d.alert < 0.6).expect("another calm guard");
    sim.drones.list[g2].set_state(state::COMBAT);
    sim.step_bond(Input::default(), false, false, false);
    assert!(sim.drones.list[g2].alert >= 0.99, "Combat writes +0x50C = 1.0 [M 0x1731b8:18] (got {})", sim.drones.list[g2].alert);
}
