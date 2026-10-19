//! mp-shot-gate: every machine enforces the shooter's fire rate and ammo (docs/systems/mp-shot-gate.md).
//!
//! A shot is a description in the input row (`Input::shot` + `shot_weapon` + `shot_aim`), so a
//! machine (a bug, a modified client) can send one EVERY tick — far past what the game allows.
//! The shared gate (`MpCombat::gate_tick` / `gate_allow`) applies a slot's shot only when the
//! game would let that slot fire: it holds that weapon ([M] `Player_WeaponFiring` 0x1a5460 fires
//! the held weapon, player data `+0x62`), has the rounds ([M] `Player_RoundToFire` 0x1a18e0's
//! check and spend of the `+0x91` byte), and the weapon's refire interval has run ([M] the
//! `BLData+0x92c` timer's fall and the fire block's reset to `max(weapon_data[w]+0x40, 1.0)`).
//! Two machines tick the same rows through `nf_view::net_tick`; every tick's hash must agree.
//! The Sentinel Unguided 0x2d: interval 30 frames, clip 4, `+0x91` = 1 [D the ACTION.ELF weapon
//! table, read through the same records the gate uses]. The Storm M32 0x12: interval 6, clip 32.
//! NF_ISO-gated (the arena loads from the disc).

use nf_game::mp::combat as cb;
use nf_game::mp::MpSettings;
use nf_net::NEUTRAL;

const N: usize = nf_net::MAX_AGENTS;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

fn settings() -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = 2;
    s.player_count = 2;
    s.bot_count = 0;
    s.total_players = 2;
    s.seed = 0x0bed_0bad;
    s.time_limit = 60 * 60;
    s.frag_limit = -1;
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

/// The gate's expected grant for a first-held weapon: the game's spawn-equip split —
/// `clip_size` into the clip, `clip_size` more carried (`mp_equip_player`'s `clip_size·2`
/// through `Inventory::equip_weapon`) [M nf-game::mp::equip].
fn seed_rounds(sim: &nf_view::scene::Sim, weapon: u8) -> i32 {
    let w = &sim.weapons;
    2 * w.clip_size(weapon as usize) as i32
}

#[test]
fn rocket_spam_fires_at_the_launcher_rate_and_stops_when_dry() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let s = settings();
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    let interval = u32::from_le_bytes(a.sim.as_ref().unwrap().weapons.records[0x2d][0x40..0x44].try_into().unwrap()) as i32;
    assert_eq!(interval, 30, "the Unguided Sentinel's refire interval [D +0x40]");
    let total_rounds = seed_rounds(a.sim.as_ref().unwrap(), 0x2d);
    assert_eq!(total_rounds, 8, "the Sentinel's clip 4 + carried 4 [D +0x92]");
    // A rocket every tick, fire held (the cheat the stress test exposed), aimed high so the
    // blasts never touch the shooter (no death → no respawn re-equip muddies the count).
    let spam = |t: u32| {
        let mut i = NEUTRAL;
        i.held = 0x2d;
        i.fire = true;
        i.shot = true;
        i.shot_weapon = 0x2d;
        i.shot_aim = cb::aim_from_dir([0.0, 0.95, 0.05]);
        let _ = t;
        i
    };
    let ticks = 700u32;
    let mut accepted_a: Vec<u32> = Vec::new();
    let mut accepted_b: Vec<u32> = Vec::new();
    for t in 0..ticks {
        let row: [nf_net::Input; N] = std::array::from_fn(|sl| if sl == 0 { spam(t) } else { NEUTRAL });
        let h0 = nf_view::net_tick(a.sim.as_mut().unwrap(), &row, 0);
        let h1 = nf_view::net_tick(b.sim.as_mut().unwrap(), &row, 1);
        assert_eq!(h0, h1, "tick {t}: the two machines agree");
        let fa = a.sim.as_ref().unwrap().mp_combat.fired.len();
        let fb = b.sim.as_ref().unwrap().mp_combat.fired.len();
        assert_eq!(fa, fb, "tick {t}: both machines accept alike");
        if fa > 0 { accepted_a.push(t); }
        if fb > 0 { accepted_b.push(t); }
    }
    assert_eq!(accepted_a.len(), total_rounds as usize,
        "the spam fires exactly the seeded rounds (accepted at {accepted_a:?})");
    assert_eq!(accepted_a.first().copied(), Some(0), "the first shot leaves immediately");
    // The rate: no two accepted shots closer than the record's interval.
    for w in accepted_a.windows(2) {
        assert!((w[1] - w[0]) as i32 >= interval, "shots {w:?} are at least {interval} apart");
    }
    // The dry tail: after the rounds ran out the same spam yields nothing for the rest.
    let last = *accepted_a.last().unwrap();
    assert!(ticks - last > 300, "the dry tail is long ({last} was the last)");
    // Sister: the shared clip really is spent — the Unguided 0x2d shares its base's (the guided
    // 0x2c's) clip slot (`+0x05` twin, the same `+0x90` ammo [M Player_RoundToFire 0x1a18e0]).
    let sa = a.sim.as_ref().unwrap();
    let g = &sa.mp_combat.gates[0];
    let cs = sa.weapons.base(0x2d as usize) as usize;
    assert_eq!(g.inv.clip.get(cs).copied(), Some(0), "the shared clip is empty");
    assert_eq!(g.inv.carried.get(sa.weapons.ammo_type(0x2d as usize)).copied(), Some(0), "the shared spare is empty");
}

#[test]
fn normal_rate_loses_no_shots() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let s = settings();
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    let interval = u32::from_le_bytes(a.sim.as_ref().unwrap().weapons.records[0x12][0x40..0x44].try_into().unwrap()) as i32;
    assert_eq!(interval, 6, "the Storm M32's refire interval [D +0x40]");
    let clip = a.sim.as_ref().unwrap().weapons.clip_size(0x12 as usize) as i32;
    // A normal player: one shot description every interval ticks, the launcher's real cadence —
    // the clip holds out (the sent count stays under it), so no reload muddies the run.
    let sent = clip.min(30) as u32;
    let mut accepted: Vec<u32> = Vec::new();
    for t in 0..(sent * interval as u32 + 60) {
        let mut i = NEUTRAL;
        if t % interval as u32 == 0 && (t / interval as u32) < sent {
            i.held = 0x12;
            i.fire = true;
            i.shot = true;
            i.shot_weapon = 0x12;
            i.shot_aim = cb::aim_from_dir([0.0, 0.9, 0.1]);
        } else {
            i.held = 0x12;
        }
        let row: [nf_net::Input; N] = std::array::from_fn(|sl| if sl == 0 { i } else { NEUTRAL });
        let h0 = nf_view::net_tick(a.sim.as_mut().unwrap(), &row, 0);
        let h1 = nf_view::net_tick(b.sim.as_mut().unwrap(), &row, 1);
        assert_eq!(h0, h1, "tick {t}: the two machines agree");
        let fa = a.sim.as_ref().unwrap().mp_combat.fired.len();
        assert_eq!(fa, b.sim.as_ref().unwrap().mp_combat.fired.len(), "tick {t}: both machines accept alike");
        if fa > 0 { accepted.push(t); }
    }
    assert_eq!(accepted.len(), sent as usize, "every shot the real cadence sends passes ({accepted:?})");
    for (k, &t) in accepted.iter().enumerate() {
        assert_eq!(t, k as u32 * interval as u32, "shot {k} leaves on its cadence tick");
    }
}

#[test]
fn shot_for_an_unheld_weapon_does_nothing() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let s = settings();
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    // The spawn weapon the gate was equipped with (the Normal set's row[0] → 6) [M mp_equip_player].
    let held = a.sim.as_ref().unwrap().mp_combat.gates[0].weapon;
    // Phase 1: the slot claims a shot for the rocket launcher while HOLDING the spawn weapon.
    let mut accepts = 0u32;
    for t in 0..40 {
        let mut i = NEUTRAL;
        i.held = held;
        i.shot = t == 10;
        i.shot_weapon = 0x2d;
        i.shot_aim = cb::aim_from_dir([0.0, 0.9, 0.1]);
        let row: [nf_net::Input; N] = std::array::from_fn(|sl| if sl == 0 { i } else { NEUTRAL });
        let h0 = nf_view::net_tick(a.sim.as_mut().unwrap(), &row, 0);
        let h1 = nf_view::net_tick(b.sim.as_mut().unwrap(), &row, 1);
        assert_eq!(h0, h1, "tick {t}: the two machines agree");
        accepts += a.sim.as_ref().unwrap().mp_combat.fired.len() as u32;
        assert_eq!(a.sim.as_ref().unwrap().mp_combat.fired.len(), b.sim.as_ref().unwrap().mp_combat.fired.len());
    }
    assert_eq!(accepts, 0, "a shot for a weapon the slot does not hold spawns nothing");
    // Sister: the same description with the slot actually holding the launcher goes through —
    // the drop was the holding check, not the record or the aim.
    let mut passed = 0u32;
    for t in 0..10 {
        let mut i = NEUTRAL;
        i.held = 0x2d;
        i.shot = t == 3;
        i.shot_weapon = 0x2d;
        i.shot_aim = cb::aim_from_dir([0.0, 0.9, 0.1]);
        let row: [nf_net::Input; N] = std::array::from_fn(|sl| if sl == 0 { i } else { NEUTRAL });
        let h0 = nf_view::net_tick(a.sim.as_mut().unwrap(), &row, 0);
        let h1 = nf_view::net_tick(b.sim.as_mut().unwrap(), &row, 1);
        assert_eq!(h0, h1, "tick {t}: the two machines agree");
        passed += a.sim.as_ref().unwrap().mp_combat.fired.len() as u32;
    }
    assert!(passed >= 1, "the held launcher's shot goes through");
}
