//! 07000006's guard-kill chain: AND #709 raises ch 0x2c once the death channels 0x30, 0x31 and
//! 0x1c are all on [D #709 out 0x2c ← {0x30, 0x31, 0x1c}]. Those are the oDrone death channels
//! (param 12) of #446/#731 (0x30), #448/#732 (0x31) and #719/#729 (0x1c) [D]. Ch 0x2c then feeds
//! OR #752 → ch 0xb4, the lock of the tunnel sound plates #710/#711/#720/#721, and is itself the
//! lock of the spotlight plates #661/#662/#663/#674 [D params 7]. Each spawner brings in one guard
//! of its pair, so one kill per group closes the AND. Opt-in: NF_ISO.

use nf_game::player::RamSave;

fn carry(on: &[u32]) -> RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 240), &ch)
}

/// Lands by the truck and walks Bond into TouchOnce #531 (out 0x12, the guardhouse-court step
/// plate [D]), the gate of spawners #730/#733/#734, which bring the three groups in [D gate 0x12].
fn load(iso: &str) -> nf_view::scene::Scene {
    let mut s = nf_view::load_with_carry_difficulty(iso, "07000006.bin", false, Some(&carry(&[0x1d, 0x45])), 1).expect("load");
    for _ in 0..30 { s.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let sim = s.sim.as_mut().unwrap();
    assert!(!sim.drones.channels.on(0x12), "the spawn gate is off on arrival");
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, [18.46, 40.0, -12.14], 0.0);
    for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.drones.channels.on(0x12), "#531 raises ch 0x12 under Bond: {:?}", sim.bond.pos);
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, [57.0, 40.0, 22.6], 0.0);
    for _ in 0..600 { sim.step_bond(Default::default(), false, false, false); }
    s
}

/// A frag at the drone's feet, the walk tests' kill shape [M Explode_Propagate 0x17c518].
fn frag(s: &mut nf_view::scene::Scene, i: usize) {
    let sim = s.sim.as_mut().unwrap();
    let pos = sim.drones.list[i].pos;
    sim.drones.grenades.push(nf_game::grenade::Grenade {
        pos: [pos[0], pos[1] + 0.5, pos[2]], velocity: [0.0, 0.0, 0.0], speed: 0.0,
        travelled: 0.0, range: 60.0, damage: 100.0, radius: 3.0, weapon_type: 0x34, mesh: 0x0200_0726, owner: usize::MAX, state: 0, bounces: 0, smoke_popped: false, resting: false,
        stuck: false, seek_target: None, phase: None, burn: 0, defused: false, loop_handle: 0, born: 0,
    });
    for _ in 0..240 {
        s.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if s.sim.as_ref().unwrap().drones.list[i].corpse { break; }
    }
    for _ in 0..30 { s.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
}

fn kill_group(s: &mut nf_view::scene::Scene, ch: u8) -> usize {
    let ids: Vec<usize> = s.sim.as_ref().unwrap().drones.list.iter().enumerate()
        .filter(|(_, d)| d.death_channel == ch && d.present && !d.corpse).map(|(i, _)| i).collect();
    for &i in &ids { frag(s, i); }
    ids.len()
}

fn census(s: &nf_view::scene::Scene) {
    for (i, d) in s.sim.as_ref().unwrap().drones.list.iter().enumerate().filter(|(_, d)| matches!(d.death_channel, 0x30 | 0x31 | 0x1c)) {
        eprintln!("drone {i}: ch {:#x} present {} corpse {} pos {:?}", d.death_channel, d.present, d.corpse, d.pos);
    }
}

#[test]
fn the_three_guard_groups_dead_raise_0x2c_and_0xb4() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let mut s = load(&iso);
    census(&s);
    for ch in [0x30u8, 0x31, 0x1c] {
        let n = kill_group(&mut s, ch);
        assert!(n > 0, "a present guard carries death channel {ch:#x}");
        assert!(s.sim.as_ref().unwrap().drones.channels.on(ch as u32), "the group's death channel {ch:#x} is on");
    }
    census(&s);
    let sim = s.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(0x2c), "AND #709 raises ch 0x2c");
    assert!(sim.drones.channels.on(0xb4), "OR #752 raises ch 0xb4");
}

/// Sister: two of the three groups dead leave the AND off.
#[test]
fn two_groups_dead_leave_0x2c_off() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let mut s = load(&iso);
    for ch in [0x30u8, 0x31] { assert!(kill_group(&mut s, ch) > 0); }
    let sim = s.sim.as_ref().unwrap();
    assert!(!sim.drones.channels.on(0x1c), "the third group lives");
    assert!(!sim.drones.channels.on(0x2c), "AND #709 waits on all three");
    assert!(!sim.drones.channels.on(0xb4), "OR #752 stays off without 0x2c or 0x39");
}

