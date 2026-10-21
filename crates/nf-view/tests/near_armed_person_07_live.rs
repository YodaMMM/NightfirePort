//! near-armed-person (833): CivilianDoorGuard 0x31's UPDATE turns hostile on
//! `NDrone2_NearArmedPerson(20.0)` [M 0x160648:39-57] — voice 0xD exempt, behaviour prop 0x3c,
//! the reaction-acked sight `+0x228 & 4`, `DroneWeap_OpponentHasWeapon` (the 20.0 is dead)
//! [M 0x1787c8:30-42, 0x17a4b8]. The port's stand-in read raw line of sight + armed.

#[path = "common/walker.rs"]
mod walker;

use nf_game::drone::state;
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

struct Guard { i: usize, p3c: bool, voice: u16 }

fn setup() -> (walker::Walker, Vec<Guard>, u8) {
    let mut w = walker::Walker::new(nf_view::load(&iso(), "07000007.bin", false).expect("load 07000007"));
    w.wait_enabled(6000, "level start");
    w.wait(120);
    let sim = w.scene.sim.as_ref().unwrap();
    let data = sim.data();
    let guards: Vec<Guard> = sim.drones.list.iter().enumerate()
        .filter(|(_, g)| g.present && g.enabled && g.state == state::CIVILIAN_DOOR_GUARD)
        .map(|(i, g)| Guard { i, p3c: g.prop(&data, 0x3c), voice: g.voice_type }).collect();
    let gun = (2u8..0x47).find(|&k| sim.drones.inventory.owned[k as usize]).unwrap_or(0);
    for g in &guards { eprintln!("door guard {} prop3c {} voice {:#x} pos {:?}", g.i, g.p3c, g.voice, sim.drones.list[g.i].pos); }
    eprintln!("carried gun {gun:#x}");
    (w, guards, gun)
}

/// Bond stands 4 units in front of door guard `gi` (along the guard's facing) for `ticks`, armed
/// or not. Returns per tick (state, los, seen).
fn face(w: &mut walker::Walker, gi: usize, armed: Option<u8>, ticks: u32) -> Vec<(u8, bool, bool)> {
    let mut out = vec![];
    for _ in 0..ticks {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            let g = &sim.drones.list[gi];
            let (p, yaw) = (g.pos, g.yaw);
            sim.bond.pos = [p[0] + yaw.sin() * 4.0, p[1], p[2] + yaw.cos() * 4.0];
            sim.bond.vel = [0.0; 3];
            sim.bond.yaw = yaw + std::f32::consts::PI;
            let k = armed.unwrap_or(0);
            sim.gun.weapon = k; sim.gun.switch_request = k;
        }
        w.tick(Input::default(), false);
        let g = &w.scene.sim.as_ref().unwrap().drones.list[gi];
        out.push((g.state, g.los, g.seen));
    }
    out
}

/// Armed Bond in front of each 07 door guard: the guard turns hostile (leaves 0x31) only on a
/// tick where the reaction-acked sight `+0x228 & 4` is set [M 0x1787c8:37 via 0x160648:39] —
/// never on raw line of sight alone (the stand-in's trigger, which fired the first visible
/// tick, before `DroneFunc_ReactionTime` acked the sighting). Sister: unarmed, the same guard
/// stays 0x31 for the whole window (`DroneWeap_OpponentHasWeapon` [M 0x17a4b8]).
#[test]
fn door_guards_react_to_an_armed_bond_only_through_the_acked_sight() {
    let (_, guards, gun) = setup();
    assert!(gun != 0, "Bond carries a gun on 07 [D level start inventory]");
    assert!(!guards.is_empty(), "07's door guards reach CivilianDoorGuard 0x31");
    let mut hostile = 0;
    for g in &guards {
        let (mut w, _, _) = setup();
        let ticks = face(&mut w, g.i, Some(gun), 150);
        if let Some(k) = ticks.iter().position(|t| t.0 != state::CIVILIAN_DOOR_GUARD) {
            hostile += 1;
            let (_, los, seen) = ticks[k];
            assert!(seen, "guard {} left 0x31 on tick {k} without the acked sight (los {los}) — the stand-in's raw-sight trigger", g.i);
            let first_los = ticks.iter().position(|t| t.1);
            eprintln!("guard {}: first los t{first_los:?}, hostile t{k}", g.i);
        }
        let (mut w, _, _) = setup();
        let un = face(&mut w, g.i, None, 150);
        assert!(un.iter().all(|t| t.0 == state::CIVILIAN_DOOR_GUARD), "sister: unarmed Bond leaves guard {} in 0x31", g.i);
    }
    assert!(hostile > 0, "an armed Bond 4 units in front turns at least one door guard hostile [M 0x160648:39-57]");
}

