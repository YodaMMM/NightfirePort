//! mp-world-sync (agent 9): the arenas' world interactions online.
//! NF_ISO-gated (the arenas load from the disc).

use nf_game::mp::MpSettings;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

const ARENAS: [u32; 8] = [0x0700_0024, 0x0700_0027, 0x0700_0029, 0x0700_0026, 0x0700_0023, 0x0700_0028, 0x0700_0025, 0x0700_004b];

fn settings(level: u32, humans: u32, bots: u32) -> MpSettings {
    let mut s = MpSettings::deathmatch(level);
    s.human_count = humans;
    s.player_count = humans as u16;
    s.bot_count = bots;
    s.total_players = humans + bots;
    s.seed = 0x0bed_0bad;
    // The Explosive Scenery enviro-mod on: `Destroy_Create` 0x12c318 builds the class-0x18
    // destroyables in MP only while it is set [M :14] (mp-lobby-settings), and this test syncs every
    // object family — 07000025's doors open through its destroyable chain (off: 0 door ticks).
    s.explosive_scenery = 1;
    s.time_limit = 10 * 60;
    s
}

/// The inventory of every interactive object family each arena carries (the census the task's
/// "find every interaction the arenas have" rests on).
#[test]
fn arena_interaction_inventory() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    for level in ARENAS {
        let s = settings(level, 2, 0);
        let scene = nf_view::load_mp_carry(&path, &format!("{level:08x}.bin"), false, None, s).expect("arena loads");
        let sim = scene.sim.as_ref().unwrap();
        let d = &sim.drones;
        let pk: Vec<String> = d.pickups.iter().map(|p| format!("k{}w{:#x}r{}rs{}ch{}", p.kind, p.weapon, p.rounds, p.respawn, p.switch_channel)).collect();
        eprintln!("{level:08x}: pickups {} [{}]", d.pickups.len(), pk.join(" "));
        eprintln!("  doors {} [{}]", d.doors.list.len(), d.doors.list.iter().map(|x| format!("kind{} ch_open{} lock{} appr{} flags{:#x}", x.kind, x.ch_open, x.ch_lock, x.approach, x.flags)).collect::<Vec<_>>().join("; "));
        eprintln!("  triggers {} [{}]", d.triggers.len(), d.triggers.iter().map(|t| format!("kind{} out{} mask{:#x}", t.kind, t.out, t.mask)).collect::<Vec<_>>().join("; "));
        eprintln!("  script players {} drones {} mines {} destroy {} gunimps {} sss {} locks {} fuseboxes {} copters {} grapple {} useobj pcs {} switches {}",
            d.scripts.players.len(), d.list.len(), d.mines.len(), d.props.destroy.len(), d.props.gunimps.len(), d.sss.list.len(), d.locks.len(), d.fuseboxes.len(), d.copters.len(), d.grapple_points.len(),
            d.useobjs.pcs.len(), d.useobjs.switches.len());
        eprintln!("  worldobjs rotors {} inverters {} shooters {} dynobjs {} thirdcams {} music {}", d.worldobjs.rotors.len(), d.worldobjs.inverters.len(), d.worldobjs.shooters.len(), d.worldobjs.dynobjs.len(), d.worldobjs.thirdcams.len(), d.worldobjs.music_triggers.len());
        eprintln!("  ladders {} wires {} creepwalls {} icons {} hints {} hurts {} breaks {}", sim.ladders.len(), sim.wires.len(), sim.creepwalls.len(), sim.icons.len(), sim.hints.len(), sim.hurts.len(), d.breaks.list.len());
        for h in &sim.hurts {
            eprintln!("  hurt #{} kind {} damage {} gate {} who {:#x} one_shot {} tris {} lo {:?} hi {:?}", h.placement, h.kind, h.damage, h.gate, h.who, h.one_shot, h.tris.len(), h.lo, h.hi);
        }
        for x in &d.props.destroy {
            eprintln!("  destroy #{} ch {} hp {} kind {} blast {} lo {:?} hi {:?}", x.placement, x.channel, x.health, x.kind, x.blast_damage, x.lo, x.hi);
        }
    }
}

/// One machine of a two-machine match (the live match-start setup: the private generator, the
/// joiner's `swap_local_slot`).
fn machine(path: &str, s: &MpSettings, slot: usize) -> nf_view::scene::Scene {
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(path, &arena, false, None, s.clone()).expect("arena loads");
    let sim = scene.sim.as_mut().expect("sim");
    sim.net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b ^ (slot as u32).wrapping_mul(0x9e37_79b9)));
    sim.swap_local_slot(slot);
    scene
}

fn body(sim: &nf_view::scene::Sim, slot: usize) -> Option<&nf_game::player::Bond> {
    if slot == sim.mp_local_slot { Some(&sim.bond) } else { sim.mp_bonds[slot].as_ref() }
}

fn wrap(a: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let mut a = a % tau;
    if a > std::f32::consts::PI { a -= tau; }
    if a < -std::f32::consts::PI { a += tau; }
    a
}

fn centre(lo: [f32; 3], hi: [f32; 3]) -> [f32; 3] {
    [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0]
}

/// A per-slot walker that heads for world objects: the nearest live pickup (or, every third
/// pick, a door), turning toward it, pressing E now and then, jumping when stuck and giving up on
/// a target it cannot reach; every 90 ticks it turns to the nearest visible breakable or
/// destroyable and fires once lined up. Every machine feeds the same rows (built from machine
/// A's state — the machines are equal while the hashes are).
struct Walker { target: Option<[f32; 3]>, since: u32, best: f32, skip: Vec<[f32; 3]>, picks: u32, shoot: Option<[f32; 3]>, aim_ticks: u32, course: bool }

impl Walker {
    fn new() -> Walker { Walker { target: None, since: 0, best: f32::MAX, skip: Vec::new(), picks: 0, shoot: None, aim_ticks: 0, course: false } }

    fn input(&mut self, sim: &nf_view::scene::Sim, slot: usize, t: u32) -> nf_net::Input {
        let mut i = nf_net::NEUTRAL;
        i.held = 2;
        let Some(b) = body(sim, slot) else { return i };
        let d = &sim.drones;
        if self.target.is_none() {
            self.picks += 1;
            let mut cands: Vec<[f32; 3]> = Vec::new();
            if self.picks % 3 == 0 {
                cands.extend(d.doors.list.iter().map(|x| x.pos));
            }
            if cands.is_empty() {
                cands.extend(d.pickups.iter().filter(|p| !p.gone && !p.hidden).map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]]));
            }
            let skip = self.skip.clone();
            cands.retain(|p| !skip.iter().any(|s| s == p));
            let dist = |p: &[f32; 3]| (p[0] - b.pos[0]).powi(2) + (p[2] - b.pos[2]).powi(2);
            cands.sort_by(|a, c| dist(a).total_cmp(&dist(c)));
            // Each slot takes a different near candidate so the two walk to different places.
            self.target = cands.get(slot % cands.len().max(1)).copied().or(cands.first().copied());
            self.since = 0;
            self.best = f32::MAX;
        }
        if t % 90 == 0 && self.shoot.is_none() {
            let h = b.head();
            let boxes: Vec<[f32; 3]> = d.props.destroy.iter().filter(|x| !x.dead).map(|x| centre(x.lo, x.hi))
                .chain(d.breaks.list.iter().filter(|x| !x.dead).map(|x| centre(x.lo, x.hi)))
                .filter(|c| ((c[0] - h[0]).powi(2) + (c[2] - h[2]).powi(2)) < 40.0 * 40.0)
                .collect();
            self.shoot = boxes.into_iter().filter(|c| sim.world.segment(h, *c).map_or(true, |t| t > 0.97))
                .min_by(|a, c| { let q = |p: &[f32; 3]| (p[0] - h[0]).powi(2) + (p[2] - h[2]).powi(2); q(a).total_cmp(&q(c)) });
            self.aim_ticks = 0;
        }
        if let Some(c) = self.shoot {
            // Turn to the box and fire once lined up (the combat's own direction convention:
            // dir = (cos p · sin y, sin p, cos p · cos y)).
            let h = b.head();
            let want = (c[0] - h[0]).atan2(c[2] - h[2]);
            let pitch = (c[1] - h[1]).atan2(((c[0] - h[0]).powi(2) + (c[2] - h[2]).powi(2)).sqrt());
            let dy = wrap(want - b.yaw);
            i.turn = dy.clamp(-0.15, 0.15);
            i.look = (pitch - b.pitch).clamp(-0.1, 0.1);
            self.aim_ticks += 1;
            if (dy.abs() < 0.02 && (pitch - b.pitch).abs() < 0.02) || self.aim_ticks > 120 {
                i.shot = true;
                i.trigger = true;
                self.shoot = None;
            }
            return i;
        }
        let Some(tg) = self.target else { return i };
        let (dx, dz) = (tg[0] - b.pos[0], tg[2] - b.pos[2]);
        let dist = (dx * dx + dz * dz).sqrt();
        let want = dx.atan2(dz);
        let dy = wrap(want - b.yaw);
        i.turn = dy.clamp(-0.08, 0.08);
        i.forward = if dy.abs() < 0.8 { 1.0 } else { 0.2 };
        i.look = (-b.pitch).clamp(-0.05, 0.05);
        i.use_key = t % 40 == slot as u32 * 7;
        self.since += 1;
        if dist + 0.05 < self.best { self.best = dist; self.since = self.since.min(60); }
        if self.since % 50 == 49 { i.jump = true; }
        if self.course {
            // A course leg walks THROUGH its target (a pickup / a door) until the next leg.
            if dist < 1.0 { i.turn = 0.0; i.forward = 1.0; }
            return i;
        }
        if dist < 0.6 || self.since > 400 {
            if self.since > 400 { self.skip.push(tg); }
            self.target = None;
        }
        i
    }
}

struct Run { takes: [u32; 2], remote_takes: u32, doors_moved: u32, destroyed: u32, broken: u32, respawned: u32, world_deaths: u32 }

/// Two machines (slots 0 and 1) tick the same rows through `nf_view::net_tick` (the live tick's
/// shape) on `level`; every tick's hash must agree. On the first mismatch the test names the
/// diverging world fields.
/// Places a slot's body on BOTH machines (the same `Bond::stand_at` on each — the machines stay
/// equal): a course leg's start, `R` units from the target on a floor with a clear view of it.
fn place_leg(a: &mut nf_view::scene::Sim, b: &mut nf_view::scene::Sim, slot: usize, target: [f32; 3], k: u32) -> bool {
    const R: f32 = 3.5;
    for j in 0..8 {
        let ang = k as f32 * 1.3 + j as f32 * std::f32::consts::FRAC_PI_4;
        let from = [target[0] + R * ang.sin(), target[1] + 1.0, target[2] + R * ang.cos()];
        let yaw = (target[0] - from[0]).atan2(target[2] - from[2]);
        let nb = nf_game::player::Bond::stand_at(&a.world, from, yaw);
        if (nb.pos[1] - target[1]).abs() > 2.0 { continue; }
        if a.world.segment(nb.head(), [target[0], target[1] + 0.3, target[2]]).is_some_and(|t| t < 0.95) { continue; }
        for sim in [&mut *a, &mut *b] {
            if slot == sim.mp_local_slot { sim.bond = nb.clone(); } else { sim.mp_bonds[slot] = Some(nb.clone()); }
        }
        return true;
    }
    false
}

fn run(level: u32, bots: u32, ticks: u32) -> Option<Run> {
    run_shape(level, bots, ticks, false).map(|r| r.expect("the fixed shape agrees"))
}

/// `old` = the pre-fix tick shape (`old_tick`): returns Err(first mismatching tick) instead of
/// panicking.
fn run_shape(level: u32, bots: u32, ticks: u32, old: bool) -> Option<Result<Run, u32>> {
    let path = iso()?;
    let s = settings(level, 2, bots);
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    let mut walkers = [Walker::new(), Walker::new()];
    let mut out = Run { takes: [0; 2], remote_takes: 0, doors_moved: 0, destroyed: 0, broken: 0, respawned: 0, world_deaths: 0 };
    let mut hidden_before: Vec<bool> = a.sim.as_ref().unwrap().drones.pickups.iter().map(|p| p.hidden).collect();
    let mut was_dead = [false; 2];
    let mut legs = 0u32;
    for t in 0..ticks {
        // Course legs: every 300 ticks each slot starts 3.5 units from a live pickup (every
        // other leg a door, where the arena has doors) and walks through it.
        if t % 300 == 0 && t > 0 {
            for sl in 0..2 {
                legs += 1;
                let sa = a.sim.as_ref().unwrap();
                if sa.mp_dead_ticks[sl] > 0 { continue; }
                let d = &sa.drones;
                let doors: Vec<[f32; 3]> = d.doors.list.iter().map(|x| x.pos).collect();
                let live: Vec<[f32; 3]> = d.pickups.iter().filter(|p| !p.gone && !p.hidden).map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]]).collect();
                let target = if !doors.is_empty() && legs % 2 == 0 { doors[(legs as usize / 2) % doors.len()] }
                    else if !live.is_empty() { live[(legs as usize * 7 + sl * 3) % live.len()] } else { continue };
                let (sa, sb) = (a.sim.as_mut().unwrap(), b.sim.as_mut().unwrap());
                if place_leg(sa, sb, sl, target, legs) {
                    walkers[sl].target = Some(target);
                    walkers[sl].course = true;
                    walkers[sl].since = 0;
                }
            }
        }
        let row: [nf_net::Input; nf_net::MAX_AGENTS] = {
            let sa = a.sim.as_ref().unwrap();
            let mut r = [nf_net::NEUTRAL; nf_net::MAX_AGENTS];
            for sl in 0..2 { r[sl] = walkers[sl].input(sa, sl, t); }
            r
        };
        let (ha, hb) = if old {
            (old_tick(a.sim.as_mut().unwrap(), &row, 0), old_tick(b.sim.as_mut().unwrap(), &row, 1))
        } else {
            (nf_view::net_tick(a.sim.as_mut().unwrap(), &row, 0), nf_view::net_tick(b.sim.as_mut().unwrap(), &row, 1))
        };
        let (sa, sb) = (a.sim.as_ref().unwrap(), b.sim.as_ref().unwrap());
        if ha != hb {
            let mut fa = sa.net_world_fields();
            let mut fb = sb.net_world_fields();
            for sl in 0..2 {
                fa.push((format!("body{sl}"), format!("{:?}", body(sa, sl).map(|x| (x.pos, x.yaw, x.substate)))));
                fb.push((format!("body{sl}"), format!("{:?}", body(sb, sl).map(|x| (x.pos, x.yaw, x.substate)))));
                fa.push((format!("hp{sl}"), format!("{} {}", sa.mp_hp[sl], sa.mp_dead_ticks[sl])));
                fb.push((format!("hp{sl}"), format!("{} {}", sb.mp_hp[sl], sb.mp_dead_ticks[sl])));
            }
            fa.push(("rng".into(), format!("{:x}/{:x}/{}", sa.drones.rng.x, sa.drones.rng.y, sa.drones.rng.draws)));
            fb.push(("rng".into(), format!("{:x}/{:x}/{}", sb.drones.rng.x, sb.drones.rng.y, sb.drones.rng.draws)));
            let diff: Vec<String> = fa.iter().zip(fb.iter()).filter(|(x, y)| x.1 != y.1)
                .map(|(x, y)| format!("{}: A {} | B {}", x.0, x.1, y.1)).collect();
            if old {
                eprintln!("{level:08x} (pre-fix shape) tick {t}: diverging:\n{}", diff.join("\n"));
                return Some(Err(t));
            }
            panic!("{level:08x} tick {t}: hash {ha:x} vs {hb:x}; diverging:\n{}", diff.join("\n"));
        }
        for &(_, slot, bot) in &sa.net_last_takes {
            if !bot && slot < 2 { out.takes[slot] += 1; if slot == 1 { out.remote_takes += 1; } }
        }
        out.doors_moved += sa.drones.doors.list.iter().filter(|d| d.pos01 > 0.0).count() as u32;
        for (k, p) in sa.drones.pickups.iter().enumerate() {
            if hidden_before.get(k).copied().unwrap_or(false) && !p.hidden && !p.gone { out.respawned += 1; }
        }
        hidden_before = sa.drones.pickups.iter().map(|p| p.hidden).collect();
        for sl in 0..2 {
            let dd = sa.mp_dead_ticks[sl] > 0;
            if dd && !was_dead[sl] { out.world_deaths += 1; }
            was_dead[sl] = dd;
        }
    }
    let sa = a.sim.as_ref().unwrap();
    out.destroyed = sa.drones.props.destroy.iter().filter(|x| x.dead).count() as u32;
    out.broken = sa.drones.breaks.list.iter().filter(|x| x.dead).count() as u32;
    eprintln!("{level:08x} + {bots} bots, {ticks} ticks agree: takes {:?} (slot 1's on machine 0: {}), door-open ticks {}, destroyed {}, broken {}, respawns {}, deaths {}",
        out.takes, out.remote_takes, out.doors_moved, out.destroyed, out.broken, out.respawned, out.world_deaths);
    // Sister: the world fold can fail — one machine's pickup taken alone diverges the hash.
    let sb = b.sim.as_mut().unwrap();
    let h0 = sb.net_state_hash(1);
    if let Some(p) = sb.drones.pickups.iter_mut().find(|p| !p.gone) { p.gone = true; }
    assert_ne!(sb.net_state_hash(1), h0, "a pickup's state is in the hash");
    Some(Ok(out))
}

#[test]
fn atlantis_walkers_take_pickups_hashes_agree() {
    let Some(r) = run(0x0700_0023, 0, 5400) else { eprintln!("skip: NF_ISO not set"); return };
    assert!(r.takes[0] > 0 && r.takes[1] > 0, "both walkers take pickups: {:?}", r.takes);
    assert!(r.remote_takes > 0, "machine 0 saw slot 1's takes (the remote body takes)");
}

#[test]
fn sub_pen_doors_triggers_hashes_agree() {
    let Some(r) = run(0x0700_0025, 0, 5400) else { eprintln!("skip: NF_ISO not set"); return };
    assert!(r.takes[0] + r.takes[1] > 0, "pickups taken: {:?}", r.takes);
    assert!(r.doors_moved > 0, "a body opened a door (the trigger plates see every body)");
}

#[test]
fn skyrail_with_bots_hashes_agree() {
    let Some(r) = run(0x0700_0024, 2, 3600) else { eprintln!("skip: NF_ISO not set"); return };
    assert!(r.takes[0] + r.takes[1] > 0, "pickups taken: {:?}", r.takes);
}

#[test]
fn fort_knox_and_phoenix_breakables_hashes_agree() {
    let Some(r) = run(0x0700_0027, 0, 3600) else { eprintln!("skip: NF_ISO not set"); return };
    assert!(r.takes[0] + r.takes[1] > 0, "pickups taken: {:?}", r.takes);
    let Some(r2) = run(0x0700_0026, 0, 3600) else { return };
    assert!(r2.takes[0] + r2.takes[1] > 0, "pickups taken: {:?}", r2.takes);
    assert!(r.destroyed + r.broken + r2.destroyed + r2.broken > 0, "a shot broke a breakable/destroyable");
}

#[test]
fn ravine_and_snow_blind_hashes_agree() {
    let Some(r) = run(0x0700_004b, 0, 3600) else { eprintln!("skip: NF_ISO not set"); return };
    assert!(r.takes[0] + r.takes[1] > 0, "pickups taken: {:?}", r.takes);
    run(0x0700_0029, 0, 2400);
}

/// The pre-fix tick shape (the local `step_bond` without the row, then the remotes, then the
/// combat) — the sister of every agreeing run above: with the world folded into the hash, a
/// remote body's take on one machine only is now DETECTED.
fn old_tick(sim: &mut nf_view::scene::Sim, row: &[nf_net::Input; nf_net::MAX_AGENTS], local: usize) -> u64 {
    let li = row[local];
    sim.step_bond(nf_view::mpnet::to_player_input(&li), li.trigger, li.reload, li.use_key);
    let inputs = std::array::from_fn(|i| nf_view::mpnet::to_player_input(&row[i]));
    sim.step_net_remotes(&inputs, local);
    sim.step_net_combat(row, local);
    sim.net_state_hash(local)
}

#[test]
fn the_pre_fix_tick_shape_diverges_on_world_interactions() {
    // The same course on Atlantis through the old shape: the first remote take / shot object
    // splits the machines — and the fold now sees it.
    let Some(r) = run_shape(0x0700_0023, 0, 3000, true) else { eprintln!("skip: NF_ISO not set"); return };
    match r {
        Err(t) => eprintln!("pre-fix shape: first mismatch at tick {t}"),
        Ok(_) => panic!("the old shape never diverged — the world fold must detect a one-machine take"),
    }
}

/// Phoenix Base's pit kill floors (#25/#26/#122/#123: type 6, 1000 damage, who 0 [D the
/// inventory above]): slot 1 dropped over the pit dies on BOTH machines at the same tick (the
/// old path applied a hurt only to the local health block, which the online combat overwrote
/// every tick — nobody died there online).
#[test]
fn phoenix_kill_floor_kills_the_remote_body_on_both_machines() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let s = settings(0x0700_0026, 2, 0);
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    let hurt = a.sim.as_ref().unwrap().hurts.iter().find(|h| h.damage >= 1000.0 && h.who == 0).map(|h| (h.lo, h.hi)).expect("a kill floor");
    let mid = [(hurt.0[0] + hurt.1[0]) / 2.0, hurt.1[1] + 3.0, (hurt.0[2] + hurt.1[2]) / 2.0];
    for sim in [a.sim.as_mut().unwrap(), b.sim.as_mut().unwrap()] {
        let mut nb = sim.mp_bonds[1].clone().unwrap_or_else(|| sim.bond.clone());
        nb.pos = mid;
        nb.prev_pos = mid;
        nb.vel = [0.0; 3];
        if sim.mp_local_slot == 1 { sim.bond = nb; } else { sim.mp_bonds[1] = Some(nb); }
    }
    let mut died = [None, None];
    for t in 0..300u32 {
        let row = [nf_net::NEUTRAL; nf_net::MAX_AGENTS];
        let ha = nf_view::net_tick(a.sim.as_mut().unwrap(), &row, 0);
        let hb = nf_view::net_tick(b.sim.as_mut().unwrap(), &row, 1);
        assert_eq!(ha, hb, "tick {t}");
        for (k, sc) in [&a, &b].iter().enumerate() {
            if died[k].is_none() && sc.sim.as_ref().unwrap().mp_dead_ticks[1] > 0 { died[k] = Some(t); }
        }
    }
    eprintln!("phoenix pit: slot 1 died at {died:?} (machine 0, machine 1)");
    assert!(died[0].is_some() && died[0] == died[1], "both machines kill slot 1 at the same tick: {died:?}");
    assert_eq!(a.sim.as_ref().unwrap().mp.as_ref().unwrap().game.slots[1].deaths, 1, "the death scores through MP_PlayerKilled");
}
