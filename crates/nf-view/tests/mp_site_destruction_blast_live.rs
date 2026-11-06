//! mp-site-destruction-blast: the Demolition/Protection site's death blast —
//! `MP_DemolitionProtectionUpdate` 0x1870a0:144-146's
//! `Explode_Create(5.0, 5.0, 500.0, site_obj, obj+0x80, 0x2d6210, 0x600004f, 0xff, 0xff, 100, 0)`
//! (float args 5.0 scale, 5.0 radius, 500.0 damage, pos a1 = the site's obj+0x80 — the contact
//! sphere's world centre; the attacker t3 = 0 NULL [M 0x187474 `daddu t3,zero,zero`]). The port
//! queues it at the death (`MpObjectsRt::pending_blasts`) and propagates it the NEXT tick
//! through the shared blast pass — the game's explosion object runs its contact on its first
//! update [M Explode_CollisionHandler 0x17cde0's one-shot latch]: online through
//! `MpCombat::step`'s case-3/case-5 pass (every machine, the lockstep hash), offline through the
//! drones' shared pending_blasts slot (the copter death blast's route). Sisters:
//! mp_site_damage_offline_live.rs (the site's damage feeds), mp_scenarios_live.rs (the rig).
//! NF_ISO-gated; arena 07000023.

use nf_game::mp::{scenario, MpSettings};
use nf_view::mpnet::to_player_input;
use nf_view::scene::{Scene, Sim};

const ARENA: u32 = 0x0700_0023;
const N: usize = nf_net::MAX_AGENTS;
/// `Explode_Create`'s call-site constants [M 0x1870a0:144-146 + disasm 0x187434-0x187474].
const BLAST_RADIUS: f32 = 5.0; // f13 = 0x40a00000
const BLAST_DAMAGE: f32 = 500.0; // f14 = 0x43fa0000

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

/// The online rig's settings: a 60-min round — the negative limit defaults 60.0 s for
/// demo/prot [M 0x183438:51-54], and the siege must outlive it (the timeout branch scores the
/// defenders with NO blast [M 0x1870a0:112-124]).
fn online_settings(sc: u32, teams: [u32; 2]) -> MpSettings {
    let mut s = settings(sc, teams);
    s.time_limit = 3600; // seconds — the field is the raw limit word [M 0x183438:39]
    s
}

// ---------------------------------------------------------------- the offline rig
// (the mp_site_damage_offline_live.rs shapes: the local pipeline, Bond's real fire path)

fn offline_load(sc: u32, teams: [u32; 2]) -> Option<Scene> {
    let path = iso()?;
    Some(nf_view::load_mp_carry(&path, &format!("{ARENA:08x}.bin"), false, None, settings(sc, teams)).expect("loads"))
}

fn site_hp(scene: &Scene) -> i16 {
    scene.sim.as_ref().unwrap().mp.as_ref().unwrap().objects_rt.site.as_ref().unwrap().state
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

/// A floor spot `dist` from `target` with a clear head-to-target line.
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

/// Stand Bond at `spot`, aim the head at `target`.
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

/// One offline tick, re-aimed at `target`; `press` drives the trigger.
fn tick_fire(scene: &mut Scene, target: [f32; 3], press: bool) {
    stand_aim(scene, scene.sim.as_ref().unwrap().bond.pos, target);
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), press, false, false);
}

/// Bond's strongest plain bullet (the mp_site_damage_offline_live census shape).
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

/// The last `drones.explosions` record — `spawn_blast`'s (pos, radius, damage, owner, weapon).
fn last_explosion(scene: &Scene) -> ([f32; 3], f32, f32, usize, u16) {
    *scene.sim.as_ref().unwrap().drones.explosions.last().expect("a blast fired")
}

/// The siege to the fall from `dist` (safe-range: 8.0 keeps the shooter outside the 5.0 blast),
/// then the death tick's follow-ups. Returns the tick count used.
fn siege_to_death(scene: &mut Scene, sc: u32, teams: [u32; 2]) {
    let Some(_) = iso() else { return };
    let (centre, _r) = scene.sim.as_ref().unwrap().mp.as_ref().unwrap().site_sphere().expect("the live site");
    let held = strongest_weapon(scene.sim.as_ref().unwrap());
    let spot = firing_spot(scene.sim.as_ref().unwrap(), centre, 8.0);
    stand_aim(scene, spot, centre);
    arm(scene.sim.as_mut().unwrap(), held);
    for t in 0..6000 {
        if game_state(scene) == 6 { return; }
        tick_fire(scene, centre, t % 8 < 6);
    }
    panic!("{sc:?} siege never felled the site (teams {teams:?})");
}

/// Demolition, offline: the site falls to Bond's real bullets and the death blast fires the
/// NEXT tick at the site's obj+0x80 — the drones' `explosions` record carries the game's exact
/// call-site constants (pos = the site sphere's centre, 5.0 / 500.0 [M 0x1870a0:144-146], the
/// NULL-attacker mark), and the shooter standing outside the 5.0 radius takes nothing
/// (`blast_falloff`'s truncation: 500·(1 − trunc(8)/5) ≤ 0).
#[test]
fn the_death_blast_fires_at_the_site_offline() {
    let Some(mut scene) = offline_load(scenario::DEMOLITION, [1, 0]) else { eprintln!("skip: NF_ISO not set"); return };
    let (centre, _r) = scene.sim.as_ref().unwrap().mp.as_ref().unwrap().site_sphere().expect("the live site");
    let health_before = scene.sim.as_ref().unwrap().drones.health.health;
    assert!(scene.sim.as_ref().unwrap().drones.explosions.is_empty(), "no blast before the fall");
    siege_to_death(&mut scene, scenario::DEMOLITION, [1, 0]);
    // The death tick queued the blast; the NEXT tick's drones pass propagates it.
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    let (pos, radius, damage, owner, _weapon) = last_explosion(&scene);
    assert_eq!(radius, BLAST_RADIUS, "the blast radius is the call's f13 [M 0x1870a0:144-146]");
    assert_eq!(damage, BLAST_DAMAGE, "the blast damage is the call's f14 [M 0x1870a0:144-146]");
    for k in 0..3 {
        assert!((pos[k] - centre[k]).abs() < 1e-3, "the blast sits at the site's obj+0x80 (axis {k}: {pos:?} vs {centre:?}) [M :145 `addiu a1,s4,128`]");
    }
    assert_eq!(owner, usize::MAX - 1, "the NULL attacker rides the copter's not-the-player mark [M 0x187474 t3 = 0]");
    // The shooter at 8.0 stands outside the 5.0 blast: unharmed.
    let after = scene.sim.as_ref().unwrap().drones.health.health;
    assert_eq!(after, health_before, "the shooter outside the radius takes nothing");
    // The round ended with the site down [M :160] and the last-hitter label stands [M :146-158:
    // the hitter's team word != 0 → label 0x200002e].
    assert_eq!(game_state(&scene), 6, "the round is over [M 0x1870a0:160]");
    assert_eq!(scene.sim.as_ref().unwrap().mp.as_ref().unwrap().objects_rt.site_status, Some(0x200002e));
}

/// Demolition, offline: a body INSIDE the 5.0 blast takes the damage — Bond parked 2.0 from the
/// centre when the site falls eats the falloff (500·(1 − trunc(2.0 − 0)/5) = 300 raw before the
/// pain multipliers — a kill from full health). The game's own flow: the bomber shoots from
/// point-blank and the blast catches him ([M] the propagation itself:
/// `Explode_Propagate` 0x17c518 case 3 → `Player_Hurt`).
#[test]
fn the_death_blast_hurts_a_body_inside_the_radius_offline() {
    let Some(mut scene) = offline_load(scenario::DEMOLITION, [1, 0]) else { eprintln!("skip: NF_ISO not set"); return };
    let (centre, _r) = scene.sim.as_ref().unwrap().mp.as_ref().unwrap().site_sphere().expect("the live site");
    let held = strongest_weapon(scene.sim.as_ref().unwrap());
    let spot = firing_spot(scene.sim.as_ref().unwrap(), centre, 8.0);
    stand_aim(&mut scene, spot, centre);
    arm(scene.sim.as_mut().unwrap(), held);
    for t in 0..6000 {
        if game_state(&scene) == 6 { break; }
        tick_fire(&mut scene, centre, t % 8 < 6);
    }
    assert_eq!(game_state(&scene), 6, "the site fell");
    // Step into the blast radius before the next tick's propagation.
    let close = nf_game::player::Bond::stand_at(&scene.sim.as_ref().unwrap().world, [centre[0] + 2.0, centre[1] + 0.5, centre[2]], 0.0);
    scene.sim.as_mut().unwrap().bond = close;
    let before = scene.sim.as_ref().unwrap().drones.health.health;
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    let after = sim.drones.health.health;
    assert!(after <= before - 100.0, "the blast hurt the body inside the radius ({before} -> {after})");
}

/// Protection (0x20000080), offline: the same death blast through the Protection site — the
/// defending team is 1 there [M 0x18b198 case 8 → param_3 = 1], the hitter's team word 0 shows
/// label 0x200002f [M :149-151], and the blast's constants are the call's.
#[test]
fn protection_site_death_blast_offline() {
    let Some(mut scene) = offline_load(scenario::PROTECTION, [0, 1]) else { eprintln!("skip: NF_ISO not set"); return };
    let (centre, _r) = scene.sim.as_ref().unwrap().mp.as_ref().unwrap().site_sphere().expect("the live site");
    let health_before = scene.sim.as_ref().unwrap().drones.health.health;
    siege_to_death(&mut scene, scenario::PROTECTION, [0, 1]);
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    let (pos, radius, damage, _owner, _weapon) = last_explosion(&scene);
    assert_eq!(radius, BLAST_RADIUS, "the Protection blast radius is the call's f13");
    assert_eq!(damage, BLAST_DAMAGE, "the Protection blast damage is the call's f14");
    for k in 0..3 {
        assert!((pos[k] - centre[k]).abs() < 1e-3, "the blast sits at the Protection site's obj+0x80 (axis {k})");
    }
    assert_eq!(game_state(&scene), 6, "the round is over [M 0x1870a0:160]");
    assert_eq!(scene.sim.as_ref().unwrap().mp.as_ref().unwrap().objects_rt.site_status, Some(0x200002f), "team 0's label [M :149-151]");
    let after = scene.sim.as_ref().unwrap().drones.health.health;
    assert_eq!(after, health_before, "the 8.0-out shooter takes nothing");
}

// ---------------------------------------------------------------- the online rig
// (the mp_scenarios_live.rs shapes: two machines, the lockstep hash every tick)

/// The two machines: `a` plays slot 0, `b` plays slot 1.
struct Pair {
    a: Scene,
    b: Scene,
    t: u32,
}

impl Pair {
    fn sims(&mut self) -> [(&mut Sim, usize); 2] {
        [(self.a.sim.as_mut().unwrap(), 0), (self.b.sim.as_mut().unwrap(), 1)]
    }
    fn m(&self) -> &nf_game::mp::MpMatch {
        self.a.sim.as_ref().unwrap().mp.as_ref().unwrap()
    }
    fn sim(&self) -> &Sim {
        self.a.sim.as_ref().unwrap()
    }
}

fn start(s: MpSettings) -> Option<Pair> {
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

/// One machine's online tick — the live loop's shape (lib.rs, the net branch).
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
                if let Some(b) = &sim.mp_bonds[sl] {
                    m.set_slot(sl, b.pos);
                }
            }
        }
        m.set_slot(local, sim.bond.pos);
    }
    sim.net_state_hash(local)
}

fn row_at(shooter: usize, held: u8, sim: &Sim, target: [f32; 3]) -> [nf_net::Input; N] {
    let b = if shooter == sim.mp_local_slot { sim.bond.clone() } else { sim.mp_bonds[shooter].clone().unwrap() };
    let h = b.head();
    let d = [target[0] - h[0], target[1] - h[1], target[2] - h[2]];
    let mut r = [nf_net::NEUTRAL; N];
    for i in r.iter_mut() { i.held = held; }
    r[shooter].shot = true;
    r[shooter].shot_weapon = held;
    r[shooter].shot_aim = nf_game::mp::combat::aim_from_dir(d);
    r
}

/// One lockstep tick on both machines; the hashes must agree.
fn tick(p: &mut Pair, ra: &[nf_net::Input; N], rb: &[nf_net::Input; N]) {
    let t = p.t;
    let [(sa, la), (sb, lb)] = p.sims();
    let ha = machine(sa, la, ra);
    let hb = machine(sb, lb, rb);
    assert_eq!(ha, hb, "tick {t}: the two machines' states agree");
    p.t += 1;
}

/// Teleport a slot's body (both machines, identically).
fn place(p: &mut Pair, slot: usize, pos: [f32; 3], yaw: f32) {
    for (sim, local) in p.sims() {
        let b = nf_game::player::Bond::stand_at(&sim.world, [pos[0], pos[1] + 0.5, pos[2]], yaw);
        let bp = b.pos;
        if slot == local {
            sim.bond = b;
        } else {
            sim.mp_bonds[slot] = Some(b);
        }
        sim.mp.as_mut().unwrap().set_slot(slot, bp);
    }
}

/// Demolition, online: the site falls to slot 0's shared-combat siege and the death blast —
/// drained from the match's pending queue into the shared blast pass the NEXT tick — kills the
/// slot 1 body parked 2.0 from the centre, on BOTH machines, with the lockstep hash agreeing
/// every tick. The blast's attacker is the game's NULL [M 0x187474], so the kill credits
/// nobody: no kill message, no score beyond the site's own +1 [M 0x17c518:128-134's gate].
#[test]
fn the_death_blast_kills_a_nearby_body_online_in_step() {
    let Some(mut p) = start(online_settings(scenario::DEMOLITION, [1, 0])) else { eprintln!("skip: NF_ISO not set"); return };
    let (centre, _r) = p.m().site_sphere().expect("the live site");
    let held = {
        let w = &p.sim().weapons;
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
    };
    // Slot 0 sieges from 8.0 (outside the 5.0 blast); slot 1 parks 2.0 from the centre — inside.
    let siege_spot = {
        let sim = p.sim();
        let mut found = None;
        for k in 0..16 {
            let a = k as f32 * std::f32::consts::TAU / 16.0;
            let spot = [centre[0] + 8.0 * a.sin(), centre[1] + 0.5, centre[2] + 8.0 * a.cos()];
            let b = nf_game::player::Bond::stand_at(&sim.world, spot, 0.0);
            if (b.pos[1] - centre[1]).abs() > 1.0 { continue; }
            if sim.world.segment(b.head(), centre).is_none() { found = Some(b.pos); break; }
        }
        found.expect("no siege spot")
    };
    let near = nf_game::player::Bond::stand_at(&p.sim().world, [centre[0] + 2.0, centre[1] + 0.5, centre[2]], 0.0).pos;
    place(&mut p, 0, siege_spot, (centre[0] - siege_spot[0]).atan2(centre[2] - siege_spot[2]));
    place(&mut p, 1, near, 0.0);
    // Arm slot 0's shared fire gate on BOTH machines, identically (the gate state is hashed):
    // the siege weapon owned, its clip slot and the carried ammo topped — the spawn loadout
    // runs dry long before 2000 HP and the gate drops dry shots.
    for (sim, _) in p.sims() {
        let at = sim.weapons.ammo_type(held as usize);
        let cs = nf_game::pickup::Inventory::clip_slot(&sim.weapons, held as usize);
        if let Some(g) = sim.mp_combat.gates.get_mut(0) {
            if cs < g.inv.clip.len() { g.inv.clip[cs] = 999; }
            if at < g.inv.carried.len() { g.inv.carried[at] = 999; }
            if (held as usize) < g.inv.owned.len() { g.inv.owned[held as usize] = true; }
        }
    }
    // Besiege: slot 0 fires at the site every tick (the shot description rides the row).
    for t in 0..40000u32 {
        if p.m().game.state == 6 { break; }
        let ra = row_at(0, held, p.sim(), centre);
        let rb = row_at(0, held, p.b.sim.as_ref().unwrap(), centre);
        tick(&mut p, &ra, &rb);
        if t % 5000 == 0 {
            let s = p.sim();
            let m = s.mp.as_ref().unwrap();
            eprintln!("[blastdbg] t{t} hp {} clock {} rounds {}", m.objects_rt.site.as_ref().unwrap().state, m.game.clock, s.mp_combat.rounds.len());
        }
    }
    assert_eq!(p.m().game.state, 6, "the site fell online");
    {
        let s = p.sim();
        let m = s.mp.as_ref().unwrap();
        let site = m.objects_rt.site.as_ref().unwrap();
        eprintln!("[blastdbg] site hp {} pos {:?} bounds {:?}", site.state, site.pos, site.bounds.map(|b| (b.centre, b.radius)));
        eprintln!("[blastdbg] slot1 body {:?} hp {} dead_ticks {}", s.mp_bonds[1].as_ref().map(|b| b.pos), s.mp_hp[1], s.mp_dead_ticks[1]);
        eprintln!("[blastdbg] pending {} clock {}", m.objects_rt.pending_blasts.len(), m.game.clock);
    }
    assert!(p.m().objects_rt.site.as_ref().unwrap().state < 1, "the site fell to the bullets, not the timeout (HP {})", p.m().objects_rt.site.as_ref().unwrap().state);
    let deaths1 = p.m().game.slots[1].deaths;
    let score0 = p.m().game.slots[0].score;
    // The death tick queued the blast; the NEXT tick's shared combat pass kills slot 1.
    let ra = [nf_net::NEUTRAL; N];
    let rb = [nf_net::NEUTRAL; N];
    tick(&mut p, &ra, &rb);
    for sim in [&p.a, &p.b] {
        let s = sim.sim.as_ref().unwrap();
        assert_eq!(s.mp_hp[1], 0.0, "the blast killed the body inside the radius");
        assert!(s.mp_dead_ticks[1] > 0 && s.mp_dead_ticks[1] < 300, "the death latch is set (the tick's respawn pass advances it past 1) (frame {})", s.drones.frame);
    }
    assert_eq!(p.m().game.slots[1].deaths, deaths1 + 1, "the death scored once");
    assert!((p.m().game.slots[0].score - score0).abs() < 1e-4, "the NULL-attacker blast credits nobody");
    assert!(p.m().kill_msgs.iter().all(|&(_, v)| v != 1), "no kill message for the NULL-attacker kill");
    // The next ticks stay in step (the respawn machinery, the queue empty).
    for _ in 0..5 {
        tick(&mut p, &ra, &rb);
    }
}
