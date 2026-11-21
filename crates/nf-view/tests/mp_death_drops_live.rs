//! mp-drops: a human who dies in a multiplayer match drops the gun in hand
//! (`Player_CheckForDeath` 0x190810:42-56 [M]): a kind-0 pickup of the held record's base id with
//! the base slot's clip, created at the view model's matrix and falling to the floor, lasting
//! `FRAME_RATE_INT · 30` ticks (`Pickup_CreateSimple` 0x18f348's MP lifetime) and spinning
//! (`Pickup_Create` 0x18dda0's MP `+0x38`). Pinned on the real arena 07000023: the offline local
//! match (the tick driver `Sim::step_bond`) and the online lockstep (two machines through
//! `nf_view::net_tick`, the hashes agree and both hold the same pickup). Also pinned online: an
//! arena pickup's respawn wait is `FRAME_RATE_INT · respawn · 10 + 1` ticks (`Pickup_Update`
//! 0x18e148:183-185 restores once `elapsed > limit`), the count the offline path already used.
//! NF_ISO-gated.

use nf_game::mp::MpSettings;
use nf_net::NEUTRAL;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn step(scene: &mut nf_view::scene::Scene, n: u32) {
    for _ in 0..n {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    }
}

fn local_match() -> nf_view::scene::Scene {
    let iso = iso().expect("NF_ISO");
    let ms = MpSettings::deathmatch(0x0700_0023);
    let mut scene = nf_view::load_mp_carry(&iso, "07000023.bin", false, None, ms).expect("the arena loads");
    // Past the spawn draw: the spawn weapon is in hand.
    step(&mut scene, 120);
    scene
}

/// Offline: the local player's death drops the held gun with its clip; it falls, spins, lasts
/// 30 s, and the player takes it after the respawn.
#[test]
fn a_dead_player_drops_the_held_gun_in_a_local_match() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = local_match();
    let (held, base, clip, before) = {
        let sim = scene.sim.as_ref().unwrap();
        let held = sim.gun.weapon;
        let base = sim.weapons.base(held as usize);
        (held, base, sim.drones.inventory.clip[base as usize], sim.drones.pickups.len())
    };
    eprintln!("held {held:#x} base {base:#x} clip {clip}");
    assert!(held != 0x47 && clip > 0, "the spawn weapon is drawn with rounds in the clip");
    scene.sim.as_mut().unwrap().drones.health.health = 0.0;
    step(&mut scene, 1);
    let k = {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.death.active, "the death started");
        assert_eq!(sim.drones.pickups.len(), before + 1, "the death adds one pickup [M 0x190810:46-50]");
        let p = &sim.drones.pickups[before];
        assert_eq!((p.kind, p.weapon, p.rounds), (nf_game::pickup::kind::WEAPON, base, clip), "the held base with its clip");
        assert_eq!(p.lifetime, 1800, "FRAME_RATE_INT · 30 [M 0x18f440-0x18f47c]");
        assert!(p.spin && p.state == 0 && p.respawn == 0, "spinning, falling, deleted when taken");
        let eye = sim.bond.head();
        let at = [p.m[3][0], p.m[3][1], p.m[3][2]];
        let d = ((at[0] - eye[0]).powi(2) + (at[1] - eye[1]).powi(2) + (at[2] - eye[2]).powi(2)).sqrt();
        assert!(d < 1.5, "created at the view model, near the eye ({d:.2})");
        before
    };
    // It lands and turns: the spin rotates the matrix rows every tick.
    let r0 = scene.sim.as_ref().unwrap().drones.pickups[k].m[0];
    step(&mut scene, 120);
    {
        let sim = scene.sim.as_ref().unwrap();
        let p = &sim.drones.pickups[k];
        assert_eq!(p.state, 1, "landed");
        assert!(!p.gone);
        assert!((p.m[0][0] - r0[0]).abs() + (p.m[0][2] - r0[2]).abs() > 1e-3, "spun");
        assert_eq!(p.lifetime, 1800 - 120, "the lifetime runs once per tick");
    }
    // After the 5 s respawn the player walks back onto it and takes it.
    step(&mut scene, 200);
    let at = {
        let sim = scene.sim.as_ref().unwrap();
        assert!(!sim.drones.death.active, "respawned");
        let p = &sim.drones.pickups[k];
        [p.m[3][0], p.m[3][1], p.m[3][2]]
    };
    let taken_before = scene.sim.as_ref().unwrap().drones.taken.len();
    {
        let sim = scene.sim.as_mut().unwrap();
        // Spend some rounds so the ammo take is not refused at the cap.
        let t = sim.weapons.ammo_type(base as usize);
        sim.drones.inventory.carried[t] = 0;
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, at, 0.0);
    }
    step(&mut scene, 2);
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.pickups[k].gone, "taken and deleted (respawn 0)");
    assert!(sim.drones.taken.len() > taken_before, "the take landed in the inventory");
}

/// Offline sister: the drop expires after exactly 1800 ticks when nobody takes it.
#[test]
fn an_untaken_drop_is_gone_after_30_seconds() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = local_match();
    let before = scene.sim.as_ref().unwrap().drones.pickups.len();
    scene.sim.as_mut().unwrap().drones.health.health = 0.0;
    step(&mut scene, 1);
    assert_eq!(scene.sim.as_ref().unwrap().drones.pickups.len(), before + 1);
    // Keep the respawned player away from it.
    let mut life = 0u32;
    for t in 0..2000u32 {
        if t == 400 {
            let sim = scene.sim.as_mut().unwrap();
            let p = sim.drones.pickups[before].m[3];
            sim.bond = nf_game::player::Bond::stand_at(&sim.world, [p[0], p[1], p[2] + 40.0], 0.0);
        }
        step(&mut scene, 1);
        life += 1;
        if scene.sim.as_ref().unwrap().drones.pickups[before].gone { break; }
    }
    assert_eq!(life, 1800, "deleted on the tick its +0x2E reaches 0 [M 0x18e148:258-262]");
}

fn settings(humans: u32) -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = humans;
    s.player_count = humans as u16;
    s.total_players = humans;
    s.seed = 0x0bed_0bad;
    s.frag_limit = 120;
    s.time_limit = 10 * 60;
    s
}

fn machine(path: &str, s: &MpSettings, slot: usize) -> nf_view::scene::Scene {
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(path, &arena, false, None, s.clone()).expect("arena loads");
    let sim = scene.sim.as_mut().expect("sim");
    sim.net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b ^ (slot as u32).wrapping_mul(0x9e37_79b9)));
    sim.swap_local_slot(slot);
    scene
}

fn tick2(a: &mut nf_view::scene::Scene, b: &mut nf_view::scene::Scene, t: u32) {
    // Both humans hold the spawn weapon 6 (the wire's held byte the shot gate follows).
    let mut i = NEUTRAL;
    i.held = 6;
    let row: [nf_net::Input; nf_net::MAX_AGENTS] = [i; nf_net::MAX_AGENTS];
    let ha = nf_view::net_tick(a.sim.as_mut().unwrap(), &row, 0);
    let hb = nf_view::net_tick(b.sim.as_mut().unwrap(), &row, 1);
    assert_eq!(ha, hb, "tick {t}: the machines agree");
}

/// Online: slot 1 dies; both machines create the same drop on the same tick, from the slot's
/// shared gate (held weapon + clip), and stay in lockstep while it falls and spins.
#[test]
fn an_online_death_drop_is_the_same_on_every_machine() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let s = settings(2);
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    for t in 0..120 { tick2(&mut a, &mut b, t); }
    let before = a.sim.as_ref().unwrap().drones.pickups.len();
    let (held, clip) = {
        let sim = a.sim.as_ref().unwrap();
        let g = &sim.mp_combat.gates[1];
        (g.weapon, g.inv.clip[sim.weapons.base(g.weapon as usize) as usize])
    };
    eprintln!("slot 1 holds {held:#x} with {clip} in the clip");
    assert!(clip > 0);
    // The kill: what every kill site writes (the slot's health out, the dead counter's first tick).
    for sc in [&mut a, &mut b] {
        let sim = sc.sim.as_mut().unwrap();
        sim.mp_hp[1] = 0.0;
        sim.mp_dead_ticks[1] = 1;
    }
    tick2(&mut a, &mut b, 120);
    let pa = a.sim.as_ref().unwrap().drones.pickups.get(before).cloned().expect("machine a holds the drop");
    let pb = b.sim.as_ref().unwrap().drones.pickups.get(before).cloned().expect("machine b holds the drop");
    let base = a.sim.as_ref().unwrap().weapons.base(held as usize);
    assert_eq!((pa.kind, pa.weapon, pa.rounds), (nf_game::pickup::kind::WEAPON, base, clip));
    assert_eq!(pa.m, pb.m, "the same matrix on both machines");
    assert!(pa.spin);
    for t in 121..300 { tick2(&mut a, &mut b, t); }
    let (pa, pb) = (&a.sim.as_ref().unwrap().drones.pickups[before], &b.sim.as_ref().unwrap().drones.pickups[before]);
    assert_eq!((pa.state, pa.lifetime, pa.m), (pb.state, pb.lifetime, pb.m));
    assert_eq!(pa.state, 1, "landed");
}

/// Online: a taken arena pickup with respawn r waits `60·r·10 + 1` ticks — the restore lands on
/// the tick after `elapsed == limit` [M 0x18e148:183-185 `<= limit → return`; the take stamp
/// 0x18e650:345-347]. The online arm used to set `60·r·10`, one tick early.
#[test]
fn an_online_take_waits_the_full_respawn() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let s = settings(2);
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    for t in 0..60 { tick2(&mut a, &mut b, t); }
    let (k, at, r) = {
        let sim = a.sim.as_ref().unwrap();
        let (k, p) = sim.drones.pickups.iter().enumerate()
            .find(|(_, p)| !p.gone && p.state == 1 && p.respawn != 0 && p.respawn != 0xFFFF && p.kind == nf_game::pickup::kind::WEAPON)
            .expect("a respawning arena weapon");
        (k, [p.m[3][0], p.m[3][1], p.m[3][2]], p.respawn)
    };
    // Slot 0 stands on it on both machines (its own body on a, the replica on b).
    {
        let sim = a.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, at, 0.0);
    }
    {
        let sim = b.sim.as_mut().unwrap();
        let nb = nf_game::player::Bond::stand_at(&sim.world, at, 0.0);
        sim.mp_bonds[0] = Some(nb);
    }
    let mut taken = None;
    for t in 60..80 {
        tick2(&mut a, &mut b, t);
        if a.sim.as_ref().unwrap().drones.pickups[k].state == 2 { taken = Some(t); break; }
    }
    let t0 = taken.expect("slot 0 takes it");
    // Move slot 0 away so the restored pickup is not retaken.
    {
        let sim = a.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [at[0], at[1], at[2] + 40.0], 0.0);
    }
    {
        let sim = b.sim.as_mut().unwrap();
        let nb = nf_game::player::Bond::stand_at(&sim.world, [at[0], at[1], at[2] + 40.0], 0.0);
        sim.mp_bonds[0] = Some(nb);
    }
    let limit = 60 * r as u32 * 10;
    let mut t = t0 + 1;
    loop {
        tick2(&mut a, &mut b, t);
        if a.sim.as_ref().unwrap().drones.pickups[k].state != 2 { break; }
        t += 1;
        assert!(t < t0 + limit + 10, "never restored");
    }
    assert_eq!(t - t0, limit + 1, "restored once elapsed > {limit}");
}

/// mp-drops: the match take runs no `Upgrade_Weapon` [M 0x18e650:141-160]: slot 0 taking an
/// arena P99 (weapon 6) pickup online lands on record 6 in the shared gate, never on the
/// single-player handgun table's record 2 (the P2K).
#[test]
fn an_online_p99_take_stays_record_6() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let s = settings(2);
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    for t in 0..60 { tick2(&mut a, &mut b, t); }
    let found = {
        let sim = a.sim.as_ref().unwrap();
        sim.drones.pickups.iter().enumerate()
            .find(|(_, p)| !p.gone && p.state == 1 && p.weapon == 6 && p.kind == nf_game::pickup::kind::WEAPON)
            .map(|(k, p)| (k, [p.m[3][0], p.m[3][1], p.m[3][2]]))
    };
    let (k, at) = found.expect("the arena places a weapon-6 pickup (weapon set row 0, kind 7)");
    assert!(!a.sim.as_ref().unwrap().mp_combat.gates[0].inv.owned[2], "no P2K before the take");
    {
        let sim = a.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, at, 0.0);
    }
    {
        let sim = b.sim.as_mut().unwrap();
        sim.mp_bonds[0] = Some(nf_game::player::Bond::stand_at(&sim.world, at, 0.0));
    }
    let mut taken = false;
    for t in 60..80 {
        tick2(&mut a, &mut b, t);
        let p = &a.sim.as_ref().unwrap().drones.pickups[k];
        if p.state == 2 || p.gone { taken = true; break; }
    }
    assert!(taken, "slot 0 takes it");
    for sc in [&a, &b] {
        let g = &sc.sim.as_ref().unwrap().mp_combat.gates[0];
        assert!(!g.inv.owned[2], "the match take never grants the P2K record");
        assert!(g.inv.owned[6], "record 6 is the one owned");
    }
}

/// mp-drops: on a finished game the default kit arm's `EquipWeapon(10, 999)` lands on record 0xC
/// (`Player_EquipWeapon` → `Upgrade_Weapon` case 10 with `GameState[0x52]` set [M 0x1a8a58:24-30,
/// 0x1a8f30]); an unfinished game keeps record 10.
#[test]
fn a_finished_game_kit_holds_record_0xc() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let sc = nf_view::load_front_end(&path, "07000005.bin", false, None, false, 1, 0).expect("loads");
    let sim = sc.sim.as_ref().unwrap();
    let wd = sim.weapons.clone();
    let kit = |complete: bool| {
        let mut inv = nf_game::pickup::Inventory::empty(sim.drones.inventory.clip.len(), sim.drones.inventory.carried.len());
        let mut health = nf_game::bullet::PlayerHealth::default();
        let mut ch = nf_game::triggers::Channels::default();
        // 0x0700004b: an id outside the switch's jump table, the default arm [M 001a8268].
        nf_game::player::init_level_loadout_ex(0x0700_004b, None, &mut health, &mut inv, &mut ch, &wd, |_| true, complete);
        inv
    };
    let plain = kit(false);
    let done = kit(true);
    eprintln!("bases: 10 → {:#x}, 0xC → {:#x}", wd.base(10), wd.base(0xC));
    assert!(plain.owned[wd.base(10) as usize] && !plain.owned[wd.base(0xC) as usize] || wd.base(10) == wd.base(0xC), "unfinished: record 10");
    assert!(done.owned[wd.base(0xC) as usize], "finished: record 0xC is owned");
    if wd.base(10) != wd.base(0xC) {
        assert!(!done.owned[wd.base(10) as usize], "finished: record 10 is not");
    }
}
