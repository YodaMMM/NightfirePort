//! first-sight-flag (834): `DroneFunc_FirstAttack` 0x148010 runs once per drone — `+0x4F8 &
//! 0x2000000` set returns 0 at once [M :17-19] — so its `AttackTalk` [M :94] plays on the FIRST
//! Attack entry only; `DSTATE_Attack` 0x164210 itself never talks. The own-sight bit `+0x4F8 &
//! 0x8000000` (the port's `own_sight`) routes that first entry through `FirstSightState` 0x147c48
//! (`SeenPlayerTalk` first [M :106]). The port's old `attack_talked` stand-in let every re-entry
//! bark `AttackTalk` again.

use nf_game::drone::state;
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// 07000011: voice-type-1 guards always take the AttackTalk line (odds 0 [M 0x13eac0]). The
/// first Attack entry may talk; a forced re-entry after the line finishes queues nothing.
/// Sister: the first entry did go through FirstAttack (`first_attack_done`).
#[test]
fn attack_reentry_does_not_bark_again() {
    let mut scene = nf_view::load(&iso(), "07000011.bin", false).expect("load 07000011");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..600u32 { sim.step_bond(Input::default(), false, false, false); }
    let g = sim.drones.list.iter().position(|d| d.present && d.enabled && d.team == 1 && d.voice_type == 1 && !d.corpse)
        .expect("an enabled voice-type-1 guard on 07000011");
    // Bond 5 units in front of him.
    let mut entered = false;
    for _ in 0..600u32 {
        let (p, yaw) = (sim.drones.list[g].pos, sim.drones.list[g].yaw);
        sim.bond.pos = [p[0] + yaw.sin() * 5.0, p[1], p[2] + yaw.cos() * 5.0];
        sim.bond.vel = [0.0; 3];
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.list[g].first_attack_done { entered = true; break; }
    }
    assert!(entered, "the guard reached Attack's FirstAttack");
    // Let any line finish.
    for _ in 0..600u32 {
        sim.step_bond(Input::default(), false, false, false);
        let d = &sim.drones.list[g];
        if d.voice.is_none() || d.voice_finished { break; }
    }
    let before = sim.drones.list[g].voice;
    sim.drones.list[g].voice = None;
    sim.drones.list[g].voice_finished = true;
    let talks0 = sim.drones.list[g].attack_entry_talks;
    sim.drones.list[g].set_state(state::ATTACK);
    let mut trace = vec![];
    for _ in 0..3u32 {
        sim.step_bond(Input::default(), false, false, false);
        let d = &sim.drones.list[g];
        trace.push((d.state, d.voice));
    }
    eprintln!("voice before {before:x?}; per tick (state, voice) {trace:x?}");
    // The follow-on states (AimStandFire 0x166328, CombatNoMove 0x1643e8 …) carry their own
    // AttackTalk calls in the game; only Attack's entry site is pinned here.
    assert_eq!(sim.drones.list[g].attack_entry_talks, talks0,
        "an Attack re-entry makes no AttackTalk call [M 0x148010:17-19 — FirstAttack returns at once]");
}
