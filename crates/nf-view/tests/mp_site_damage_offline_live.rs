//! mp-site-damage-offline: the Demolition/Protection site takes Bond's LOCAL bullets and
//! explosives offline (GAPS G-053 item 1). The game's two paths: the frame's bullet contacts
//! sum into ONE `SP_GetHitDamage` HP store [M `MP_DemolitionProtectionUpdate` 0x1870a0:81-91,
//! the mask-1 category filter 0x1ece30:24-31], and each blast applies `MP_ApplyDamage` directly
//! [M `Explode_Propagate` 0x17c518:187 case 0x35, `MP_ApplyDamage` 0x18b2c8]. The port's
//! stand-in is the site sphere (`MpMatch::site_sphere`, the live load's model bounds) fed by the
//! local pipeline (`drone.rs`'s bullet pass + `spawn_blast`), drained by the Sim. NF_ISO-gated;
//! arena 07000023 (the census's every-object arena). Sisters: a missed shot leaves the damage
//! at 0; online the shared combat still owns the site — one shot, exactly one round's damage
//! (the local visual bullets never feed it).

use nf_game::mp::{scenario, MpSettings};
use nf_view::mpnet::to_player_input;
use nf_view::scene::{Scene, Sim};

const ARENA: u32 = 0x0700_0023;
const N: usize = nf_net::MAX_AGENTS;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

fn settings(sc: u32, teams: [u32; 2]) -> MpSettings {
    let mut s = MpSettings::deathmatch(ARENA);
    s.scenario = sc;
    s.frag_limit = 3;
    s.time_limit = -1;
    s.human_count = 2;
    s.player_count = 2;
    s.total_players = 2;
    s.seed = 0x5ce7_a210;
    s.players[0].team = teams[0];
    s.players[1].team = teams[1];
    s.players[0].codename = "Alpha".into();
    s.players[1].codename = "Bravo".into();
    s
}

/// An offline `--mp` load (no session): the local pipeline only.
fn offline_load(sc: u32, teams: [u32; 2]) -> Option<Scene> {
    let path = iso()?;
    Some(nf_view::load_mp_carry(&path, &format!("{ARENA:08x}.bin"), false, None, settings(sc, teams)).expect("loads"))
}

fn site_hp(scene: &Scene) -> i16 {
    scene.sim.as_ref().unwrap().mp.as_ref().unwrap().objects_rt.site.as_ref().unwrap().state
}

fn last_hitter(scene: &Scene) -> i16 {
    scene.sim.as_ref().unwrap().mp.as_ref().unwrap().objects_rt.site.as_ref().unwrap().last_hitter
}

fn game_state(scene: &Scene) -> u32 {
    scene.sim.as_ref().unwrap().mp.as_ref().unwrap().game.state
}

/// Arm Bond with `w` through the real inventory (the fire gate reads carried/clip/owned).
fn arm(sim: &mut Sim, w: u8) {
    sim.gun.weapon = w;
    sim.gun.switch_request = w;
    sim.gun.switch_state = 0;
    let at = sim.weapons.ammo_type(w as usize);
    if at < sim.drones.inventory.carried.len() { sim.drones.inventory.carried[at] = 999; }
    if (w as usize) < sim.drones.inventory.clip.len() { sim.drones.inventory.clip[w as usize] = 999; }
    if (w as usize) < sim.drones.inventory.owned.len() { sim.drones.inventory.owned[w as usize] = true; }
}

/// A floor spot `dist` from `target` with a clear head-to-target line (the online census's
/// firing spot; the site centre segment clears — the crate is not world geometry there).
fn firing_spot(sim: &Sim, target: [f32; 3], dist: f32) -> [f32; 3] {
    for k in 0..16 {
        let a = k as f32 * std::f32::consts::TAU / 16.0;
        let spot = [target[0] + dist * a.sin(), target[1] + 0.5, target[2] + dist * a.cos()];
        let b = nf_game::player::Bond::stand_at(&sim.world, spot, 0.0);
        if (b.pos[1] - target[1]).abs() > 1.0 {
            continue; // another floor
        }
        if sim.world.segment(b.head(), target).is_none() {
            return b.pos;
        }
    }
    panic!("no clear firing spot around {target:?}");
}

/// Stand Bond at `spot`, aim the head at `target` (yaw + the pitch the local pipeline's shot
/// rides — `Sim::step_bond`'s look is `bond.look()`, yaw + pitch).
fn stand_aim(scene: &mut Scene, spot: [f32; 3], target: [f32; 3]) {
    let sim = scene.sim.as_mut().unwrap();
    let mut b = nf_game::player::Bond::stand_at(&sim.world, spot, 0.0);
    let head = b.head();
    let dx = target[0] - spot[0];
    let dz = target[2] - spot[2];
    let dy = target[1] - head[1];
    let dh = (dx * dx + dz * dz).sqrt().max(1e-6);
    b.yaw = dx.atan2(dz);
    b.pitch = (dy / dh).atan2(1.0) / std::f32::consts::FRAC_PI_2;
    sim.bond = b;
}

/// One offline tick, re-aimed at `target`; `press` drives the trigger — the fire scripts are
/// per-press in the local pipeline (the state-9 restart gate [M 0x1a5460:664-672]), so the
/// siege pulses (press, release, press…).
fn tick_fire(scene: &mut Scene, target: [f32; 3], press: bool) {
    stand_aim(scene, scene.sim.as_ref().unwrap().bond.pos, target);
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), press, false, false);
}

/// `n` ticks of a 6-on/2-off trigger pulse (a fresh press restarts the fire script).
fn ticks_fire(scene: &mut Scene, n: u32, target: [f32; 3]) {
    for i in 0..n {
        tick_fire(scene, target, i % 8 < 6);
    }
}

/// Bond's strongest plain bullet: the record with the biggest +0x0C among the non-blast,
/// non-homing, ranged, multi-shot-capable records (the online census's `strongest_weapon`).
fn strongest_weapon(sim: &Sim) -> u8 {
    let w = &sim.weapons;
    let plain = |i: usize| {
        let r = &w.records[i];
        let f = |o: usize| u32::from_le_bytes(r[o..o + 4].try_into().unwrap());
        let range = f32::from_le_bytes(r[0x1c..0x20].try_into().unwrap());
        f(0x6c) & 0x8 == 0 && f(0x70) & (0x4 | 0x10_0000 | 0x20_0000) == 0 && f(0x68) & 0x2_0000 == 0 && range >= 10.0 && f(0x18) >= 1
    };
    (0..w.records.len().min(0x47))
        .filter(|&i| plain(i))
        .max_by(|&x, &y| {
            let d = |i: usize| f32::from_le_bytes(w.records[i][0x0c..0x10].try_into().unwrap());
            d(x).total_cmp(&d(y))
        })
        .unwrap() as u8
}

/// The offline siege: Bond's bullets dent the site (the per-frame chain sum) and the threshold
/// ends the round the game's way — the attacking team +1, the last hitter +1, state 6 [M
/// 0x1870a0:125-160]. Demolition: the defenders are team 0, the attacker (slot 0) team 1.
#[test]
fn bond_shots_damage_and_destroy_the_site_offline() {
    let Some(mut scene) = offline_load(scenario::DEMOLITION, [1, 0]) else { eprintln!("skip: NF_ISO not set"); return };
    let (centre, _r) = scene.sim.as_ref().unwrap().mp.as_ref().unwrap().site_sphere().expect("the live site");
    // The first contact: HP drops, the local slot is the last hitter [M :84-91]. The setup
    // borrows in scopes (the sim and the aim helpers share `scene`).
    let held = {
        let sim = scene.sim.as_mut().unwrap();
        strongest_weapon(sim)
    };
    let spot = {
        let sim = scene.sim.as_mut().unwrap();
        firing_spot(sim, centre, 4.0)
    };
    stand_aim(&mut scene, spot, centre);
    {
        let sim = scene.sim.as_mut().unwrap();
        arm(sim, held);
    }
    for t in 0..40 {
        tick_fire(&mut scene, centre, t % 8 < 6);
        if site_hp(&scene) < 2000 { break; }
        assert!(t < 39, "Bond's shots never dented the site");
    }
    assert!(site_hp(&scene) < 2000, "the shot damaged the site");
    assert_eq!(last_hitter(&scene), 0, "Bond (slot 0) is the last hitter");
    // The siege to the fall.
    for t in 0..4000 {
        if game_state(&scene) == 6 { break; }
        tick_fire(&mut scene, centre, t % 8 < 6);
        if std::env::var_os("NF_SITE_LOG").is_some() && t % 100 == 0 {
            let sim = scene.sim.as_ref().unwrap();
            eprintln!("[sitelog] t{t} hp {} clip {}", site_hp(&scene), sim.drones.inventory.clip[held as usize]);
        }
    }
    assert_eq!(game_state(&scene), 6, "the site fell: the round is over");
    let m = scene.sim.as_ref().unwrap().mp.as_ref().unwrap();
    assert_eq!(m.game.team_scores, [0.0, 1.0], "the attacking team (1) scores");
    assert_eq!(m.game.slots[0].score, 1.0, "the last hitter +1");
}

/// Sister: the shot that misses leaves the site's damage at 0 — HP 2000, no recorded hitter
/// (the port fires for real here: the tracer paths prove the gun ran).
#[test]
fn a_missed_shot_leaves_the_site_undamaged_offline() {
    let Some(mut scene) = offline_load(scenario::DEMOLITION, [1, 0]) else { eprintln!("skip: NF_ISO not set"); return };
    let (centre, _r) = scene.sim.as_ref().unwrap().mp.as_ref().unwrap().site_sphere().expect("the live site");
    let held = {
        let sim = scene.sim.as_mut().unwrap();
        strongest_weapon(sim)
    };
    // 12 units out, aimed directly AWAY from the site: the flight segment never approaches
    // the sphere.
    let away = [centre[0] + 12.0, centre[1] + 0.5, centre[2]];
    let far = [centre[0] + 60.0, away[1] + 0.5, centre[2]];
    stand_aim(&mut scene, away, far);
    {
        let sim = scene.sim.as_mut().unwrap();
        arm(sim, held);
    }
    ticks_fire(&mut scene, 30, [centre[0] + 60.0, centre[1] + 1.0, centre[2]]);
    {
        let sim = scene.sim.as_ref().unwrap();
        // The gun actually fired (else the HP check is vacuous): the fire pipeline spent rounds.
        let spent = sim.drones.inventory.clip[held as usize];
        assert!(spent < 999, "the gun actually fired (clip spent, found {spent})");
    }
    assert_eq!(site_hp(&scene), 2000, "the missed shot left the damage at 0");
    assert_eq!(last_hitter(&scene), -1, "no hitter recorded");
}

/// The explosive: the Sentinel Unguided 0x2d (the contact-blast rocket, radius 8, damage 100
/// [D the EE weapon table @ 0x2bf150 + 0x2d·0x10C]) fired at the site — the rocket's blast
/// applies `MP_ApplyDamage` directly (the case-0x35 path [M 0x17c518:187]) and the threshold
/// follows. The per-rocket dent is position-dependent (the falloff of where the rocket dies) —
/// only the drop is pinned, not the amount.
#[test]
fn an_explosive_damages_and_destroys_the_site_offline() {
    let Some(mut scene) = offline_load(scenario::DEMOLITION, [1, 0]) else { eprintln!("skip: NF_ISO not set"); return };
    let (centre, _r) = scene.sim.as_ref().unwrap().mp.as_ref().unwrap().site_sphere().expect("the live site");
    let spot = {
        let sim = scene.sim.as_mut().unwrap();
        firing_spot(sim, centre, 10.0)
    };
    stand_aim(&mut scene, spot, centre);
    {
        let sim = scene.sim.as_mut().unwrap();
        arm(sim, 0x2d);
    }
    // The first blast: HP drops (the blast reaches the site — the case-0x35 path ran). The
    // launcher's fire script is long — one 1-tick press every 80 ticks lets the script reach
    // its release frame (the shot spawns there) and the rocket fly its ~10 units.
    for t in 0..400 {
        tick_fire(&mut scene, centre, t % 80 == 0);
        if site_hp(&scene) < 2000 { break; }
        assert!(t < 399, "the rockets never damaged the site");
    }
    assert!(site_hp(&scene) < 2000, "the blast damaged the site");
    assert_eq!(last_hitter(&scene), 0, "Bond's blast recorded the hitter");
    for t in 0..5000 {
        if game_state(&scene) == 6 { break; }
        tick_fire(&mut scene, centre, t % 80 == 0);
    }
    assert_eq!(game_state(&scene), 6, "the site fell: the round is over");
    let m = scene.sim.as_ref().unwrap().mp.as_ref().unwrap();
    assert_eq!(m.game.team_scores, [0.0, 1.0], "the attacking team (1) scores");
    assert_eq!(m.game.slots[0].score, 1.0, "the last hitter +1");
}

// ------------------------------------------------------------------ the online agreement

/// The online pair machinery (the mp_scenarios_live shape, trimmed): two machines, the shared
/// combat owns the site, the hashes agree.
struct Pair {
    a: Scene,
    b: Scene,
    t: u32,
}

fn start_online(s: MpSettings) -> Option<Pair> {
    let path = iso()?;
    let arena = format!("{:08x}.bin", s.level_code);
    let mut a = nf_view::load_mp_carry(&path, &arena, false, None, s.clone()).expect("machine A loads");
    let mut b = nf_view::load_mp_carry(&path, &arena, false, None, s).expect("machine B loads");
    nf_view::net_slot0_setup(a.sim.as_mut().unwrap());
    let sb = b.sim.as_mut().unwrap();
    sb.swap_local_slot(1);
    sb.net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b ^ 1u32.wrapping_mul(0x9e37_79b9)));
    Some(Pair { a, b, t: 0 })
}

fn machine(sim: &mut Sim, local: usize, row: &[nf_net::Input; N]) -> u64 {
    let li = row[local];
    sim.net_row = Some(*row);
    sim.step_bond(to_player_input(&li), li.trigger, li.reload, li.use_key);
    let inputs = std::array::from_fn(|i| to_player_input(&row[i]));
    sim.step_net_remotes(&inputs, local);
    sim.step_net_combat(row, local);
    if let Some(m) = sim.mp.as_mut() {
        for sl in 0..N {
            if sl != local {
                if let Some(b) = &sim.mp_bonds[sl] { m.set_slot(sl, b.pos); }
            }
        }
        m.set_slot(local, sim.bond.pos);
    }
    sim.net_state_hash(local)
}

fn online_tick(p: &mut Pair, r: &[nf_net::Input; N]) {
    let t = p.t;
    let sa = p.a.sim.as_mut().unwrap();
    let ha = machine(sa, 0, r);
    let sb = p.b.sim.as_mut().unwrap();
    let hb = machine(sb, 1, r);
    assert_eq!(ha, hb, "tick {t}: the two machines' states agree");
    p.t += 1;
}

fn online_row_at(p: &Pair, shooter: usize, held: u8, target: [f32; 3]) -> [nf_net::Input; N] {
    let s = p.a.sim.as_ref().unwrap();
    let b = if shooter == s.mp_local_slot { s.bond.clone() } else { s.mp_bonds[shooter].clone().unwrap() };
    let h = b.head();
    let d = [target[0] - h[0], target[1] - h[1], target[2] - h[2]];
    let mut r = [nf_net::NEUTRAL; N];
    for i in r.iter_mut() { i.held = held; }
    r[shooter].shot = true;
    r[shooter].shot_weapon = held;
    r[shooter].shot_aim = nf_game::mp::combat::aim_from_dir(d);
    r
}

/// The online path still agrees: one shared-combat shot takes exactly ONE round's damage off
/// the site (the strongest plain bullet's +0x0C word) — the local pipeline's visual bullets
/// never feed it (the offline gate holds online), and the machines' hashes agree every tick.
#[test]
fn online_shot_takes_exactly_one_round_of_site_damage() {
    let Some(mut p) = start_online(settings(scenario::DEMOLITION, [1, 0])) else { eprintln!("skip: NF_ISO not set"); return };
    let held = {
        let sim = p.a.sim.as_ref().unwrap();
        strongest_weapon(sim)
    };
    let (centre, _r) = p.a.sim.as_ref().unwrap().mp.as_ref().unwrap().site_sphere().expect("the live site");
    // Slot 1 (the other machine's local) stands far away; slot 0 at a clear firing spot.
    let spot = firing_spot(p.a.sim.as_ref().unwrap(), centre, 4.0);
    for (scene, slot) in [(&mut p.a, 0usize), (&mut p.b, 1usize)] {
        let sim = scene.sim.as_mut().unwrap();
        let mut b = nf_game::player::Bond::stand_at(&sim.world, if slot == 0 { spot } else { [centre[0] + 30.0, centre[1] + 0.5, centre[2]] }, 0.0);
        if slot == 0 {
            let head = b.head();
            let dx = centre[0] - spot[0];
            let dz = centre[2] - spot[2];
            b.yaw = dx.atan2(dz);
            let dy = centre[1] - head[1];
            b.pitch = (dy / dx.hypot(dz)).atan2(1.0) / std::f32::consts::FRAC_PI_2;
        }
        if slot == 0 {
            sim.bond = b;
        } else {
            sim.mp_bonds[slot] = Some(b);
        }
        let m = sim.mp.as_mut().unwrap();
        m.set_slot(slot, if slot == 0 { sim.bond.pos } else { sim.mp_bonds[slot].as_ref().unwrap().pos });
    }
    online_tick(&mut p, &[nf_net::NEUTRAL; N]);
    let before = p.a.sim.as_ref().unwrap().mp.as_ref().unwrap().objects_rt.site.as_ref().unwrap().state;
    let r = online_row_at(&p, 0, held, centre);
    for _ in 0..12 {
        online_tick(&mut p, &r);
    }
    let after = p.a.sim.as_ref().unwrap().mp.as_ref().unwrap().objects_rt.site.as_ref().unwrap().state;
    let dmg = {
        let w = &p.a.sim.as_ref().unwrap().weapons;
        f32::from_le_bytes(w.records[held as usize][0x0c..0x10].try_into().unwrap())
    };
    assert_eq!(before - after, dmg as i16, "one round's damage, once");
}
