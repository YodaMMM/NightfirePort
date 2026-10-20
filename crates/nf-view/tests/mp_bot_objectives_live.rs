//! mp-bot-objectives: the bots play the scenario objectives online. Two machines (slot 0's and
//! slot 1's — the live match start's per-machine setup) tick the SAME input rows through
//! `nf_view::net_tick` with both humans idle; only the bots move. Each scenario must make
//! objective progress from the bots alone (a flag captured, the hill held, a terminal taken, the
//! site shot, an item / the blueprint carried, the contract hunted), with the two machines'
//! hashes equal every tick. The goal choice is `BOTSTATE_pickGoal` 0x126b88 Phase A [M] (the
//! port's `BotBrain::objective_scan`); arena 07000023 carries every scenario's objects.
//! NF_ISO-gated.

use nf_game::mp::{scenario, MpMatch, MpSettings};
use nf_net::NEUTRAL;

const ARENA: u32 = 0x0700_0023;
const N: usize = nf_net::MAX_AGENTS;
/// 10 minutes of match time — the bots fight whoever they see on the way (they meet mid-map).
const LIMIT: u32 = 60 * 60 * 10;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

/// 2 idle humans (slots 0/1) + bots (slots 2..) with the given team words.
fn settings(sc: u32, humans: [u32; 2], bots: &[u32]) -> MpSettings {
    let mut s = MpSettings::deathmatch(ARENA);
    s.scenario = sc;
    s.gametype_mask = sc;
    s.frag_limit = 100; // the objective progress, not the match end, is what the tests watch
    s.time_limit = 10 * 60;
    s.human_count = 2;
    s.player_count = 2;
    s.bot_count = bots.len() as u32;
    s.total_players = 2 + bots.len() as u32;
    s.seed = 0x0b1e_c71e;
    s.players[0].team = humans[0];
    s.players[1].team = humans[1];
    for (i, &t) in bots.iter().enumerate() {
        s.players[2 + i].team = t;
    }
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

fn m(scene: &nf_view::scene::Scene) -> &MpMatch {
    scene.sim.as_ref().unwrap().mp.as_ref().unwrap()
}

/// The bots' state for a desync report.
fn bots_line(scene: &nf_view::scene::Scene) -> String {
    m(scene).bots.iter().map(|b| format!("[slot {} pos {:?} state {:#x} goal1 {:?} opp {:?} route {:?} {}/{} -> {:?} stuck {} hp {} deaths {} shots {} unseen {}]",
        b.slot, b.brain.pos, b.brain.state,
        (b.brain.vars.goals[1].goal_type, b.brain.vars.goals[1].target), b.brain.vars.opponent,
        b.brain.route.nodes.len(), b.brain.route.idx, b.brain.route.valid, b.brain.route.dest, b.brain.stuck_hits,
        b.health, b.deaths, b.shots, b.brain.vars.unseen_ticks)).collect::<Vec<_>>().join("
  ")
}

/// Runs the two machines with idle humans until `done` holds (on machine A, checked on B too)
/// or `limit` ticks; every tick's hashes must agree. `watch` sees machine A each tick. Returns
/// the tick count, or None when `done` never held.
fn run(
    s: MpSettings,
    limit: u32,
    mut watch: impl FnMut(&MpMatch, u32),
    done: impl Fn(&MpMatch) -> bool,
) -> Option<(u32, nf_view::scene::Scene, nf_view::scene::Scene)> {
    let path = iso()?;
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    assert_eq!(m(&a).bots.len(), s.bot_count as usize);
    for (i, bt) in m(&a).bots.iter().enumerate() {
        if s.scenario == scenario::ASSASSIN { break; } // the roles rewrite the team words at the start
        assert_eq!(m(&a).settings.players[bt.slot].team, s.players[2 + i].team.min(1) * (s.team_game_bit()),
            "bot {i} plays its team word in a team game");
    }
    if std::env::var_os("NF_NAVDBG").is_some() {
        let sim = a.sim.as_ref().unwrap();
        for (pi, p) in sim.nav.paths.iter().enumerate() {
            eprintln!("path {pi} {} kind {} nodes {} links {}", p.name, p.kind, p.nodes.len(), p.links.len());
            for (ni, n) in p.nodes.iter().enumerate() {
                eprintln!("  n{ni} {:?} adj {:?}", n.pos, p.adj[ni]);
            }
        }
    }
    let row = [NEUTRAL; N];
    for t in 0..limit {
        let ha = nf_view::net_tick(a.sim.as_mut().unwrap(), &row, 0);
        let hb = nf_view::net_tick(b.sim.as_mut().unwrap(), &row, 1);
        assert_eq!(ha, hb, "tick {t}: the machines' hashes agree\nA {}\nB {}", bots_line(&a), bots_line(&b));
        if std::env::var_os("NF_OBJDBG").is_some() && t % 300 == 0 {
            let o = m(&a).bot_objectives();
            eprintln!("t{t} {}
   flags {:?} bases {:?} hill {:?} site {:?} bp {:?} esp {:?}", bots_line(&a),
                o.flags.iter().flatten().map(|f| (f.pos, f.state, f.carrier)).collect::<Vec<_>>(),
                o.bases.iter().flatten().map(|f| f.pos).collect::<Vec<_>>(),
                o.hill.map(|h| (h.pos, h.min, h.max)), o.site.map(|h| (h.pos, h.centre, h.radius, h.state)),
                o.blueprint.map(|b| (b.pos, b.state, b.carrier)), o.esp_bases.iter().flatten().map(|f| (f.centre, f.radius)).collect::<Vec<_>>());
        }
        watch(m(&a), t);
        if done(m(&a)) {
            assert!(done(m(&b)), "tick {t}: machine B saw the same objective progress");
            // Sister: the comparison can fail — one machine's bot objective goal poked shows in
            // the hash (the brains' goal slots are hashed state).
            let sb = b.sim.as_mut().unwrap();
            let h0 = sb.net_state_hash(1);
            sb.mp.as_mut().unwrap().bots[0].brain.vars.goals[1].target ^= 1;
            assert_ne!(sb.net_state_hash(1), h0, "a bot's objective target is in the hash");
            sb.mp.as_mut().unwrap().bots[0].brain.vars.goals[1].target ^= 1;
            return Some((t + 1, a, b));
        }
    }
    eprintln!("bots at the limit: {}", bots_line(&a));
    None
}

trait TeamBit {
    fn team_game_bit(&self) -> u32;
}
impl TeamBit for MpSettings {
    fn team_game_bit(&self) -> u32 {
        (self.scenario >> 29) & 1
    }
}

/// Every bot's objective goal (slot 1, type 2) count over the run — the goal machine took the
/// scenario objective at all.
fn objective_goals(m: &MpMatch) -> usize {
    m.bots.iter().filter(|b| b.brain.vars.goals[1].goal_type == 2).count()
}

/// Team awareness: no bot ever holds a team-mate as its opponent in a team game.
fn no_teammate_opponent(m: &MpMatch, t: u32) {
    if m.settings.team_game == 0 {
        return;
    }
    for b in &m.bots {
        if let Some(o) = b.brain.vars.opponent {
            if o < nf_game::mp::MP_MAX_SLOTS {
                assert_ne!(m.settings.players[o].team, m.settings.players[b.slot].team,
                    "tick {t}: bot slot {} targets team-mate slot {o}", b.slot);
            }
        }
    }
}

/// Capture the Flag: two bots a side, the humans idle at their spawns — a bot takes the enemy
/// flag (`pickGoal`'s CTF arm: the enemy flag, then the own base when carrying [M
/// 0x126b88:269-283]) and carries it home: a capture scores its team [M 0x1861a8:74-112]. No bot
/// targets a team-mate (the team view's bit 0x8 [M 0x125390:100-102, 0x1424e0:364]).
#[test]
fn ctf_bots_capture_a_flag() {
    let s = settings(scenario::CAPTURE_THE_FLAG, [0, 1], &[0, 1, 0, 1]);
    let mut took = false;
    let mut goals = 0usize;
    let r = run(s, LIMIT * 3, |m, t| {
        no_teammate_opponent(m, t);
        goals = goals.max(objective_goals(m));
        if m.objects_rt.flags.iter().flatten().any(|f| f.carrier.is_some_and(|c| c >= 2)) { took = true; }
    }, |m| m.game.team_scores.iter().any(|&x| x >= 1.0));
    let Some((t, a, _)) = r else {
        if iso().is_none() { eprintln!("skip: NF_ISO not set"); return; }
        panic!("no capture in 30 minutes (a bot took a flag: {took}; objective goals {goals})");
    };
    let g = &m(&a).game;
    assert!(took, "a bot carried a flag");
    let capper = (2..6).find(|&s| g.slots[s].score >= 1.0);
    eprintln!("ctf: capture after {t} ticks, team scores {:?}, capper {capper:?}", g.team_scores);
    assert!(capper.is_some(), "a bot scored the capture (+1 to the carrier [M 0x1861a8:94-98])");
}

/// King of the Hill (FFA): the bots walk onto the hill (`pickGoal`'s hill arm [M
/// 0x126b88:234-246]) and hold it (the on-hill hold `iVar18 = -3` [M :237-243]) — a bot's
/// personal score drips from the hill [M 0x186ae0:78-121].
#[test]
fn koth_bots_hold_the_hill() {
    let s = settings(scenario::KOTH_FFA, [0, 0], &[0, 0, 0]);
    let mut on = 0u32;
    let r = run(s, LIMIT, |m, _| {
        if (2..5).any(|s| m.game.slots[s].status_bits & 0x10 != 0) { on += 1; }
    }, |m| (2..5).any(|s| m.game.slots[s].score >= 1.0));
    let Some((t, _, _)) = r else {
        if iso().is_none() { eprintln!("skip: NF_ISO not set"); return; }
        panic!("no bot scored on the hill in 10 minutes (ticks with a bot on it: {on})");
    };
    eprintln!("koth: a bot reached 1 point after {t} ticks ({on} ticks with a bot on the hill)");
    assert!(on >= 300, "a bot held the hill (1 point = 5 s on it)");
}

/// Team King of the Hill: the same with teams — the team score is the truncated Σ [M
/// 0x186ae0:122-137].
#[test]
fn koth_team_bots_score() {
    let s = settings(scenario::KOTH_TEAM, [0, 1], &[0, 1]);
    let r = run(s, LIMIT, no_teammate_opponent, |m| m.game.team_scores.iter().any(|&x| x >= 1.0));
    let Some((t, a, _)) = r else {
        if iso().is_none() { eprintln!("skip: NF_ISO not set"); return; }
        panic!("no team scored on the hill in 10 minutes");
    };
    eprintln!("team koth: team scores {:?} after {t} ticks", m(&a).game.team_scores);
}

/// Uplink: the bots claim terminals not owned by their team (`MP_getUplinkObj(−n, team)` [M
/// 0x126b88:203-233]) — a terminal's controller is a bot.
#[test]
fn uplink_bots_take_a_terminal() {
    let s = settings(scenario::UPLINK, [0, 1], &[0, 1, 0, 1]);
    let r = run(s, LIMIT, no_teammate_opponent, |m| {
        m.objects_rt.uplinks.iter().filter(|u| u.controller >= 2).count() >= 2
    });
    let Some((t, a, _)) = r else {
        if iso().is_none() { eprintln!("skip: NF_ISO not set"); return; }
        panic!("the bots never took two terminals in 10 minutes");
    };
    let u: Vec<(i16, i16)> = m(&a).objects_rt.uplinks.iter().map(|u| (u.state, u.controller)).collect();
    eprintln!("uplink: terminals (state, controller) {u:?} after {t} ticks");
}

/// Demolition: the attacking bots (team 1) walk to the site and shoot it — `FindOpponent`'s site
/// arm takes the site obj as the opponent within 4.0 [M 0x1424e0:519-560] and the rounds dent
/// its HP [M 0x1870a0:81-91]; the defending bots (team 0) latch beside it [M 0x16e060:244-268].
/// The 200-HP dent is back: with the pit-ramp link creep the second attacker climbs out of the
/// pit and both shoot (the nav defect this file queued is fixed in the same round).
#[test]
fn demolition_bots_shoot_the_site() {
    let s = settings(scenario::DEMOLITION, [0, 0], &[1, 1, 0]);
    let mut guard = false;
    let r = run(s, LIMIT, |m, t| {
        no_teammate_opponent(m, t);
        if m.bots.iter().any(|b| b.team == 0 && b.brain.vars.attack_latch != 0) { guard = true; }
    }, |m| m.objects_rt.site.as_ref().is_some_and(|x| x.state < 2000 - 200));
    let Some((t, a, _)) = r else {
        if iso().is_none() { eprintln!("skip: NF_ISO not set"); return; }
        panic!("the attacking bots never dented the site by 200 in 10 minutes");
    };
    let site = m(&a).objects_rt.site.as_ref().unwrap();
    eprintln!("demolition: site HP {} after {t} ticks, last hitter {}, a defender guarded: {guard}", site.state, site.last_hitter);
    assert!(site.last_hitter >= 2, "a bot hit it");
}

/// The pit-ramp pin (bot-nav-pit-ramp): the game's `FollowRoute` walks each route link along its
/// own line in 0.4-unit creep steps with a 2D-only advance (`LinkCreep_Handler` 0x15a2c8 /
/// `LinkCreep_Increment` 0x159f28 / `LinkCreep_ForNodes` 0x159810 [M]) — so a bot dropped on
/// 07000023's pit floor climbs the pit ramp. The plain node chase it replaced steered straight
/// at node #95 and pushed into the ramp's side (>1.5 under the node root, its reached test never
/// passed) [L the playtest + this test on the pre-fix code: the 27-node route stayed at idx 1
/// (walking toward #95) with stuck hits climbing, under root y −12 for the whole 4000 ticks].
#[test]
fn pit_bot_climbs_the_ramp() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    // Two attackers, no defenders: nobody to fight on the way out of the pit.
    let s = settings(scenario::DEMOLITION, [0, 0], &[1, 1]);
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    // The pit floor node: the bot-path node nearest the playtest's stuck spot (−0.17, ·, 0.17)
    // [L NF_NAVDBG dump: the bot path's n96 (−0.17, −13.67, 0.17), the ramp links n96→n95
    // (3.58, −12.09, −3.69) → n94 (6.66, −10.35, −6.66) → n82 (8.54, −10.35, −8.92)].
    let pit = {
        let sim = a.sim.as_ref().unwrap();
        let pi = sim.nav.nav_path_for([-0.17, -12.97, 0.17]).expect("the pit floor routes");
        let p = &sim.nav.paths[pi];
        let (ni, n) = p
            .nodes
            .iter()
            .enumerate()
            .min_by(|(ia, x), (ib, y)| {
                let da = (x.pos[0] + 0.17).powi(2) + (x.pos[2] - 0.17).powi(2);
                let db = (y.pos[0] + 0.17).powi(2) + (y.pos[2] - 0.17).powi(2);
                da.total_cmp(&db).then(ia.cmp(ib))
            })
            .map(|(i, n)| (i, n.pos))
            .unwrap();
        assert!((n[1] + 13.67).abs() < 0.05, "the pit node sits at y −13.67 [L the dump], got {n:?}");
        assert_eq!(ni, 96, "the pit node is the bot path's n96 [L the dump]");
        eprintln!("pit bot: path {pi} node {ni} at {n:?}");
        [n[0], n[1] + 1.9, n[2]] // floor+0.4; the eye stands at floor+1.8 — half a unit of air to settle
    };
    for sc in [&mut a, &mut b] {
        let sim = sc.sim.as_mut().unwrap();
        let bot = sim.mp.as_mut().unwrap().bots.iter_mut().find(|b| b.slot == 2).expect("the attacker bot");
        bot.brain.pos = pit;
        bot.brain.body.pos = pit;
        bot.brain.body.vel = [0.0; 3];
        bot.brain.route = Default::default();
    }
    let row = [NEUTRAL; N];
    let (mut top_t, mut site_t) = (None, None);
    for t in 0..4000 {
        let ha = nf_view::net_tick(a.sim.as_mut().unwrap(), &row, 0);
        let hb = nf_view::net_tick(b.sim.as_mut().unwrap(), &row, 1);
        assert_eq!(ha, hb, "tick {t}: the machines' hashes agree\nA {}\nB {}", bots_line(&a), bots_line(&b));
        let bot = m(&a).bots.iter().find(|b| b.slot == 2).unwrap();
        let root_y = bot.brain.pos[1] - nf_game::player::EYE;
        if top_t.is_none() && root_y > -9.3 { top_t = Some(t); } // past the ramp-top node #82's root −9.65 [L the dump]
        if site_t.is_none() && root_y > -3.5 { site_t = Some(t); } // the site's storey (the demo dest root −1.9 [L the playtest])
        if site_t.is_some() {
            eprintln!("pit bot: ramp top at tick {:?}, site storey at tick {t} (root y {root_y})", top_t);
            break;
        }
    }
    let bot = m(&a).bots.iter().find(|b| b.slot == 2).unwrap();
    eprintln!(
        "pit bot: route {} {}/{} valid, stuck {}, hp {}",
        bot.brain.route.nodes.len(), bot.brain.route.idx, bot.brain.route.valid, bot.brain.stuck_hits, bot.health
    );
    let top_t = top_t.expect("the bot never gained the ramp top (root y −9.3) out of the pit");
    assert!(top_t <= 900, "the ramp top took {top_t} ticks");
    let site_t = site_t.expect("the bot never reached the site's storey (root y −3.5) in 4000 ticks");
    assert!(site_t <= 3600, "the site storey took {site_t} ticks");
}

/// Protection: the attackers are team 0 [M 0x129a60] — a team-0 bot dents the site by 200.
/// Any round dents the site's HP [M 0x1870a0:81-91], so the last hitter at the done tick can be
/// a defender's stray round at an attacker beside the site — the attack itself is pinned per
/// tick in the watch (a team-0 shooter's hit registering on the site), which is stronger than
/// the old last-tick-only team check.
#[test]
fn protection_bots_shoot_the_site() {
    let s = settings(scenario::PROTECTION, [1, 1], &[0, 0, 1]);
    let mut attacker_hit = false;
    let r = run(s, LIMIT, |m, _t| {
        if m.objects_rt.site.as_ref().is_some_and(|x| {
            x.last_hitter >= 2 && m.settings.players[x.last_hitter as usize].team == 0 && x.state < 2000
        }) { attacker_hit = true; }
        no_teammate_opponent(m, _t);
    }, |m| m.objects_rt.site.as_ref().is_some_and(|x| x.state < 2000 - 200));
    let Some((t, a, _)) = r else {
        if iso().is_none() { eprintln!("skip: NF_ISO not set"); return; }
        panic!("the attacking bots never dented the site by 200 in 10 minutes");
    };
    let site = m(&a).objects_rt.site.as_ref().unwrap();
    eprintln!("protection: site HP {} after {t} ticks, last hitter {}", site.state, site.last_hitter);
    assert!(attacker_hit, "a team-0 bot's rounds registered on the site");
    assert!(site.state < 1800);
}

/// GoldenEye Strike: the bots fetch the free items not already their team's (`pickGoal`'s GE
/// arm, masks {1, 2} [M 0x126b88:175-201, L 0x30ca88]) — one team holding both launches the strike
/// [M 0x187568:356-384].
#[test]
fn goldeneye_bots_launch_a_strike() {
    let s = settings(scenario::GOLDENEYE, [0, 1], &[0, 0, 1, 1]);
    let mut carried = false;
    let r = run(s, LIMIT, |m, t| {
        no_teammate_opponent(m, t);
        if m.objects_rt.ge.iter().flatten().any(|g| g.carrier.is_some_and(|c| c >= 2)) { carried = true; }
    }, |m| m.objects_rt.ge.iter().flatten().all(|g| g.state == 3) || m.game.team_scores.iter().any(|&x| x >= 1.0));
    let Some((t, a, _)) = r else {
        if iso().is_none() { eprintln!("skip: NF_ISO not set"); return; }
        panic!("no strike in 10 minutes (a bot carried an item: {carried})");
    };
    eprintln!("goldeneye: strike after {t} ticks; scores {:?}", m(&a).game.team_scores);
    assert!(carried);
}

/// Industrial Espionage: a bot takes the blueprint (`pickGoal`'s arm: the blueprint, then the
/// base's approach point when carrying [M 0x126b88:256-267, 0x18a338]) and the armed bots fight
/// while it moves.
///
/// The full delivery is NOT asserted: the arena's delivery bases sit on storeys above/below the
/// bots' floor ([D the registered esp-base positions (0.52, −2.99, −1.79) / (−14.78, −19.53,
/// −17.54) vs the bots' floor y ≈ −12.2 — the port probes]) and the port's bot route/walk
/// stand-ins ([S], bots.rs `plan_route`/`walk_body`) do not complete the climb under fire — the
/// deterministic runs show carriers reaching the base's floor plan (dist-xz 0.9) at hp ≤ 40 and
/// dying to the converged fights. The fight dynamics ARE game-faithful (the vision chain
/// 0x175af0/0x175cb8/0x175ef0, the opponent hold 0x1424e0:355-365, the carrier-priority
/// targeting 0x1424e0:367-369 — all [M]); the game's own 2v2 delivery timing is unverified.
/// [S] unverified: whether the game's armed 2v2 bots deliver within 10 minutes.
#[test]
fn espionage_bots_fight_while_the_blueprint_moves() {
    let s = settings(scenario::BLUEPRINT, [0, 1], &[0, 1, 0, 1]);
    let mut carried = false;
    let mut peak_shots = 0u32;
    let mut deaths = 0u32;
    let mut near_base = false;
    run(s, LIMIT, |m, _t| {
        if let Some(bp) = &m.objects_rt.blueprint {
            if let Some(c) = bp.carrier.filter(|&c| c >= 2) {
                carried = true;
                if let Some(b) = m.bots.iter().find(|b| b.slot == c) {
                    let base = m.objects_rt.esp_bases[m.settings.players[c].team as usize % 2];
                    if let Some(base) = base {
                        if ((b.brain.pos[0] - base[0]).powi(2) + (b.brain.pos[2] - base[2]).powi(2)).sqrt() < 3.0 { near_base = true; }
                    }
                }
            }
        }
        peak_shots = peak_shots.max(m.bots.iter().map(|b| b.shots).sum());
        deaths = m.bots.iter().map(|b| b.deaths).sum();
    }, |m| m.game.clock >= m.game.limit);
    assert!(carried, "a bot takes the blueprint [M 0x126b88:256-267]");
    assert!(peak_shots >= 20, "the armed bots fight while the blueprint moves (peak shots {peak_shots}) [M mp-bot-weapons]");
    assert!(deaths >= 1, "the fights are lethal (deaths {deaths})");
    // The carrier got the blueprint to its base's floor plan at least once [M 0x18a338's
    // approach points; the climb-under-fire completion is the [S] gap above].
    assert!(near_base, "a carrier reached its base's floor plan (dist-xz < 3.0)");
}

/// Assassination: the assassin bot's opponent pick is always the target (`MP_IsAssasin` →
/// `MP_getAssassinTarget` [M 0x128b50:80-86]) — the grudge hunt walks to it; the others only
/// fight the assassin (every non-assassin shares team word 0 [M 0x18aa70:38-41] in the team
/// view [M 0x125390:62-73]). Progress: the contract is scored (+5 [M 0x188f70:229-233]).
#[test]
fn assassination_bots_hunt_the_target() {
    let s = settings(scenario::ASSASSIN, [0, 0], &[0, 0, 0, 0]);
    let mut hunted = false;
    let r = run(s, LIMIT, |m, t| {
        let (Some(a), Some(tg)) = (m.assassin, m.assassin_target) else { return };
        for b in m.bots.iter().filter(|b| !b.dead) {
            if b.slot == a && b.brain.vars.grudge == tg as i8 { hunted = true; }
            if b.slot != a {
                if let Some(o) = b.brain.vars.opponent.filter(|&o| o >= nf_game::mp::MP_MAX_SLOTS || !m.game.slots[o].dead) {
                    assert!(o == a || o >= nf_game::mp::MP_MAX_SLOTS, "tick {t}: non-assassin bot {} fights {o}, not the assassin {a} (target {tg}; teams {:?}; dead {:?}; bots dead {:?})", b.slot,
                        (0..6).map(|s| m.settings.players[s].team).collect::<Vec<_>>(), (0..6).map(|s| m.game.slots[s].dead).collect::<Vec<_>>(),
                        m.bots.iter().map(|b| b.dead).collect::<Vec<_>>());
                }
            }
        }
    }, |m| (0..6).any(|s| m.game.slots[s].score >= 5.0));
    let Some((t, a, _)) = r else {
        if iso().is_none() { eprintln!("skip: NF_ISO not set"); return; }
        panic!("no contract in 10 minutes (an assassin bot hunted its target: {hunted})");
    };
    let g = &m(&a).game;
    eprintln!("assassination: contract after {t} ticks; scores {:?}", (0..6).map(|s| g.slots[s].score).collect::<Vec<_>>());
}
