//! attack-reentry-choice (913): `NDrone2_DSTATE_Attack` 0x164210 case 1 calls
//! `DroneFunc_FirstAttack` 0x148010, which returns 0 at once once `+0x4F8 & 0x2000000` is set
//! [M 0x148010:17-19]; Attack then takes its own fallback [M 0x164210:24-90] — prop 0x18 in
//! range → AimStand 0x67 (resume 0x68), else Combat 0x58, snipers SniperAim 0x2a — never the
//! first sight's combat-move roll (props 0x1B/0x19 → `ChooseCombatMove`).

use nf_game::drone::state;
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn attack_reentries_take_attacks_own_fallback() {
    let mut scene = nf_view::load(&iso(), "07000011.bin", false).expect("load 07000011");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..600u32 { sim.step_bond(Input::default(), false, false, false); }
    let data_props = |sim: &nf_view::scene::Sim, i: usize, p: usize| { let d = sim.data(); sim.drones.list[i].prop(&d, p) };
    // An enabled, armed guard whose props allow the first-sight combat-move roll (0x19 or 0x1B).
    let cands: Vec<usize> = (0..sim.drones.list.len()).filter(|&i| {
        let d = &sim.drones.list[i];
        d.present && d.enabled && d.team == 1 && !d.corpse && (d.weapon as u8) > 1
    }).collect();
    let g = cands.iter().copied().find(|&i| data_props(sim, i, 0x19) || data_props(sim, i, 0x1b))
        .expect("an enabled armed guard with prop 0x19/0x1B on 07000011");
    let p18 = data_props(sim, g, 0x18);
    // First sight on the real path: Bond 5 units in front.
    let mut entered = false;
    for _ in 0..600u32 {
        let (p, yaw) = (sim.drones.list[g].pos, sim.drones.list[g].yaw);
        sim.bond.pos = [p[0] + yaw.sin() * 5.0, p[1], p[2] + yaw.cos() * 5.0];
        sim.bond.vel = [0.0; 3];
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.list[g].first_attack_done { entered = true; break; }
    }
    assert!(entered, "sister: the guard's first Attack ran FirstAttack");
    // 40 forced re-entries: each must land in Attack's fallback.
    let mut seen = std::collections::BTreeMap::new();
    for _ in 0..40u32 {
        sim.drones.list[g].set_state(state::ATTACK);
        sim.step_bond(Input::default(), false, false, false);
        let st = sim.drones.list[g].state;
        *seen.entry(st).or_insert(0) += 1;
        for _ in 0..10u32 { sim.step_bond(Input::default(), false, false, false); }
    }
    eprintln!("guard {g} prop18 {p18} re-entry states {seen:x?}");
    // This guard has no prop 0x18 and is no sniper: the fallback is Combat 0x58 every time
    // [M 0x164210:69-88]. Fail-first: the old arm re-ran the first-sight roll — 11 of 40 re-entries
    // landed in AimStandFire 0x68 (`ChooseCombatMove`) on master 0d52227f.
    assert!(!p18 && !matches!(sim.drones.list[g].mode, 2 | 0x19), "the guard is the no-0x18 non-sniper case");
    assert_eq!(seen.keys().copied().collect::<Vec<u8>>(), vec![state::COMBAT],
        "every re-entry takes Attack's fallback — Combat for this guard [M 0x148010:17-19 + 0x164210:69-88]: {seen:x?}");
}
