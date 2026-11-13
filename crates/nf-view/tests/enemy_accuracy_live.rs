//! Enemy accuracy and damage against Bond, measured through the real path (`nf_view::load_with_carry_difficulty`
//! + `Sim::step_bond`): Bond stands still in the open, one hostile drone is left active at a fixed distance,
//! and every shot it fires is logged with its accuracy roll (`Drone::shot_rolls`) and every hit with its damage
//! (`Drones::player_hits`). Bond's health is restored after each tick so the run never ends early.
//!
//! The chain measured (all [M] in work/decomp/ACTION.ELF):
//! - `DroneWeap_DoBulletAccuracy` 0x179740: p = 100 − 5·`+0xB4`, × TooClose_Accuracy under TooClose_Distance,
//!   × TargetFirstMoved / FirstStopped (0C/0D, anim set 0x13) / Moving, × DroneFiring_Accuracy_{Easy,Normal,Hard}
//!   by `GameState+0x28` (1 / 0,2,≥5 / 3,4), × TooClose again; hit = FRand(100) < p, gated beyond TooClose on
//!   `+0x270 ≥ NewSighting_TimeToHit·60` and `+0x41 == 0`; dist < 3 always hits; a miss aims at the target plus
//!   the (0.5 sin a, 1.5 cos a, 1.5 sin 2a) wobble.
//! - `DroneWeap_FireWeapon` 0x178f68: `weapon_data+0x18` rounds per shot through `Bullet_init`.
//! - `Drone_ModBulletDamage` 0x139f50: `weapon_data+0x0C` × TooClose_Damage × `+0x100` × the back-shot factor.
//! - `Player_HandlePain` 0x1902c8: × `Plr_DMod_*` by difficulty, armour first, health below 1 → 0.
//!
//! Opt-in via NF_ISO.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// One measurement: shots fired, rolled hits, landed hits and the damage of each landed hit over `ticks`
/// ticks after the first shot.
#[derive(Debug, Default, Clone)]
pub struct Tally {
    pub shots: usize,
    pub rolled: usize,
    pub hits: usize,
    pub damage: Vec<f32>,
    pub start_dist: f32,
    pub end_dist: f32,
    pub weapon: u32,
    pub accuracy: u8,
    pub aggression: u8,
    /// The frames of every counted shot, relative to the first.
    pub shot_frames: Vec<u32>,
    /// The window's total accuracy rolls (rolled + missed): every landed hit corresponds to one
    /// roll — a rolled miss still flies the wobble-offset bullet and can clip Bond's bounds
    /// [M 0x179740's miss arm; the port records exactly that in `shot_rolls`' doc] — so `hits`
    /// may exceed `rolled`, never `rolls`.
    pub rolls: usize,
}

impl Tally {
    fn per_hit(&self) -> f32 { if self.hits == 0 { 0.0 } else { self.damage.iter().sum::<f32>() / self.hits as f32 } }
}

/// Loads `archive` at `difficulty`, plays the intro out, keeps only the drone `pick` chooses, parks Bond `dist`
/// units from it in the open (a clear line of sight, level floor), and counts for `ticks` after its first shot.
pub fn measure(archive: &str, difficulty: u32, dist: f32, ticks: u32, skip: usize, pick: &dyn Fn(&nf_game::drone::Drone) -> bool) -> Option<Tally> {
    let iso = iso()?;
    let mut scene = nf_view::load_with_carry_difficulty(&iso, archive, false, None, difficulty).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..4000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    if std::env::var_os("NF_ACC_LIST").is_some() {
        for (i, d) in sim.drones.list.iter().enumerate() {
            eprintln!("[acc] drone {i} present {} enabled {} team {} weapon {:#x} skin {:#x} acc {} pos {:?}", d.present, d.enabled, d.team, d.weapon, d.skin, d.accuracy, d.pos);
        }
    }
    let cands: Vec<usize> = sim.drones.list.iter().enumerate().filter(|(_, d)| !d.corpse && pick(d)).map(|(i, _)| i).skip(skip).collect();
    if cands.is_empty() { eprintln!("[acc] no drone picked"); return None; }
    // Zero-G floaters (the 0700001b astronauts) are moved to Bond instead: a spot `dist` from Bond's
    // spawn at his chest height with a clear line. Ground drones get Bond parked `dist` from them on their floor.
    let float = sim.drones.list[cands[0]].weapon == 0x6a;
    let mut found = None;
    'c: for &g in &cands {
        let gp = sim.drones.list[g].pos;
        for k in 0..64 {
            let a = k as f32 * std::f32::consts::TAU / 64.0;
            if float {
                let b = sim.bond.pos;
                let q = [b[0] + dist * a.sin(), b[1] + 0.25, b[2] + dist * a.cos()];
                if sim.world.segment([b[0], b[1] + 0.7, b[2]], q).is_some() { continue; }
                found = Some((g, b, q));
                break 'c;
            }
            let p = [gp[0] + dist * a.sin(), gp[1], gp[2] + dist * a.cos()];
            let Some(f) = sim.world.floor_below([p[0], p[1] + 0.5, p[2]], 3.0) else { continue };
            if (f - (gp[1] - 1.0)).abs() > 0.3 { continue; }
            // A clear corridor: rays from the drone's body (centre ± 0.5 sideways, hip to head) to
            // Bond's whole body (feet+0.3 to head) — a corner the drone hugs would eat the rounds.
            let side = [a.cos(), 0.0, -a.sin()];
            let clear = [-0.5f32, 0.0, 0.5].iter().all(|&o| [0.0f32, 0.4, 0.8].iter().all(|&h| [0.3f32, 1.0, 1.8].iter().all(|&bh| {
                let from = [gp[0] + side[0] * o, gp[1] + h, gp[2] + side[2] * o];
                sim.world.segment(from, [p[0] + side[0] * o * 0.5, f + bh, p[2] + side[2] * o * 0.5]).is_none()
            })));
            if !clear { continue; }
            // Open ground: no wall within 1.5 of the spot at waist height in 8 directions.
            let open = (0..8).all(|j| { let b = j as f32 * std::f32::consts::FRAC_PI_4;
                sim.world.segment([p[0], f + 1.0, p[2]], [p[0] + 1.5 * b.sin(), f + 1.0, p[2] + 1.5 * b.cos()]).is_none() });
            if !open { continue; }
            found = Some((g, [p[0], f + 1.1, p[2]], gp));
            break 'c;
        }
    }
    let Some((g, spot, gp)) = found else { eprintln!("[acc] no open spot at {dist}"); return None };
    for (i, d) in sim.drones.list.iter_mut().enumerate() { if i != g { d.present = false; d.enabled = false; } else { d.enabled = true; d.present = true; } }
    let face = (gp[0] - spot[0]).atan2(gp[2] - spot[2]);
    if !float { sim.bond = nf_game::player::Bond::stand_at(&sim.world, spot, face); } else { sim.bond.yaw = face; }
    {
        let ch = sim.drones.list[g].switch_channel;
        let f = sim.drones.frame;
        if ch != 0 { sim.drones.channels.set(ch as u32, 1, f); }
        let dr = &mut sim.drones.list[g];
        if float { dr.pos = gp; }
        let yaw = (spot[0] - gp[0]).atan2(spot[2] - gp[2]);
        dr.yaw = yaw; dr.desired_yaw = yaw;
    }
    // The harness measures the fire chain from a FIXED stream state: everything before this line
    // (the level intro, the settle, the other drones' fire — the glass-passed plasma rounds'
    // 14400-tick lives and their DoTrails draws especially [M Bullet_init 0x12c5a0:199-211 the
    // 0x3b fuse; Bullet_DoTrails 0x12d890 — Rand_Rand(5) within 3 u of the viewer; Collide_Update
    // 0x1ea398:216-229 the glass pass the rounds ride]) shifts the shared RNG stream by an amount
    // that has nothing to do with the chain being measured, and the sight-acquisition rolls the
    // window's first shot rides used to depend on that stream position — guard-batch-repin
    // bisected the astronaut red to exactly that (reverting only the fire-path glass pass turned
    // the test green). Reset to a fixed seed here and drop the settle's stray rounds, so every
    // case starts its window from the same stream state regardless of what flew before it.
    sim.drones.rng = nf_game::drone::Rng::new(0xacd_1b6a);
    sim.drones.bullets.clear();
    let mut t = Tally { start_dist: dist, weapon: sim.drones.list[g].weapon, accuracy: sim.drones.list[g].accuracy, ..Default::default() };
    let mut first: Option<u32> = None;
    let mut shots0 = 0;
    let mut rolls0 = 0;
    for tick in 0..(ticks + 1800) {
        sim.drones.player_hits.clear();
        sim.drones.health.health = 100.0;
        sim.drones.health.armour = 0.0;
        sim.step_bond(Input::default(), false, false, false);
        let dr = &sim.drones.list[g];
        if std::env::var_os("NF_ACC_TRACE").is_some() && tick % 30 == 0 {
            eprintln!("[acc] t{tick} st {:#x} cls {} ast {:#x} firing {} burst {} next {} frame {} clip {} yaw {:.2}+{:.2} opp {:.2} unseen {} recoil {} ev21 {} los {} seen {}",
                dr.state, dr.anim.class, dr.anim.state, dr.firing, dr.burst, dr.next_shot, sim.drones.frame, dr.clip, dr.yaw, dr.aim_yaw, dr.yaw_to_opp, dr.unseen_ticks, dr.recoil_weapon, dr.event_21, dr.los, dr.seen);
        }
        if first.is_none() && !dr.shots.is_empty() { first = Some(tick); shots0 = dr.shots.len() - 1; rolls0 = dr.shot_rolls.len().saturating_sub(1); }
        if let Some(f) = first {
            if tick >= f + ticks { break; }
            if std::env::var_os("NF_ACC_SHOTS").is_some() {
                let dr = &sim.drones.list[g];
                if dr.shots.last() == Some(&sim.drones.frame.wrapping_sub(0)) || dr.shot_aims.last().is_some_and(|a| a.0 + 1 >= sim.drones.frame) {
                    if let Some(a) = dr.shot_aims.last() { if a.0 + 1 == sim.drones.frame || a.0 == sim.drones.frame { eprintln!("[acc] shot f{} muzzle {:?} target {:?} bond {:?} roll {:?}", a.0, a.1, a.2, sim.bond.pos, dr.shot_rolls.last()); } }
                }
                for e in &sim.drones.impacts { eprintln!("[acc]   impact {e:?}"); }
            }
            for &(_, owner, dmg) in &sim.drones.player_hits { if owner == g && dmg > 0.0 { t.hits += 1; t.damage.push(dmg); } }
        }
    }
    let dr = &sim.drones.list[g];
    if first.is_none() { eprintln!("[acc] drone {g} never fired (dist now {:.1}, state {:#x}, seen {} los {} vis {:.2} alert {:.2})", dr.dist, dr.state, dr.seen, dr.los, dr.visibility, dr.alert); return None; }
    t.shots = dr.shots.len() - shots0;
    t.aggression = dr.aggression;
    t.shot_frames = dr.shots[shots0..].iter().map(|&f| f - dr.shots[shots0]).collect();
    t.rolls = dr.shot_rolls[rolls0..].len();
    t.rolled = dr.shot_rolls[rolls0..].iter().filter(|&&r| r).count();
    t.end_dist = dr.dist;
    Some(t)
}

fn report(label: &str, t: &Tally) {
    eprintln!("[acc] {label}: weapon {:#x} acc {} dist {:.1}->{:.1}: shots {} rolled {} hits {} per-hit {:.2} total {:.1} /10s",
        t.weapon, t.accuracy, t.start_dist, t.end_dist, t.shots, t.rolled, t.hits, t.per_hit(), t.damage.iter().sum::<f32>());
    if std::env::var_os("NF_ACC_FRAMES").is_some() { eprintln!("[acc]   aggression {} shot frames {:?}", t.aggression, t.shot_frames); }
}

/// Exploration: `NF_ACC_LEVEL` (default 07000002.bin), every difficulty × 5/10/20 u, printed.
#[test]
#[ignore = "exploration printout; the pinned matrix is below"]
fn print_matrix() {
    if iso().is_none() { return; }
    let level = std::env::var("NF_ACC_LEVEL").unwrap_or_else(|_| "07000002.bin".into());
    for diff in [1, 2, 3] {
        for dist in [5.0, 10.0, 20.0] {
            let want: Option<u32> = std::env::var("NF_ACC_WEAPON").ok().and_then(|w| u32::from_str_radix(w.trim_start_matches("0x"), 16).ok());
            let skip0: usize = std::env::var("NF_ACC_SKIP").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
            let p = |d: &nf_game::drone::Drone| d.team == 1 && (d.weapon > 1 && d.weapon < 0x47 || (0x6a..=0x71).contains(&d.weapon)) && want.is_none_or(|w| d.weapon == w);
            match (skip0..skip0 + 6).find_map(|k| measure(&level, diff, dist, 600, k, &p)) {
                Some(t) => report(&format!("{level} diff {diff} d {dist}"), &t),
                None => eprintln!("[acc] {level} diff {diff} d {dist}: no measurement"),
            }
        }
    }
}

#[test]
#[ignore = "exploration printout"]
fn print_class4_danims() {
    let Some(iso) = iso() else { return };
    let scene = nf_view::load(&iso, "07000002.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    for (i, info) in sim.tables.info.iter().enumerate() {
        if info[0xc] == 4 { eprintln!("[acc] danim {i:#x} state {:#x} class 4", u16::from_le_bytes([info[8], info[9]])); }
    }
}

#[test]
#[ignore = "exploration printout"]
fn print_guard_box_envelope() {
    let Some(iso) = iso() else { return };
    let mut scene = nf_view::load(&iso, "07000002.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
    let d = sim.data();
    for (i, dr) in sim.drones.list.iter().enumerate().take(8) {
        let floor = sim.world.floor_below([dr.pos[0], dr.pos[1] + 0.2, dr.pos[2]], 3.0);
        let bx = dr.hit_boxes(&d);
        for b in &bx {
            let c = b.m[3];
            eprintln!("[acc] drone {i} part {} centre-rel ({:.2},{:.2},{:.2}) half {:?}", b.part, c[0] - dr.pos[0], c[1] - dr.pos[1], c[2] - dr.pos[2], b.half);
        }
        eprintln!("[acc] drone {i} pos {:?} floor {:?} danim {:#x}", dr.pos, floor, dr.anim.danim);
    }
}

#[test]
#[ignore = "exploration printout"]
fn print_weapon_fire_fields() {
    let Some(iso) = iso() else { return };
    let scene = nf_view::load(&iso, "07000002.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    let d = sim.data();
    for w in [0x0eusize, 0x12, 0x1c, 0x6a] {
        let r = &d.weapons.records[w];
        let f = |o: usize| f32::from_le_bytes(r[o..o + 4].try_into().unwrap());
        let h = |o: usize| u16::from_le_bytes([r[o], r[o + 1]]);
        eprintln!("[acc] weapon {w:#x}: dmg {} rounds {} speed {} range {} spread {} burst {} interval {}", f(0x0c), r[0x18], f(0x20), f(0x1c), f(0x24), h(0x2c), h(0x40) as i16);
    }
}

// ---- pinned ------------------------------------------------------------------------------------------------

/// The level tuning as `ReadTuningVars` 0x1d3a80 leaves it.
fn tuning(level: u32) -> nf_game::tuning::Tuning {
    let mut t = nf_game::tuning::Tuning::default();
    nf_game::tuning::read_tuning_vars(&mut t, level);
    t
}

/// `Player_HandlePain`'s difficulty factor (`Plr_DMod_*`) [M 0x1902c8:30-49].
fn dmod(t: &nf_game::tuning::Tuning, diff: u32) -> f32 {
    match diff { 1 => t.Plr_DMod_Easy, 3 => t.Plr_DMod_Hard, 4 => 2.0, _ => t.Plr_DMod_Normal }
}

fn measured(level: &str, diff: u32, dist: f32, weapon: u32) -> Option<Tally> {
    let p = |d: &nf_game::drone::Drone| d.team == 1 && d.weapon == weapon;
    (0..6).find_map(|k| measure(level, diff, dist, 600, k, &p))
}

/// The rolled hits of a run against `DroneWeap_DoBulletAccuracy`'s percentage: within three binomial
/// sigmas (+1) of `shots · p/100`.
fn rolls_match(t: &Tally, level: u32, diff: u32) -> bool {
    let p = (nf_game::drone::hit_chance(&tuning(level), level, diff, t.accuracy, t.start_dist.min(t.end_dist).max(3.0),
        nf_game::drone::Motion::default(), 0) / 100.0).clamp(0.0, 1.0);
    let n = t.shots as f32;
    let sigma = (n * p * (1.0 - p)).sqrt();
    ((t.rolled as f32) - n * p).abs() <= 3.0 * sigma + 1.0
}

/// 07000002's shotgun guard (weapon 0x1C: 8 rounds of 3.0, spread 32 [D weapon_data 0x2bf150 +0x18/+0x0C/+0x24]):
/// every shot sends 8 pellets [M DroneWeap_FireWeapon 0x178f68:233-241], so the landed pellets far outnumber
/// the rolled hits; each pellet costs 3.0 × Plr_DMod (0.5 / 0.7 / 1.2 on 01–04 [M 0x1d3a80]).
/// Sister: the old single-pellet fire landed at most one hit per shot (hits ≤ shots), which fails here.
#[test]
fn shotgun_guards_fire_eight_pellets() {
    if iso().is_none() { eprintln!("NF_ISO not set — skipping"); return; }
    let t02 = tuning(0x0700_0002);
    for diff in [1, 2, 3] {
        let t = measured("07000002.bin", diff, 5.0, 0x1c).expect("the 0x1C guard fires at Bond 5 u away");
        report(&format!("02 shotgun diff {diff} d 5"), &t);
        assert!(t.hits > t.shots, "diff {diff}: {} pellets landed from {} shots — the 8-pellet loop is missing", t.hits, t.shots);
        assert!(t.hits >= 4 * t.rolled.max(1), "diff {diff}: {} hits from {} rolled hits", t.hits, t.rolled);
        let want = 3.0 * dmod(&t02, diff);
        assert!(t.damage.iter().all(|&d| (d - want).abs() < 1e-4), "diff {diff}: every pellet costs {want}, got {:?}", t.damage);
        assert!(rolls_match(&t, 0x0700_0002, diff), "diff {diff}: {} rolled of {} shots vs the decomp's chance", t.rolled, t.shots);
    }
}

/// Single-round guards on 07000002 at 5 / 20 u and each difficulty: the rolled hits follow
/// `DoBulletAccuracy`'s chance, the rolled hits land (a corridor in the open), and no round
/// beyond the chain's rolls lands (a rolled miss still flies the wobble-offset bullet and can
/// clip Bond's bounds [M 0x179740's miss arm — landed hits may exceed rolled hits, never the
/// roll count]), and every hit costs `weapon_data+0x0C × Drone+0x100 × Plr_DMod`. Sister: each
/// bound is two-sided (a hit without a round behind it breaks `hits ≤ rolls`, hits lost to the
/// geometry break `hits + 4 ≥ rolled`), and the chance check fails for any other difficulty
/// factor (`rolls_match` with Easy vs Hard differs by ×2.4 on 01–04). The landed bound's ±4
/// slack covers the acquisition gate: a rolled-true shot beyond TooClose still clears to a miss
/// while `seen_ticks < NewSighting_TimeToHit·60` [M 0x179740:93-99 — the roll is consumed
/// either way], and the window opens on exactly those early shots.
#[test]
fn rifle_guards_hit_as_often_and_as_hard_as_the_chain() {
    if iso().is_none() { eprintln!("NF_ISO not set — skipping"); return; }
    let t02 = tuning(0x0700_0002);
    for (weapon, raw) in [(0x12u32, 1.5f32), (0x0e, 5.0)] {
        for diff in [1, 2, 3] {
            for dist in [5.0, 20.0] {
                let Some(t) = measured("07000002.bin", diff, dist, weapon) else { eprintln!("[acc] {weapon:#x} diff {diff} d {dist}: no corridor"); continue };
                report(&format!("02 {weapon:#x} diff {diff} d {dist}"), &t);
                assert!(rolls_match(&t, 0x0700_0002, diff), "{weapon:#x} diff {diff} d {dist}: {} rolled of {} shots", t.rolled, t.shots);
                assert!(t.hits <= t.rolls, "{weapon:#x} diff {diff} d {dist}: {} hits from {} rolls ({} rolled) — rounds without a roll are landing", t.hits, t.rolls, t.rolled);
                assert!(t.hits + 4 >= t.rolled, "{weapon:#x} diff {diff} d {dist}: only {} of {} rolled hits landed", t.hits, t.rolled);
                for &d in &t.damage {
                    let ok = [1.0f32, 2.0].iter().any(|m| (d - raw * m * dmod(&t02, diff)).abs() < 1e-4);
                    assert!(ok, "{weapon:#x} diff {diff}: a hit cost {d}, not {raw} × (captain 1|2) × {}", dmod(&t02, diff));
                }
            }
        }
    }
}

/// 0700001B's astronauts (weapon 0x6A: one round of 5.0, interval 180 [D weapon_data +0x0C/+0x40]): about
/// three shots per 10 s and 5.0 × Plr_DMod (0.6 / 0.7 / 1.0 on 1B [M 0x1d3a80 case 0x700001b]) a hit;
/// the station never back-shots [M 0x139f50:24-48]. Sister: the difficulty factors differ, so a Hard
/// hit is not an Easy hit.
#[test]
fn astronauts_hit_like_the_chain() {
    if iso().is_none() { eprintln!("NF_ISO not set — skipping"); return; }
    let t1b = tuning(0x0700_001b);
    let mut per = Vec::new();
    for diff in [1, 2, 3] {
        for dist in [5.0, 10.0, 20.0] {
            let t = measured("0700001b.bin", diff, dist, 0x6a).expect("an astronaut fires at Bond");
            report(&format!("1b 0x6a diff {diff} d {dist}"), &t);
            assert!((3..=4).contains(&t.shots), "diff {diff} d {dist}: {} shots in 10 s (interval 180)", t.shots);
            assert!(t.hits <= t.rolled + 1, "diff {diff} d {dist}: {} hits from {} rolled", t.hits, t.rolled);
            let want = 5.0 * dmod(&t1b, diff);
            assert!(t.damage.iter().all(|&d| (d - want).abs() < 1e-4), "diff {diff}: {want} a hit, got {:?}", t.damage);
            per.push((diff, want));
        }
    }
    assert!((per[0].1 - per[8].1).abs() > 1.0, "Easy and Hard hits differ");
}

/// Bond's hit volume spans his body boxes (1.03 below obj+0x30 to 0.17 over the eye): a round at the shins
/// lands, a round 0.5 over the head flies on. Sister: the old cylinder (0.4 below to 1.4 above) did the reverse.
#[test]
fn bond_is_hit_at_the_shins_not_over_the_head() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, [128.0, 2.0, 128.0], 0.0);
    // A plain round: record 0x12 (+0x70 = 0x12, no bit 4 [D weapon_data]). Record 0x2c, the old
    // pick, is an explosive round (+0x70 & 4, radius 8, damage 100 [D]): its stop is
    // `Explode_Create` [M Bullet_CollisionHandler 0x12ce80:193-199], which the drone-round pass now
    // runs, so a round over the head would still hurt Bond through the floor blast.
    let shoot = |sim: &mut nf_view::scene::Scene, dy: f32| -> f32 {
        let s = sim.sim.as_mut().unwrap();
        s.drones.health = nf_game::bullet::PlayerHealth::default();
        let p = s.bond.pos;
        s.drones.bullets.push(nf_game::bullet::Bullet { pos: [p[0], p[1] + dy, p[2] + 2.0], dir: [0.0, 0.0, -1.0], speed: 15.0,
            travelled: 0.0, range: 100.0, damage: 10.0, owner: usize::MAX, state: 0, weapon: 0x12, flyby: false, bounces: 0,
            ..nf_game::bullet::Bullet::TAIL });
        s.step_bond(Input::default(), false, false, false);
        100.0 - s.drones.health.health
    };
    assert!(shoot(&mut scene, -0.8) > 0.0, "a shin-high round hits Bond");
    assert!(shoot(&mut scene, 0.6) > 0.0, "a head-high round hits Bond");
    assert_eq!(shoot(&mut scene, 1.2), 0.0, "a round 0.5 over the head misses");
}

/// A non-captain shooter of `weapon` (the captains' ×2 / ½-accuracy would mix two models in one cell);
/// up to 12 candidates, so the 10 u cell finds an open corridor.
fn measured_regular(level: &str, diff: u32, dist: f32, weapon: u32) -> Option<Tally> {
    let p = |d: &nf_game::drone::Drone| d.team == 1 && d.weapon == weapon && !d.captain;
    (0..12).find_map(|k| measure(level, diff, dist, 600, k, &p))
}

/// The pinned matrix: landed hits on Bond in the 10 s after a regular guard's first shot and the damage
/// of each, at 5 / 10 / 20 u and every difficulty — 07000002's rifle guard (0x12: 1.5 a round, bursts of
/// 6 at interval 6 for aggression 1) and pistol guard (0x0E: 5.0 × `+0x100` 2.0), 0700001B's astronaut
/// (0x6A: 5.0, interval 180). Per hit = `weapon_data+0x0C × Drone+0x100 × Plr_DMod` exactly [M
/// 0x139f50 + 0x1902c8; D weapon_data]; the hit counts follow `DoBulletAccuracy`'s p with the type's
/// stats [M 0x179740; L drone_stats] and are pinned at the measured deterministic value ± max(2, 20%).
/// Measured on this harness's FIXED stream (the `measure()` reset above): every case starts its
/// window from the same stream state, so the cells no longer ride on whatever the settle's
/// glass-passed rounds drew (guard-batch-repin's bisect; the values shifted one recalibration
/// step — e.g. the 0x12 diff-3 5-u cell 24 → 29 with 3 wobble-clip misses landing, 26 of 46 shots
/// rolled against p = 60 %).
/// join-reds-a: re-measured on the stream join (join-final 3c94fa9ea, the same fixed-stream harness):
/// the cells moved with the joined guard changes (not bisected to one change); the two-sided
/// chance test above still holds on every cell, and the per-hit pins are unchanged.
/// aim-stop-root: re-measured once AimStandFire drops the stand's root step (+0x38 [M 0x166328:117-123
/// → 0x13b490; L 07000011 the guard stops dead on the 0x68 entry]) - the shooter no longer slides
/// on into Bond on its stop, so the 5-u cells and the d1 20-u approach moved (the chance test holds).
/// Sister: the old chain (accuracy 5, aggression 0, the pistol at ×1) — its 0x12 cells landed
/// 8/2/7/11/16/16 at 5/20 u (60 in all) and its 0x0E hits cost half — fails the totals and the per-hit.
#[test]
fn hits_per_10s_and_damage_per_hit_are_pinned() {
    if iso().is_none() { eprintln!("NF_ISO not set — skipping"); return; }
    // (archive, level, weapon, raw × +0x100, diff, dist, pinned hits / 10 s)
    const PINS: &[(&str, u32, u32, f32, u32, f32, usize)] = &[
        ("07000002.bin", 0x0700_0002, 0x12, 1.5, 1, 5.0, 16), ("07000002.bin", 0x0700_0002, 0x12, 1.5, 1, 10.0, 14), ("07000002.bin", 0x0700_0002, 0x12, 1.5, 1, 20.0, 14),
        ("07000002.bin", 0x0700_0002, 0x12, 1.5, 2, 5.0, 15), ("07000002.bin", 0x0700_0002, 0x12, 1.5, 2, 10.0, 18), ("07000002.bin", 0x0700_0002, 0x12, 1.5, 2, 20.0, 19),
        ("07000002.bin", 0x0700_0002, 0x12, 1.5, 3, 5.0, 21), ("07000002.bin", 0x0700_0002, 0x12, 1.5, 3, 10.0, 21), ("07000002.bin", 0x0700_0002, 0x12, 1.5, 3, 20.0, 24),
        ("07000002.bin", 0x0700_0002, 0x0e, 10.0, 1, 5.0, 0), ("07000002.bin", 0x0700_0002, 0x0e, 10.0, 2, 5.0, 4), ("07000002.bin", 0x0700_0002, 0x0e, 10.0, 3, 5.0, 4),
        ("0700001b.bin", 0x0700_001b, 0x6a, 5.0, 1, 5.0, 2), ("0700001b.bin", 0x0700_001b, 0x6a, 5.0, 1, 10.0, 0), ("0700001b.bin", 0x0700_001b, 0x6a, 5.0, 1, 20.0, 1),
        ("0700001b.bin", 0x0700_001b, 0x6a, 5.0, 2, 5.0, 1), ("0700001b.bin", 0x0700_001b, 0x6a, 5.0, 2, 10.0, 1), ("0700001b.bin", 0x0700_001b, 0x6a, 5.0, 2, 20.0, 1),
        ("0700001b.bin", 0x0700_001b, 0x6a, 5.0, 3, 5.0, 1), ("0700001b.bin", 0x0700_001b, 0x6a, 5.0, 3, 10.0, 1), ("0700001b.bin", 0x0700_001b, 0x6a, 5.0, 3, 20.0, 1),
    ];
    let mut bad = Vec::new();
    let mut total_12 = 0usize;
    for &(arc, level, weapon, raw, diff, dist, want) in PINS {
        let label = format!("{arc} {weapon:#x} diff {diff} d {dist}");
        let Some(t) = measured_regular(arc, diff, dist, weapon) else { bad.push(format!("{label}: no measurement")); continue };
        report(&label, &t);
        let per = raw * dmod(&tuning(level), diff);
        // The pool round-trip (the hit is read off 100 − health, see fix_guard_weapons_live).
        let pool = 100.0_f32 - (100.0_f32 - per);
        if let Some(d) = t.damage.iter().find(|&&d| (d - pool).abs() > 1e-4 && (d - 100.0_f32 + (100.0_f32 - per * 2.0)).abs() > 1e-4) {
            bad.push(format!("{label}: a hit cost {d}, the chain gives {per} (x2 under TooClose / back-shot)"));
        }
        let tol = (want / 5).max(2);
        if t.hits.abs_diff(want) > tol { bad.push(format!("{label}: {} hits in 10 s, pinned {want} ± {tol}", t.hits)); }
        if weapon == 0x12 && dist != 10.0 { total_12 += t.hits; }
    }
    eprintln!("[acc] 0x12 hits at 5/20 u, all difficulties: {total_12} (the old chain: 60)");
    if total_12 * 10 < 60 * 13 { bad.push(format!("sister: the 0x12 total {total_12} is within 30% of the old chain's 60")); }
    assert!(bad.is_empty(), "{bad:#?}");
}
