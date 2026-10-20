//! mp-scenarios-online (agent6): every MP scenario played online in lockstep — two machines (slot
//! 0's and slot 1's), each a full arena sim from the same settings + seed (`load_mp_carry`), fed
//! the SAME per-tick input rows through the live loop's tick shape (the local slot's full
//! `step_bond`, `step_net_remotes`, `step_net_combat`, the slot positions, `net_state_hash`). The
//! hashes must agree EVERY tick, and each scenario's objective must play out to the game's result:
//! the bodies are teleported onto the objective (identically on both machines — the stand-in for
//! walking there) and the scenario objects react through the port's `MP_HitBy` contact walk, the
//! shots through the online combat. Rules cite [M] (the decomp lines in
//! `docs/systems/mp-match-rules.md`); port choices are [S]. NF_ISO-gated; arena 07000023 carries
//! every scenario's objects (the census test below).

use nf_game::mp::{scenario, MpEndStatus, MpSettings};
use nf_view::mpnet::to_player_input;
use nf_view::scene::{Scene, Sim};

const ARENA: u32 = 0x0700_0023;
const N: usize = nf_net::MAX_AGENTS;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

fn settings(sc: u32, frag: i32, time_min: i32, teams: [u32; 2]) -> MpSettings {
    let mut s = MpSettings::deathmatch(ARENA);
    s.scenario = sc;
    s.frag_limit = frag;
    s.time_limit = time_min;
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

/// The two machines: `a` plays slot 0, `b` plays slot 1 (the live match start's setup: the
/// joiner swaps its slot's body in; each machine's private generator).
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
    fn mb(&self) -> &nf_game::mp::MpMatch {
        self.b.sim.as_ref().unwrap().mp.as_ref().unwrap()
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
    // mp-world-sync: the live loop hands `step_bond` the row (the remotes move inside it; the
    // explicit call below is then a no-op).
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

fn row(shooter: Option<usize>, held: u8) -> [nf_net::Input; N] {
    let mut r = [nf_net::NEUTRAL; N];
    for i in r.iter_mut() {
        i.held = held;
    }
    if let Some(s) = shooter {
        r[s].shot = true;
    }
    r
}

/// mp-combat-weapons: a row where `shooter` fires `held` AT `target` (the shot description the
/// live app sends: the fired record + the aim from the shooter's head). The old rows' bare
/// `shot` flag fired along the body's view — fine at one height, but the online combat now hits
/// the real body volume (feet to head) instead of the old spheres that sat 0.5-1.5 ABOVE
/// `Bond::pos` (mostly over the head), so a horizontal eye-height shot misses a victim standing
/// a little lower.
fn row_at(p: &Pair, shooter: usize, held: u8, target: [f32; 3]) -> [nf_net::Input; N] {
    let s = p.sim();
    let b = if shooter == s.mp_local_slot { s.bond.clone() } else { s.mp_bonds[shooter].clone().unwrap() };
    let h = b.head();
    let d = [target[0] - h[0], target[1] - h[1], target[2] - h[2]];
    let mut r = row(Some(shooter), held);
    r[shooter].shot_weapon = held;
    r[shooter].shot_aim = nf_game::mp::combat::aim_from_dir(d);
    r
}

/// One lockstep tick on both machines; the hashes must agree.
fn tick(p: &mut Pair, r: &[nf_net::Input; N]) {
    let t = p.t;
    let [(sa, la), (sb, lb)] = p.sims();
    let ha = machine(sa, la, r);
    let hb = machine(sb, lb, r);
    assert_eq!(ha, hb, "tick {t}: the two machines' states agree");
    p.t += 1;
}

fn ticks(p: &mut Pair, n: u32, r: &[nf_net::Input; N]) {
    for _ in 0..n {
        tick(p, r);
    }
}

/// Teleport a slot's body (both machines, identically — the stand-in for walking there): stood
/// on the floor below `pos` like every MP spawn (`Bond::stand_at`).
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

fn body_pos(p: &Pair, slot: usize) -> [f32; 3] {
    let s = p.sim();
    if slot == s.mp_local_slot { s.bond.pos } else { s.mp_bonds[slot].as_ref().unwrap().pos }
}

/// A firing spot `dist` from `target` with a clear line from the head to it (the world segment),
/// and the yaw that faces it (the combat ray's `dir = (sin yaw, ·, cos yaw)`).
fn firing_spot(p: &Pair, target: [f32; 3], aim_y: f32, dist: f32) -> ([f32; 3], f32) {
    let sim = p.sim();
    for k in 0..16 {
        let a = k as f32 * std::f32::consts::TAU / 16.0;
        let spot = [target[0] + dist * a.sin(), target[1] + 0.5, target[2] + dist * a.cos()];
        let b = nf_game::player::Bond::stand_at(&sim.world, spot, 0.0);
        if (b.pos[1] - target[1]).abs() > 1.0 {
            continue; // another floor
        }
        let head = b.head();
        let aim = [target[0], aim_y, target[2]];
        if sim.world.segment(head, aim).is_none() {
            let yaw = (target[0] - b.pos[0]).atan2(target[2] - b.pos[2]);
            return (b.pos, yaw);
        }
    }
    panic!("no clear firing spot around {target:?}");
}

/// The weapon whose record deals the most damage (`weapon_data + id·0x10C + 0x0C` — the online
/// combat's damage word) — the site/body kills take fewer ticks. mp-combat-weapons: among the
/// plain bullet records that reach the 4 u firing spot (no grenade-family throw, no contact
/// blast — those now fly/blast as in the game and would take the shooter with them).
fn strongest_weapon(p: &Pair) -> u8 {
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
}

/// Shoot `victim` from a clear spot until the victim's slot dies (bounded).
fn shoot_dead(p: &mut Pair, shooter: usize, victim: usize, held: u8) {
    let vpos = body_pos(p, victim);
    let (spot, yaw) = firing_spot(p, vpos, vpos[1] + 0.7, 3.0);
    place(p, shooter, spot, yaw);
    let deaths = p.m().game.slots[victim].deaths;
    for _ in 0..600 {
        let v = body_pos(p, victim);
        let r = row_at(p, shooter, held, [v[0], v[1] + 0.2, v[2]]);
        tick(p, &r);
        if p.m().game.slots[victim].deaths > deaths {
            return;
        }
    }
    panic!("slot {shooter} never killed slot {victim}");
}

fn far_from(p: &Pair, avoid: &[[f32; 3]], team: u32) -> [f32; 3] {
    // the team's registered spawn point farthest from every `avoid` point
    let m = p.m();
    let (lo, hi) = if m.settings.team_game != 0 { if team == 0 { (0, 32) } else { (32, 64) } } else { (0, 64) };
    let mut best = (f32::MIN, [0.0; 3]);
    for sp in &m.spawns.points[lo..hi] {
        if sp.used == 0 {
            continue;
        }
        let d = avoid.iter().map(|a| (a[0] - sp.pos[0]).powi(2) + (a[2] - sp.pos[2]).powi(2)).fold(f32::MAX, f32::min);
        if d > best.0 {
            best = (d, [sp.pos[0], sp.pos[1] - 1.6, sp.pos[2]]);
        }
    }
    best.1
}

/// Runs the end machine on: state 1/2 → 3 → 4 → 5 (the debriefing request) [M 0x184bd0].
fn run_to_end(p: &mut Pair, limit: u32) {
    for _ in 0..limit {
        if p.m().game.state == 5 {
            return;
        }
        tick(p, &row(None, 2));
    }
    panic!("the match never reached the debriefing (state {})", p.m().game.state);
}

// ------------------------------------------------------------------------------------- tests

/// Census: arena 07000023 registers every scenario object type (CTF flags/bases 0/1, uplinks 2,
/// the demolition site 3, espionage bases 4 + blueprints 5, GoldenEye items 6/7, the protection
/// site 8, the KOTH hill 9 [M `MP_RegisterMPObject` 0x185990]) WITH model bounds [D the placement
/// hashcodes resolve in the arena's MapObjects]. Sister: a bare `register_mp_object` record has
/// none.
#[test]
fn arena_carries_every_scenario_object_with_bounds() {
    for (sc, what) in [
        (scenario::CAPTURE_THE_FLAG, "ctf"),
        (scenario::UPLINK, "uplink"),
        (scenario::DEMOLITION, "demolition"),
        (scenario::PROTECTION, "protection"),
        (scenario::BLUEPRINT, "blueprint"),
        (scenario::GOLDENEYE, "goldeneye"),
        (scenario::KOTH_FFA, "koth"),
    ] {
        let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
        let scene = nf_view::load_mp_carry(&path, &format!("{ARENA:08x}.bin"), false, None, settings(sc, 5, -1, [0, 1])).expect("loads");
        let o = &scene.sim.as_ref().unwrap().mp.as_ref().unwrap().objects_rt;
        match sc {
            scenario::CAPTURE_THE_FLAG => {
                assert!(o.flags.iter().all(|f| f.as_ref().is_some_and(|f| f.bounds.is_some())), "{what}: both flags + bounds");
                assert!(o.bases.iter().all(|b| b.is_some()) && o.bases_bounds.iter().all(|b| b.is_some()), "{what}: both bases");
            }
            scenario::UPLINK => assert!(o.uplinks.len() == 3 && o.uplinks.iter().all(|u| u.bounds.is_some() && u.state == 2), "{what}: 3 neutral terminals"),
            scenario::DEMOLITION | scenario::PROTECTION => {
                let s = o.site.as_ref().expect("the site");
                assert!(s.bounds.is_some() && s.state == 2000 && s.last_hitter == -1, "{what}: the site, HP 2000, no hitter");
            }
            scenario::BLUEPRINT => {
                assert!(o.blueprint.as_ref().is_some_and(|b| b.bounds.is_some()), "{what}: the blueprint");
                assert!(o.esp_bases.iter().all(|b| b.is_some()) && o.esp_bases_bounds.iter().all(|b| b.is_some()), "{what}: both bases");
            }
            scenario::GOLDENEYE => assert!(o.ge.iter().all(|g| g.as_ref().is_some_and(|g| g.bounds.is_some())), "{what}: both items"),
            _ => assert!(o.hill.as_ref().is_some_and(|h| h.bounds.is_some()), "{what}: the hill + its box"),
        }
    }
    // sister: the plain registration carries no bounds (the headless rigs' contact-free records)
    let mut m = nf_game::mp::MpMatch::init(settings(scenario::KOTH_FFA, 5, -1, [0, 1]));
    m.register_mp_object([0.0; 3], [0.0; 3], 9, [9, 2, 0, 0, 0, 0, 0], 0);
    assert!(m.objects.created[0].bounds.is_none());
}

/// The hash now folds the scenario objects: one machine's flag state poked → the very next
/// exchange differs (before this task `net_state_hash` folded only the slots — a CTF desync was
/// invisible). Sister of every test below.
#[test]
fn a_scenario_object_desync_is_detected() {
    let Some(mut p) = start(settings(scenario::CAPTURE_THE_FLAG, 5, -1, [0, 1])) else { eprintln!("skip: NF_ISO not set"); return };
    ticks(&mut p, 5, &row(None, 2));
    p.b.sim.as_mut().unwrap().mp.as_mut().unwrap().objects_rt.flags[0].as_mut().unwrap().counter = 7;
    let r = row(None, 2);
    let [(sa, la), (sb, lb)] = p.sims();
    assert_ne!(machine(sa, la, &r), machine(sb, lb, &r), "a flag-counter difference shows in the hash");
    // and a team score
    let Some(mut q) = start(settings(scenario::TEAM_DEATHMATCH, 5, -1, [0, 1])) else { return };
    q.b.sim.as_mut().unwrap().mp.as_mut().unwrap().game.team_scores[1] = 1.0;
    let [(sa, la), (sb, lb)] = q.sims();
    assert_ne!(machine(sa, la, &r), machine(sb, lb, &r), "a team-score difference shows in the hash");
}

/// Arena (Deathmatch 0x1): slot 0 shoots slot 1 dead through the online combat — +1 score/kill
/// [M 0x188f70:157-184]; frag limit 1 latches [M 0x184268:170-172] and the FFA who-won names the
/// unique qualifier [M 0x1848d0:55-86]. Sister: the victim scores nothing.
#[test]
fn arena_deathmatch_online() {
    let Some(mut p) = start(settings(scenario::DEATHMATCH, 1, -1, [0, 0])) else { eprintln!("skip: NF_ISO not set"); return };
    shoot_dead(&mut p, 0, 1, 2);
    let g = &p.m().game;
    assert_eq!((g.slots[0].kills, g.slots[0].score, g.slots[1].deaths), (1, 1.0, 1));
    assert_eq!(g.slots[1].score, 0.0, "the victim scores nothing");
    run_to_end(&mut p, 20);
    assert_eq!(p.m().game.end_status, MpEndStatus::Won("Alpha".into()));
    assert_eq!(p.mb().game.end_status, p.m().game.end_status, "both machines crown the same winner");
}

/// Team Arena (0x20000002): an enemy kill adds |Δ| to the killer's team [M 0x188f70:246-274]; the
/// team score reaches the limit [M 0x184268:147-172]; team games end with the generic text — the
/// scoreboard judges [M 0x1848d0:87-92]. Sister: the victim's team stays 0.
#[test]
fn team_arena_online() {
    let Some(mut p) = start(settings(scenario::TEAM_DEATHMATCH, 1, -1, [0, 1])) else { eprintln!("skip: NF_ISO not set"); return };
    shoot_dead(&mut p, 0, 1, 2);
    assert_eq!(p.m().game.team_scores, [1.0, 0.0]);
    run_to_end(&mut p, 20);
    assert_eq!(p.m().game.end_status, MpEndStatus::Draw, "the team branch's generic text");
    assert!(p.m().game.team_scores[0] > p.m().game.team_scores[1], "team 0 wins on the board");
}

/// Capture the Flag (0x20000004): slot 1 (team 1) on its own flag does nothing; slot 0 (team 0)
/// walks onto team 1's flag — the contact takes it [M 0x1861a8:170-190]; carried to team 0's
/// base it scores team 0 +1 and the carrier +1 and the flag goes home [M :74-112]; limit 1 ends.
#[test]
fn capture_the_flag_online() {
    let Some(mut p) = start(settings(scenario::CAPTURE_THE_FLAG, 1, -1, [0, 1])) else { eprintln!("skip: NF_ISO not set"); return };
    let (flag1, base0) = {
        let o = &p.m().objects_rt;
        (o.flags[1].as_ref().unwrap().home, o.bases[0].unwrap())
    };
    place(&mut p, 1, flag1, 0.0); // the owner on its own flag at home
    ticks(&mut p, 10, &row(None, 2));
    assert_eq!(p.m().objects_rt.flags[1].as_ref().unwrap().state, 0, "the owner cannot take its own flag");
    let away = far_from(&p, &[flag1, base0], 1);
    place(&mut p, 1, away, 0.0);
    place(&mut p, 0, flag1, 0.0);
    ticks(&mut p, 2, &row(None, 2));
    let f = p.m().objects_rt.flags[1].as_ref().unwrap();
    assert_eq!((f.state, f.carrier), (1, Some(0)), "the enemy contact takes the flag");
    place(&mut p, 0, base0, 0.0);
    ticks(&mut p, 2, &row(None, 2));
    assert_eq!(p.m().game.team_scores, [1.0, 0.0], "the capture scores the taker's team");
    assert_eq!(p.m().game.slots[0].score, 1.0, "and the carrier");
    assert_eq!(p.m().objects_rt.flags[1].as_ref().unwrap().state, 0, "the flag reset home");
    run_to_end(&mut p, 20);
}

/// Uplink (0x60000008): slot 0 claims a neutral terminal [M 0x186d98:83-98]; slot 1 (team 1)
/// standing on it with the defender does NOT flip it [M :24-26]; the defender leaves → it flips
/// to team 1 [M :26-52]; the owner team drips and its score is the truncated Σ of its members'
/// personals [M :59-80]; limit 1 ends.
#[test]
fn uplink_online() {
    let Some(mut p) = start(settings(scenario::UPLINK, 1, -1, [0, 1])) else { eprintln!("skip: NF_ISO not set"); return };
    let term = p.m().objects_rt.uplinks[0].pos;
    let elsewhere: Vec<[f32; 3]> = p.m().objects_rt.uplinks.iter().map(|u| u.pos).collect();
    let away1 = far_from(&p, &elsewhere, 1);
    place(&mut p, 1, away1, 0.0);
    place(&mut p, 0, term, 0.0);
    // mp-hitby-contacts: the take lands past the spawn window [M 0x1830e0:36-39 — a body within
    // 4 frames of its spawn stamp is not hit; both bodies stamped at the start]
    ticks(&mut p, 6, &row(None, 2));
    assert_eq!((p.m().objects_rt.uplinks[0].state, p.m().objects_rt.uplinks[0].controller), (0, 0), "team 0 claims it");
    place(&mut p, 1, term, 0.0);
    ticks(&mut p, 120, &row(None, 2));
    assert_eq!(p.m().objects_rt.uplinks[0].state, 0, "a defender on the terminal blocks the flip");
    let away0 = far_from(&p, &elsewhere, 0);
    place(&mut p, 0, away0, 0.0);
    ticks(&mut p, 2, &row(None, 2));
    assert_eq!((p.m().objects_rt.uplinks[0].state, p.m().objects_rt.uplinks[0].controller), (1, 1), "the enemy flips it");
    for _ in 0..600 {
        if p.m().game.state != 0 { break; }
        tick(&mut p, &row(None, 2));
    }
    let g = &p.m().game;
    assert_eq!(g.team_scores[1], 1.0, "team 1 = Σ (int) personals reached the limit");
    assert_eq!(g.team_scores[0], (g.slots[0].score as i32) as f32, "team 0 shows only whole points");
    run_to_end(&mut p, 20);
}

/// Top Agent (0x10): one life each — the shot slot is out (deaths ≥ the lives limit) and cannot
/// respawn [M 0x185368:16-20]; all but one out ends the round [M 0x184268:85-135]; the winner is
/// the unique survivor with the fewest deaths, after the 5 s debrief wait [M 0x1848d0:21-54,
/// 0x184bd0:54-61]. Sister: kills never score in 0x10.
#[test]
fn top_agent_online() {
    let Some(mut p) = start(settings(scenario::DEATH_LIMIT_FFA, 1, -1, [0, 0])) else { eprintln!("skip: NF_ISO not set"); return };
    shoot_dead(&mut p, 0, 1, 2);
    ticks(&mut p, 2, &row(None, 2));
    assert!(p.m().game.state >= 1, "the elimination ends the round");
    assert_eq!(p.m().game.slots[0].score, 0.0, "kills do not score in 0x10");
    run_to_end(&mut p, 400);
    assert_eq!(p.m().game.end_status, MpEndStatus::Won("Alpha".into()));
    assert!(p.a.sim.as_ref().unwrap().mp_dead_ticks[1] > 300, "the out player never respawned");
}

/// Demolition (0x20000040): the attacker (team 1) shoots the site (HP 2000) down — team 1 +1, the
/// last hitter +1, the round over (state 6) [M 0x1870a0:81-165]; 5 s later `MP_RestartScenario`
/// re-arms the site and every player respawns at a spawn point [M 0x184bd0:79-92,
/// 0x183fc0:40-82]; scores carry over. Sister: the defender's team stays 0.
#[test]
fn demolition_online() {
    let Some(mut p) = start(settings(scenario::DEMOLITION, 3, -1, [1, 0])) else { eprintln!("skip: NF_ISO not set"); return };
    let held = strongest_weapon(&p);
    let (centre, _r) = p.m().site_sphere().expect("the live site");
    let away = far_from(&p, &[centre], 0);
    place(&mut p, 1, away, 0.0);
    let (spot, yaw) = firing_spot(&p, centre, centre[1], 4.0);
    place(&mut p, 0, spot, yaw);
    // mp-combat-weapons: the round flies (the record's speed) — a few ticks to the site.
    // mp-shot-gate: the siege outlasts any real clip, so the setup tops the shooter's shared
    // clip on both machines alike (the gate spends it identically; docs/systems/mp-shot-gate.md).
    for sim in [&mut p.a, &mut p.b] {
        let sim = sim.sim.as_mut().unwrap();
        let cs = sim.weapons.base(held as usize) as usize;
        for g in sim.mp_combat.gates.iter_mut() {
            if cs < g.inv.clip.len() { g.inv.clip[cs] = 10000; }
        }
    }
    let r = row_at(&p, 0, held, centre);
    ticks(&mut p, 4, &r);
    let hp = p.m().objects_rt.site.as_ref().unwrap().state;
    assert!(hp < 2000, "the shot damaged the site (HP {hp})");
    assert_eq!(p.m().objects_rt.site.as_ref().unwrap().last_hitter, 0);
    for _ in 0..4000 {
        if p.m().game.state == 6 { break; }
        tick(&mut p, &r);
    }
    assert_eq!(p.m().game.state, 6, "the site fell: the round is over");
    assert_eq!(p.m().game.team_scores, [0.0, 1.0], "the attacking team scores");
    assert_eq!(p.m().game.slots[0].score, 1.0, "the last hitter +1");
    let before = body_pos(&p, 0);
    ticks(&mut p, 302, &row(None, 2));
    let m = p.m();
    assert_eq!(m.game.state, 0, "the next round plays");
    assert_eq!(m.objects_rt.site.as_ref().unwrap().state, 2000, "the site re-armed");
    assert_eq!(m.game.team_scores, [0.0, 1.0], "the scores carry over");
    let now = body_pos(&p, 0);
    assert_ne!(now, before, "the restart respawned the attacker");
    assert!(m.spawns.points.iter().any(|s| s.used != 0 && (s.pos[0] - now[0]).abs() < 1e-3 && (s.pos[2] - now[2]).abs() < 1e-3),
        "at a spawn point");
}

/// Protection (0x20000080): the round times out (1 min) — the DEFENDING team (1) +1 [M
/// 0x1870a0:112-124], state 6 [M 0x184268:57-60], the 5 s restart [M 0x184bd0:79-92]. The
/// attackers' shots dent the site meanwhile (HP agrees on both machines).
#[test]
fn protection_online() {
    let Some(mut p) = start(settings(scenario::PROTECTION, 3, 1, [0, 1])) else { eprintln!("skip: NF_ISO not set"); return };
    let (centre, _r) = p.m().site_sphere().expect("the live site");
    let away = far_from(&p, &[centre], 1);
    place(&mut p, 1, away, 0.0);
    let (spot, yaw) = firing_spot(&p, centre, centre[1], 4.0);
    place(&mut p, 0, spot, yaw);
    ticks(&mut p, 3, &row(Some(0), 2));
    let hp = p.m().objects_rt.site.as_ref().unwrap().state;
    assert!(hp < 2000 && hp > 0, "dented, standing (HP {hp})");
    for _ in 0..3700 {
        if p.m().game.state == 6 { break; }
        tick(&mut p, &row(None, 2));
    }
    assert_eq!(p.m().game.state, 6, "the timeout ends the round");
    ticks(&mut p, 2, &row(None, 2));
    assert_eq!(p.m().game.team_scores, [0.0, 1.0], "the defenders score the timeout");
    ticks(&mut p, 302, &row(None, 2));
    assert_eq!(p.m().game.state, 0, "the next round plays");
    assert_eq!(p.m().objects_rt.site.as_ref().unwrap().state, 2000);
}

/// Industrial Espionage (0x20000100): slot 0 (team 0) takes the blueprint by contact [M
/// 0x1882d8:124-145] and carries it into team 0's base — team +1, carrier +1, the blueprint
/// re-picks a spawn [M 0x188970:27-55]; limit 1 ends. Sister: team 1 stays 0.
#[test]
fn espionage_online() {
    let Some(mut p) = start(settings(scenario::BLUEPRINT, 1, -1, [0, 1])) else { eprintln!("skip: NF_ISO not set"); return };
    let (bp, base0) = (p.m().objects_rt.blueprint.as_ref().unwrap().pos, p.m().objects_rt.esp_bases[0].unwrap());
    let away = far_from(&p, &[bp, base0], 1);
    place(&mut p, 1, away, 0.0);
    place(&mut p, 0, bp, 0.0);
    // mp-hitby-contacts: the take lands past the spawn window [M 0x1830e0:36-39]
    ticks(&mut p, 6, &row(None, 2));
    assert_eq!(p.m().objects_rt.blueprint.as_ref().unwrap().carrier, Some(0), "the contact takes the blueprint");
    place(&mut p, 0, base0, 0.0);
    ticks(&mut p, 2, &row(None, 2));
    assert_eq!(p.m().game.team_scores, [1.0, 0.0]);
    assert_eq!(p.m().game.slots[0].score, 1.0);
    assert_eq!(p.m().objects_rt.blueprint.as_ref().unwrap().state, 0, "free again at a spawn");
    run_to_end(&mut p, 20);
}

/// GoldenEye Strike (0x20000200): slot 0 (team 0) takes both items by contact [M
/// 0x187568:389-400]; one team holding both launches the strike — each carried item +1 to its
/// carrier, a random live enemy targeted [M :356-384]; the strike lands (the port's 3 s flight
/// [S]) and kills the target through the death path, the opposite team +1, both items re-pick
/// [M :96-179]; the target's body dies and respawns 5 s later on both machines.
#[test]
fn goldeneye_online() {
    let Some(mut p) = start(settings(scenario::GOLDENEYE, 2, -1, [0, 1])) else { eprintln!("skip: NF_ISO not set"); return };
    let (i0, i1) = (p.m().objects_rt.ge[0].as_ref().unwrap().pos, p.m().objects_rt.ge[1].as_ref().unwrap().pos);
    let away = far_from(&p, &[i0, i1], 1);
    place(&mut p, 1, away, 0.0);
    place(&mut p, 0, i0, 0.0);
    // mp-hitby-contacts: the takes land past the spawn window [M 0x1830e0:36-39]
    ticks(&mut p, 6, &row(None, 2));
    place(&mut p, 0, i1, 0.0);
    ticks(&mut p, 6, &row(None, 2));
    let o = &p.m().objects_rt;
    assert!(o.ge.iter().all(|g| g.as_ref().unwrap().state == 3), "both carried by team 0 → the strike flies");
    assert_eq!(o.ge_target, Some(1), "the only enemy is the target");
    assert_eq!(p.m().game.slots[0].score, 2.0, "+1 per carried item");
    // the carrier steps away from every item spawn (the re-picked items must stay free)
    let spawns: Vec<[f32; 3]> = p.m().objects.goldeneye.iter().flatten().map(|r| [r.matrix[3][0], r.matrix[3][1], r.matrix[3][2]]).collect();
    let park = far_from(&p, &spawns, 0);
    place(&mut p, 0, park, 0.0);
    let mut flight = 0u32;
    while p.m().game.team_scores[0] == 0.0 {
        tick(&mut p, &row(None, 2));
        flight += 1;
        assert!(flight <= nf_game::mp::rules::GE_STRIKE_FRAMES as u32 + 1, "the strike lands");
    }
    assert!(flight + 2 >= nf_game::mp::rules::GE_STRIKE_FRAMES as u32, "after the flight time ({flight} ticks)");
    let m = p.m();
    assert_eq!(m.game.team_scores, [1.0, 0.0], "the strike scores the opposite team");
    assert_eq!(m.game.slots[1].deaths, 1, "the target died");
    assert!(m.objects_rt.ge.iter().all(|g| g.as_ref().unwrap().state == 0), "both items free again");
    assert!(p.a.sim.as_ref().unwrap().mp_dead_ticks[1] > 0 && p.b.sim.as_ref().unwrap().mp_dead_ticks[1] > 0,
        "the target's body dies on both machines");
    ticks(&mut p, 300, &row(None, 2));
    assert_eq!(p.a.sim.as_ref().unwrap().mp_dead_ticks[1], 0, "and respawns after 5 s");
    assert!(!p.m().game.slots[1].dead);
}

/// Assassination (0x400): `MP_Start` picks a target and an assassin [M 0x183a50:166-167,
/// 0x18aa70]; the assassin killing the target scores +5 [M 0x188f70:229-233]; limit 5 ends with the
/// assassin the winner. Sister: the target's death does not re-pick the roles (only the
/// assassin's does [M :234-243]).
#[test]
fn assassination_online() {
    let Some(mut p) = start(settings(scenario::ASSASSIN, 5, -1, [0, 0])) else { eprintln!("skip: NF_ISO not set"); return };
    let (a, t) = (p.m().assassin.expect("an assassin"), p.m().assassin_target.expect("a target"));
    assert_ne!(a, t);
    assert_eq!(p.m().settings.players[a].team, 1, "the assassin's team word marks it [M 0x18aa70:38-41]");
    shoot_dead(&mut p, a, t, 2);
    assert_eq!(p.m().game.slots[a].score, 5.0, "+5 for the contract");
    assert_eq!((p.m().assassin, p.m().assassin_target), (Some(a), Some(t)), "roles kept");
    run_to_end(&mut p, 20);
    let name = if a == 0 { "Alpha" } else { "Bravo" };
    assert_eq!(p.m().game.end_status, MpEndStatus::Won(name.into()));
}

/// King of the Hill (FFA 0x40000800): the body inside the hill's box drips 0.2/s [M
/// 0x186ae0:78-121, the box test radius 0.0 — disasm 0x186b68/0x186b7c]; limit 1 ends with the
/// king the winner. Sister: the player off the hill scores nothing.
#[test]
fn koth_ffa_online() {
    let Some(mut p) = start(settings(scenario::KOTH_FFA, 1, -1, [0, 0])) else { eprintln!("skip: NF_ISO not set"); return };
    let hill = p.m().objects_rt.hill.as_ref().unwrap().pos;
    let away = far_from(&p, &[hill], 0);
    place(&mut p, 1, away, 0.0);
    place(&mut p, 0, hill, 0.0);
    ticks(&mut p, 2, &row(None, 2));
    assert!(p.m().hill_mask()[0] && !p.m().hill_mask()[1], "slot 0 on the hill, slot 1 off");
    assert_eq!(p.m().game.slots[0].status_bits & 0x10, 0x10, "the on-hill status bit");
    for _ in 0..400 {
        if p.m().game.state != 0 { break; }
        tick(&mut p, &row(None, 2));
    }
    assert!(p.m().game.slots[0].score >= 1.0 && p.m().game.slots[1].score == 0.0);
    run_to_end(&mut p, 20);
    assert_eq!(p.m().game.end_status, MpEndStatus::Won("Alpha".into()));
}

/// Team King of the Hill (0x60001000): the team score is the truncated Σ of its members'
/// personals [M 0x186ae0:122-137]; limit 1 ends. Sister: team 1 stays 0.
#[test]
fn koth_team_online() {
    let Some(mut p) = start(settings(scenario::KOTH_TEAM, 1, -1, [0, 1])) else { eprintln!("skip: NF_ISO not set"); return };
    let hill = p.m().objects_rt.hill.as_ref().unwrap().pos;
    let away = far_from(&p, &[hill], 1);
    place(&mut p, 1, away, 0.0);
    place(&mut p, 0, hill, 0.0);
    for _ in 0..400 {
        if p.m().game.state != 0 { break; }
        tick(&mut p, &row(None, 2));
    }
    assert_eq!(p.m().game.team_scores, [1.0, 0.0], "team 0 = (int) its king's score");
    run_to_end(&mut p, 20);
}

// ------------------------------------------------------------------ mp-hitby-contacts

/// A CTF match with one flag placement built through the real registration path (the live host's
/// 0x26 hook, `register_mp_object_bounds`): facing yaw 90°, the model's local sphere centre
/// (2.0, 0.25, 0.0) off the yaw axis, radius 0.5 — team 1's flag at `pos`. The rotation swings
/// the centre to (0, 0.25, −2.0) [M RotMatrix 0x1e2368's rows + `Control_BuildWorldSph`
/// 0x133070:25-36]; the game's contact law is `Collide_PickObj` 0x1eae80:221-228's STRICT `<`
/// sphere test against it [M].
fn rotated_flag_match() -> nf_game::mp::MpMatch {
    let mut m = nf_game::mp::MpMatch::init(settings(scenario::CAPTURE_THE_FLAG, 5, -1, [0, 1]));
    m.register_mp_object_bounds(
        [10.0, 0.0, 10.0],
        [0.0, std::f32::consts::FRAC_PI_2, 0.0],
        0,
        [0, 1, 0, 0, 0, 0, 0], // flag, team 1
        0x1234,
        Some(nf_game::mp::MpObjBounds {
            centre: [2.0, 0.25, 0.0],
            radius: 0.5,
            min: [-0.5, 0.0, -0.5],
            max: [0.5, 1.0, 0.5],
            sphere_off: [0.0; 3],
        }),
    );
    let mut rng = nf_game::drone::Rng::new(1);
    m.start(&mut rng); // stamps every slot's spawn_tick [S the Player_Init frame alignment]
    m.create_mode_objects(&mut rng);
    m
}

/// The flag's game-law sphere: the world centre offset and the contact reach against the body
/// sphere (`PLAYER_SPHERE_RADIUS` [L ee_start_05 — `M_CONTROL+0x8C` 1.0576]).
fn flag_sphere(m: &nf_game::mp::MpMatch) -> ([f32; 3], [f32; 3], f32) {
    let f = m.objects_rt.flags[1].as_ref().unwrap();
    let b = f.bounds.unwrap();
    let reach = b.radius + nf_game::mp::rules::PLAYER_SPHERE_RADIUS;
    (f.pos, b.sphere_off, reach)
}

/// A slot position whose body-sphere centre (`PLAYER_SPHERE_DY` under the slot position
/// [L ee_start_05]) sits `dist` from `centre` along `dir` (horizontal).
fn body_at(centre: [f32; 3], dir: [f32; 3], dist: f32) -> [f32; 3] {
    let dy = -nf_game::mp::rules::PLAYER_SPHERE_DY;
    [centre[0] + dir[0] * dist, centre[1] + dy, centre[2] + dir[2] * dist]
}

/// The game's contact law through the real registration path: a body at the rotated flag's
/// game-law contact point takes the flag; the same reach from the UNROTATED centre — inside the
/// pre-mp-hitby-contacts stand-in's sphere, outside the game's — never takes. The sister fails if
/// the rotation regresses to the unrotated stand-in.
#[test]
fn a_rotated_flag_base_takes_by_the_game_sphere() {
    let mut m = rotated_flag_match();
    let (pos, off, reach) = flag_sphere(&m);
    // the rotation swings the x-offset to −z [M RotMatrix 0x1e2368; yaw 90° exact lanes]
    assert!(
        off[0].abs() < 1e-4 && (off[1] - 0.25).abs() < 1e-4 && (off[2] + 2.0).abs() < 1e-4,
        "the rotated sphere offset {off:?}"
    );
    let mut rng = nf_game::drone::Rng::new(1);
    let centre = [pos[0] + off[0], pos[1] + off[1], pos[2] + off[2]];
    m.set_slot(0, body_at(centre, [1.0, 0.0, 0.0], reach * 0.99));
    for _ in 0..5 {
        m.update_objects(1.0 / 60.0, &[false; 16], &mut rng);
    }
    let f = m.objects_rt.flags[1].as_ref().unwrap();
    assert_eq!((f.state, f.carrier), (1, Some(0)), "the game-law contact takes the flag");

    // sister: reach·0.95 from the UNROTATED centre (pos + the raw local centre), on the side
    // away from the rotated centre — inside the old stand-in's sphere, outside the game's.
    let mut m2 = rotated_flag_match();
    let (pos2, off2, reach2) = flag_sphere(&m2);
    let old_centre = [pos2[0] + 2.0, pos2[1] + 0.25, pos2[2]];
    let away = [off2[0] - 2.0, 0.0, off2[2]];
    let l = (away[0] * away[0] + away[2] * away[2]).sqrt().max(1e-6);
    let dir = [away[0] / l, 0.0, away[2] / l];
    let mut rng2 = nf_game::drone::Rng::new(1);
    m2.set_slot(0, body_at(old_centre, dir, reach2 * 0.95));
    for _ in 0..5 {
        m2.update_objects(1.0 / 60.0, &[false; 16], &mut rng2);
    }
    let f2 = m2.objects_rt.flags[1].as_ref().unwrap();
    assert_eq!((f2.state, f2.carrier), (0, None), "inside the stand-in sphere but outside the game's: no take");
}

/// The 4-frame spawn window [M MP_HitBy 0x1830e0:36-39 — `|GameState+0x52 − obj+0xEC| < 4`
/// rejects; the stamps are `control_init_object` 0x134630:15 / `MP_ReSpawn` 0x185368:90 /
/// `BOT_respawn` 0x124e38:91]. The same game-law contact point: no take on ticks 1..3 (the diff
/// against the start stamp is still < 4), the take lands on tick 4 — the sister that fails if
/// the window regresses (too wide, too narrow or gone).
#[test]
fn a_body_within_the_spawn_window_is_not_hit() {
    let mut m = rotated_flag_match();
    let (pos, off, reach) = flag_sphere(&m);
    let mut rng = nf_game::drone::Rng::new(1);
    let centre = [pos[0] + off[0], pos[1] + off[1], pos[2] + off[2]];
    m.set_slot(0, body_at(centre, [1.0, 0.0, 0.0], reach * 0.99));
    for t in 1..=3 {
        m.update_objects(1.0 / 60.0, &[false; 16], &mut rng);
        assert_eq!(m.tick, t);
        assert!(m.objects_rt.flags[1].as_ref().unwrap().carrier.is_none(), "tick {t}: inside the spawn window");
    }
    m.update_objects(1.0 / 60.0, &[false; 16], &mut rng);
    assert_eq!(m.tick, 4, "the window's edge");
    assert_eq!(m.objects_rt.flags[1].as_ref().unwrap().carrier, Some(0), "tick 4: the first hittable frame");
}

/// The real arena's rotated flag base — 07000025's flags face arbitrary yaws with off-axis local
/// centres [D the placements: flag local centre (0.351, 0.220, 0.018), yaw 0.9603 → world offset
/// (0.216, 0.220, −0.278)]: the game-law contact point takes the flag; a body inside the
/// unrotated stand-in sphere but outside the game's does not — the sister. Law:
/// `Collide_PickObj` 0x1eae80:221-228 against `Control_BuildWorldSph` 0x133070's centre [M].
#[test]
fn the_real_rotated_flag_base_takes_by_the_game_sphere() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut s = settings(scenario::CAPTURE_THE_FLAG, 5, -1, [0, 1]);
    s.level_code = 0x0700_0025;
    let mut scene = nf_view::load_mp_carry(&path, &format!("{:08x}.bin", s.level_code), false, None, s).expect("loads");
    let m = scene.sim.as_mut().unwrap().mp.as_mut().unwrap();
    let (fpos, off, reach) = {
        let f = m.objects_rt.flags[1].as_ref().unwrap();
        let b = f.bounds.expect("the arena's flag carries bounds");
        (f.pos, b.sphere_off, b.radius + nf_game::mp::rules::PLAYER_SPHERE_RADIUS)
    };
    // the rotation must move the centre for this test to mean anything [D the 07000025 placement]
    let old = {
        let f = m.objects_rt.flags[1].as_ref().unwrap();
        f.bounds.unwrap().centre
    };
    assert!((off[0] - old[0]).abs() > 0.05 || (off[2] - old[2]).abs() > 0.05,
        "the flag's placement is rotated: offset {off:?} vs local {old:?}");
    let mut rng = nf_game::drone::Rng::new(1);
    let centre = [fpos[0] + off[0], fpos[1] + off[1], fpos[2] + off[2]];
    m.set_slot(0, body_at(centre, [1.0, 0.0, 0.0], reach * 0.99));
    for _ in 0..5 {
        m.update_objects(1.0 / 60.0, &[false; 16], &mut rng);
    }
    assert_eq!(m.objects_rt.flags[1].as_ref().unwrap().carrier, Some(0), "the game-law contact takes the real flag");

    // sister: reach·0.95 from the UNROTATED centre, pushed away from the rotated one — no take.
    let mut s2 = settings(scenario::CAPTURE_THE_FLAG, 5, -1, [0, 1]);
    s2.level_code = 0x0700_0025;
    let mut scene2 = nf_view::load_mp_carry(&path, &format!("{:08x}.bin", s2.level_code), false, None, s2).expect("loads");
    let m2 = scene2.sim.as_mut().unwrap().mp.as_mut().unwrap();
    let (fpos2, old2) = {
        let f = m2.objects_rt.flags[1].as_ref().unwrap();
        (f.pos, f.bounds.unwrap().centre)
    };
    let away = [off[0] - old2[0], 0.0, off[2] - old2[2]];
    let l = (away[0] * away[0] + away[2] * away[2]).max(1e-8).sqrt();
    let dir = [away[0] / l, 0.0, away[2] / l];
    let mut rng2 = nf_game::drone::Rng::new(1);
    m2.set_slot(0, body_at([fpos2[0] + old2[0], fpos2[1] + old2[1], fpos2[2] + old2[2]], dir, reach * 0.95));
    for _ in 0..5 {
        m2.update_objects(1.0 / 60.0, &[false; 16], &mut rng2);
    }
    let f2 = m2.objects_rt.flags[1].as_ref().unwrap();
    assert_eq!((f2.state, f2.carrier), (0, None), "inside the stand-in sphere, outside the game's: no take");
}
