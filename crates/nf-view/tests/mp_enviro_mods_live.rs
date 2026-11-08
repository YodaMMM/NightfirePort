//! mp-enviro-mods-proof: the Enviro-Mods rows pinned in real MP matches (the apply task's
//! "What's left", docs/reports/mp-enviro-mods-apply.md). NF_ISO-gated (the arenas load from the
//! disc).
//!
//! The rows, with the game's evidence:
//! - row 1 +0x1B8 Gun Emplacements: `GunImp_Create` 0x131f20:56-58 creates NO mount object in MP
//!   while the word is 0; :59-63 assigns the turret entry `MPSettings+0x1B4` (the weapon set)
//!   for 0..9 and `Rand_Rand(10)` for set 10 = Random [M]. The port draws that pick from a
//!   private stream keyed by the match seed (`seed ^ 0x4755_4e49`) instead of the shared
//!   `Rand_Rand` stream — the documented determinism substitution [S, gunimp.rs
//!   apply_mp_turrets]; the pin asserts the port's substitution, the <10 range and the per-mount
//!   identity are the game's [M].
//! - row 2 +0x1D4 Explosive Scenery: `Destroy_Create` 0x12c318:14-15 builds the class-0x18
//!   destroyables in MP only while the word is set [M].
//! - row 3 +0x1D0 Grapple: `MP_EquipPlayer` 0x1855f8:27-34 equips `Upgrade_MPWeapon(character,
//!   0x50)` (the Q-Claw) on every spawn while set [M].
//! - the mounts ride the online world hash (scene.rs net_world_fold: turret/active/hot/pitch/
//!   yaw/smoothed mirrors/heat/cooldown — not the machine-local fire_handle or the beam quad),
//!   so a two-machine run's tick hashes ARE the in-step mount proof.
//!
//! The mount-carrying arena is picked at run time from the census (the mp_world_sync inventory
//! print gates mounts off, so the census here loads with the mod on).

use nf_game::mp::MpSettings;
use std::sync::OnceLock;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

/// The MP arenas [M menu.rs MP_MAP_ROWS — the front end's map rows; D the parse].
const ARENAS: [u32; 8] = [0x0700_0024, 0x0700_0027, 0x0700_0029, 0x0700_0026, 0x0700_0023, 0x0700_0028, 0x0700_0025, 0x0700_004b];

/// Skyrail carries the class-0x18 destroyables — 8 of them (kind 10, hp 20, blast 5) with the
/// mod on [L the mp_world_sync_live inventory print, this session's run; M Destroy_Create
/// 0x12c318:14-15 gates them on the row].
const DESTROY_ARENA: u32 = 0x0700_0024;
const DESTROY_COUNT: usize = 8;

const SEED: u32 = 0x0bed_0bad;

fn settings(level: u32, mounts: u32, weapon_set: u32, explosive: u32, grapple: u32) -> MpSettings {
    let mut s = MpSettings::deathmatch(level);
    s.human_count = 2;
    s.player_count = 2;
    s.bot_count = 0;
    s.total_players = 2;
    s.seed = SEED;
    s.gun_emplacements = mounts;
    s.weapon_set = weapon_set;
    s.explosive_scenery = explosive;
    s.grapple = grapple;
    s.time_limit = 10 * 60;
    s
}

/// The census: (the first arena in MP_MAP_ROWS order whose 0x34 placements became mounts with
/// the mod on, the mount count, the per-mount turret with weapon set 5). Computed once per
/// binary run; `mount_census` pins it non-empty.
fn mount_arena() -> &'static Option<(u32, usize)> {
    static CELL: OnceLock<Option<(u32, usize)>> = OnceLock::new();
    CELL.get_or_init(|| {
        let Some(path) = iso() else { return None };
        for &level in &ARENAS {
            let s = settings(level, 1, 5, 1, 0);
            let scene = nf_view::load_mp_carry(&path, &format!("{level:08x}.bin"), false, None, s).expect("arena loads");
            let sim = scene.sim.as_ref().unwrap();
            let g = &sim.drones.props.gunimps;
            eprintln!("census {level:08x}: gunimps {} turrets {:?}", g.len(), g.iter().map(|x| x.turret).collect::<Vec<_>>());
            if !g.is_empty() {
                return Some((level, g.len()));
            }
        }
        None
    })
}

/// The mount census across every MP arena (the pick the task's pins rest on). The
/// mp_world_sync inventory print loads with `gun_emplacements` 0, so its print can never show
/// the mounts; this census loads with the mod on and prints the counts it finds.
#[test]
fn mount_census_finds_a_mount_carrying_arena() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    for &level in &ARENAS {
        let s = settings(level, 1, 5, 1, 0);
        let scene = nf_view::load_mp_carry(&path, &format!("{level:08x}.bin"), false, None, s).expect("arena loads");
        let sim = scene.sim.as_ref().unwrap();
        let g = &sim.drones.props.gunimps;
        eprintln!("{level:08x}: gunimps {} turrets {:?} destroy {}", g.len(), g.iter().map(|x| x.turret).collect::<Vec<_>>(), sim.drones.props.destroy.len());
    }
    let Some((level, n)) = *mount_arena() else { panic!("no MP arena carries 0x34 mounts with the mod on — the census above is empty") };
    eprintln!("mount arena: {level:08x} with {n} mounts");
    assert!(n > 0);
}

/// Row 1 OFF: no mount object at all on the mount-carrying arena [M GunImp_Create 0x131f20:56-58
/// — the MP arm returns 0 while MPSettings+0x1B8 is 0; the build's gate scene.rs clears the
/// created mounts (the port's arm of the same read)].
#[test]
fn row1_off_no_mounts_on_the_mount_arena() {
    if iso().is_none() { eprintln!("skip: NF_ISO not set"); return; }
    let Some((level, _)) = *mount_arena() else { eprintln!("skip: no mount arena"); return };
    let path = iso().unwrap();
    let s = settings(level, 0, 5, 1, 0);
    let scene = nf_view::load_mp_carry(&path, &format!("{level:08x}.bin"), false, None, s).expect("arena loads");
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.props.gunimps.is_empty(), "the mod-off load must create no mounts [M 0x131f20:56-58], got {}", sim.drones.props.gunimps.len());
}

/// Row 1 ON, weapon set 5: every mount's turret entry is the set — 5 [M 0x131f20:59-62, the
/// 0..9 arm].
#[test]
fn row1_on_weapon_set5_turret5_on_every_mount() {
    if iso().is_none() { eprintln!("skip: NF_ISO not set"); return; }
    let Some((level, n)) = *mount_arena() else { eprintln!("skip: no mount arena"); return };
    let path = iso().unwrap();
    let s = settings(level, 1, 5, 1, 0);
    let scene = nf_view::load_mp_carry(&path, &format!("{level:08x}.bin"), false, None, s).expect("arena loads");
    let sim = scene.sim.as_ref().unwrap();
    let g = &sim.drones.props.gunimps;
    assert_eq!(g.len(), n);
    assert!(g.iter().all(|x| x.turret == 5), "turrets {:?}", g.iter().map(|x| x.turret).collect::<Vec<_>>());
}

/// Row 1 ON, weapon set 10 (Random): every mount's pick is < 10 and equals the private
/// seed-keyed stream's draw for its mount index [S — the port's documented substitution for the
/// game's shared-stream `Rand_Rand(10)` at 0x131f20:63; the game's stream state at create time
/// is play-history no test can reproduce, so the substitution IS the pin]. The assignment is
/// idempotent — the build and the match start both apply it, same seed, same picks [M lib.rs
/// apply_mp_turrets' call sites].
#[test]
fn row1_on_weapon_set10_seed_keyed_picks_below10() {
    if iso().is_none() { eprintln!("skip: NF_ISO not set"); return; }
    let Some((level, n)) = *mount_arena() else { eprintln!("skip: no mount arena"); return };
    let path = iso().unwrap();
    let s = settings(level, 1, 10, 1, 0);
    let scene = nf_view::load_mp_carry(&path, &format!("{level:08x}.bin"), false, None, s).expect("arena loads");
    let sim = scene.sim.as_ref().unwrap();
    let g = &sim.drones.props.gunimps;
    assert_eq!(g.len(), n);
    let mut trng = nf_game::drone::Rng::new(SEED ^ 0x4755_4e49);
    let picks: Vec<u8> = (0..n).map(|_| trng.rand(10) as u8).collect();
    let got: Vec<u8> = g.iter().map(|x| x.turret).collect();
    assert!(got.iter().all(|&t| t < 10), "turrets {got:?} must all be < 10 [M 0x131f20:63 — Rand_Rand(10)]");
    assert_eq!(got, picks, "the picks must be the private seed-keyed stream's draws [S — gunimp.rs apply_mp_turrets]");
    // Idempotent: a second application (the match start's re-apply) keeps the same assignment.
    let mut again = g.clone();
    nf_game::gunimp::apply_mp_turrets(&mut again, 10, SEED);
    assert_eq!(again.iter().map(|x| x.turret).collect::<Vec<_>>(), got, "apply twice, same picks (the build + match-start idempotence)");
}

// ── the two-machine in-step run ──────────────────────────────────────────────────────────────

/// One machine of a two-machine match (the live match-start setup: the private generator, the
/// joiner's `swap_local_slot` — mp_world_sync_live's harness shape).
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

/// A per-slot walker (mp_world_sync_live's shape, trimmed): heads for the nearest live pickup,
/// turning toward it, pressing E now and then, jumping when stuck, giving up on an unreachable
/// target; every 90 ticks it turns to the nearest live breakable/destroyable in range and fires
/// once lined up.
struct Walker { target: Option<[f32; 3]>, since: u32, best: f32, skip: Vec<[f32; 3]>, picks: u32, shoot: Option<[f32; 3]>, aim_ticks: u32 }

impl Walker {
    fn new() -> Walker { Walker { target: None, since: 0, best: f32::MAX, skip: Vec::new(), picks: 0, shoot: None, aim_ticks: 0 } }

    fn input(&mut self, sim: &nf_view::scene::Sim, slot: usize, t: u32) -> nf_net::Input {
        let mut i = nf_net::NEUTRAL;
        i.held = 2;
        let Some(b) = body(sim, slot) else { return i };
        let d = &sim.drones;
        if self.target.is_none() {
            self.picks += 1;
            let mut cands: Vec<[f32; 3]> = d.pickups.iter().filter(|p| !p.gone && !p.hidden)
                .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]]).collect();
            let skip = self.skip.clone();
            cands.retain(|p| !skip.iter().any(|s| s == p));
            let dist = |p: &[f32; 3]| (p[0] - b.pos[0]).powi(2) + (p[2] - b.pos[2]).powi(2);
            cands.sort_by(|a, c| dist(a).total_cmp(&dist(c)));
            self.target = cands.get(slot % cands.len().max(1)).copied().or(cands.first().copied());
            self.since = 0;
            self.best = f32::MAX;
        }
        if t % 90 == 0 && self.shoot.is_none() {
            let h = b.head();
            let centre = |lo: [f32; 3], hi: [f32; 3]| [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0];
            let boxes: Vec<[f32; 3]> = d.props.destroy.iter().filter(|x| !x.dead).map(|x| centre(x.lo, x.hi))
                .chain(d.breaks.list.iter().filter(|x| !x.dead).map(|x| centre(x.lo, x.hi)))
                .filter(|c| ((c[0] - h[0]).powi(2) + (c[2] - h[2]).powi(2)) < 40.0 * 40.0)
                .collect();
            self.shoot = boxes.into_iter().filter(|c| sim.world.segment(h, *c).map_or(true, |t| t > 0.97))
                .min_by(|a, c| { let q = |p: &[f32; 3]| (p[0] - h[0]).powi(2) + (p[2] - h[2]).powi(2); q(a).total_cmp(&q(c)) });
            self.aim_ticks = 0;
        }
        if let Some(c) = self.shoot {
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
        if dist < 0.6 || self.since > 400 {
            if self.since > 400 { self.skip.push(tg); }
            self.target = None;
        }
        i
    }
}

/// The two machines' mount arrays as comparable rows (creation + every hashed field; the
/// machine-local `fire_handle` and the per-shot `beam` stay out of the hash [M scene.rs
/// net_world_fold], so the rows leave them out too).
fn mount_rows(sim: &nf_view::scene::Sim) -> Vec<(usize, u8, bool, bool, u32, u32, u32, u32, u32, u32)> {
    sim.drones.props.gunimps.iter().map(|g| (
        g.placement, g.turret, g.active, g.hot,
        g.pitch.to_bits(), g.yaw.to_bits(), g.pitch_s.to_bits(), g.yaw_s.to_bits(),
        g.heat.to_bits(), g.cooldown.to_bits(),
    )).collect()
}

/// The in-step proof the old unconditional clear was waiting for: two machines, mounts ON
/// (weapon set 10 — the seeded picks must agree too), walkers live on both. Every tick's world
/// hash must agree, and after the run both machines' mount arrays must be identical — creation
/// AND every hashed field.
#[test]
fn row1_online_two_machines_agree_on_every_tick_with_mounts_on() {
    if iso().is_none() { eprintln!("skip: NF_ISO not set"); return; }
    let Some((level, n)) = *mount_arena() else { eprintln!("skip: no mount arena"); return };
    let path = iso().unwrap();
    let s = settings(level, 1, 10, 1, 0);
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    // Creation agrees before the first tick.
    assert_eq!(mount_rows(a.sim.as_ref().unwrap()), mount_rows(b.sim.as_ref().unwrap()), "the two machines build the same mounts");
    assert_eq!(n, a.sim.as_ref().unwrap().drones.props.gunimps.len());
    let mut walkers = [Walker::new(), Walker::new()];
    const TICKS: u32 = 1500;
    for t in 0..TICKS {
        let row: [nf_net::Input; nf_net::MAX_AGENTS] = {
            let sa = a.sim.as_ref().unwrap();
            let mut r = [nf_net::NEUTRAL; nf_net::MAX_AGENTS];
            for sl in 0..2 { r[sl] = walkers[sl].input(sa, sl, t); }
            r
        };
        let ha = nf_view::net_tick(a.sim.as_mut().unwrap(), &row, 0);
        let hb = nf_view::net_tick(b.sim.as_mut().unwrap(), &row, 1);
        assert_eq!(ha, hb, "tick {t}: the world hashes diverged with mounts on");
    }
    let (sa, sb) = (a.sim.as_ref().unwrap(), b.sim.as_ref().unwrap());
    let (ra, rb) = (mount_rows(sa), mount_rows(sb));
    eprintln!("{level:08x}: {TICKS} ticks agree; mounts {} — A active {:?} heat {:?} | B active {:?} heat {:?}",
        ra.len(),
        sa.drones.props.gunimps.iter().map(|g| g.active).collect::<Vec<_>>(), sa.drones.props.gunimps.iter().map(|g| g.heat).collect::<Vec<_>>(),
        sb.drones.props.gunimps.iter().map(|g| g.active).collect::<Vec<_>>(), sb.drones.props.gunimps.iter().map(|g| g.heat).collect::<Vec<_>>());
    assert_eq!(ra, rb, "the mount arrays must be identical after the run (creation + hashed state)");
    // Sister: the world fold can fail — one machine's mount aim nudged alone diverges the hash
    // (a mount desync is DETECTED, the in-step proof's teeth).
    let mut b2 = machine(&path, &s, 1);
    b2.sim.as_mut().unwrap().drones.props.gunimps[0].yaw += 0.5;
    let row = [nf_net::NEUTRAL; nf_net::MAX_AGENTS];
    let ha = nf_view::net_tick(a.sim.as_mut().unwrap(), &row, 0);
    let hb2 = nf_view::net_tick(b2.sim.as_mut().unwrap(), &row, 1);
    assert_ne!(ha, hb2, "a one-machine mount nudge must diverge the hash (the fold sees the mounts)");
}

/// Row 2 OFF: `props.destroy` is EMPTY on the destroyable arena; ON: the arena's destroyables
/// are created [M Destroy_Create 0x12c318:14-15 — the class-0x18 creates are MP-gated on
/// MPSettings+0x1D4; L the inventory print: 8 on Skyrail with the mod on].
#[test]
fn row2_off_empty_destroy_on_fills_it() {
    if iso().is_none() { eprintln!("skip: NF_ISO not set"); return; }
    let path = iso().unwrap();
    let arena = format!("{DESTROY_ARENA:08x}.bin");
    let off = settings(DESTROY_ARENA, 0, 5, 0, 0);
    let scene = nf_view::load_mp_carry(&path, &arena, false, None, off).expect("arena loads");
    let n_off = scene.sim.as_ref().unwrap().drones.props.destroy.len();
    assert_eq!(n_off, 0, "the mod-off load must create no class-0x18 destroyables [M 0x12c318:14-15]");
    let on = settings(DESTROY_ARENA, 0, 5, 1, 0);
    let scene = nf_view::load_mp_carry(&path, &arena, false, None, on).expect("arena loads");
    let n_on = scene.sim.as_ref().unwrap().drones.props.destroy.len();
    assert_eq!(n_on, DESTROY_COUNT, "Skyrail's class-0x18 destroyables with the mod on [L the inventory print]");
}

/// Row 3 ON: the load's kit owns the Q-Claw 0x50; OFF: it does not [M MP_EquipPlayer
/// 0x1855f8:27-34 — `Upgrade_MPWeapon(character, 0x50)` while MPSettings+0x1D0 is set; the
/// port's arm nf_game::mp::equip::mp_equip_player runs at the load (lib.rs, every MP spawn)].
#[test]
fn row3_on_the_loads_kit_owns_0x50_off_does_not() {
    if iso().is_none() { eprintln!("skip: NF_ISO not set"); return; }
    let Some((level, _)) = *mount_arena() else { eprintln!("skip: no mount arena"); return };
    let path = iso().unwrap();
    let arena = format!("{level:08x}.bin");
    let on = settings(level, 0, 5, 1, 1);
    let scene = nf_view::load_mp_carry(&path, &arena, false, None, on).expect("arena loads");
    let inv = &scene.sim.as_ref().unwrap().drones.inventory;
    let owned: Vec<usize> = (0..inv.owned.len()).filter(|&i| inv.owned[i]).collect();
    assert!(inv.owned.get(0x50).copied().unwrap_or(false), "the grapple-on load must own 0x50 [M 0x1855f8:27-34], owned {owned:x?}");
    let off = settings(level, 0, 5, 1, 0);
    let scene = nf_view::load_mp_carry(&path, &arena, false, None, off).expect("arena loads");
    let inv = &scene.sim.as_ref().unwrap().drones.inventory;
    assert!(!inv.owned.get(0x50).copied().unwrap_or(false), "the grapple-off load must not own 0x50");
}

// ── mp-enviro-check: a player mounts, fires and leaves a gun online ─────────────────────────

/// Put slot `slot`'s body on machine `sim` at `pos` facing `yaw` (the same placement on every
/// machine — identical state, so the run stays in step).
fn place(sim: &mut nf_view::scene::Sim, slot: usize, pos: [f32; 3], yaw: f32, pitch: f32) {
    let cel = sim.world.find_cel(pos);
    let b = if slot == sim.mp_local_slot { &mut sim.bond } else { sim.mp_bonds[slot].as_mut().expect("the slot's body") };
    b.pos = pos;
    b.prev_pos = pos;
    b.yaw = yaw;
    b.pitch = pitch;
    b.vel = [0.0; 3];
    b.cel = cel;
}

/// Online, a slot's use key reaches the mounted guns (`Player_Activate` 0x1a5170 runs per
/// player and its one pick covers class 0x47 [M :100-179] → `GunImp_Activate` 0x132f00 links
/// that player [M :17-21]); the mount then reads THAT slot's stick and trigger every tick
/// (`GunImp_Update` 0x1320d8 [M :119-135, :247-331]) and the use key dismounts it [M :119-123].
/// Both machines must see the same mount (owner, aim, heat), carry the same body to the seat
/// [M 0x1320d8:175-194] and agree on every tick's hash. Before the fix the online use pass had
/// no class-0x47 arm: the press did nothing on either machine.
#[test]
fn online_slot_mounts_fires_and_dismounts_on_both_machines() {
    if iso().is_none() { eprintln!("skip: NF_ISO not set"); return; }
    let Some((level, _)) = *mount_arena() else { eprintln!("skip: no mount arena"); return };
    let path = iso().unwrap();
    let s = settings(level, 1, 7, 1, 0);
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    let g = a.sim.as_ref().unwrap().drones.props.gunimps[0].clone();
    // Behind the gun, on its seat line, facing it, looking a little down (the live run's pose).
    let (sy, cy) = g.yaw_base.sin_cos();
    let stand = [g.pos[0] + sy * g.seat * 1.3, g.pos[1], g.pos[2] + cy * g.seat * 1.3];
    let yaw = (g.pos[0] - stand[0]).atan2(g.pos[2] - stand[2]);
    for m in [&mut a, &mut b] { place(m.sim.as_mut().unwrap(), 0, stand, yaw, -0.2); }
    const PRESS: u32 = 2;
    const FIRE: std::ops::Range<u32> = 10..200;
    const LEAVE: u32 = 250;
    let mut fired_a = 0;
    let mut fired_b = 0;
    for t in 0..300u32 {
        let mut row = [nf_net::NEUTRAL; nf_net::MAX_AGENTS];
        for r in row.iter_mut().take(2) { r.held = 2; }
        row[0].use_key = t == PRESS || t == LEAVE;
        row[0].trigger = FIRE.contains(&t);
        row[0].turn = if FIRE.contains(&t) && t % 40 < 20 { 0.01 } else { 0.0 };
        let ha = nf_view::net_tick(a.sim.as_mut().unwrap(), &row, 0);
        let hb = nf_view::net_tick(b.sim.as_mut().unwrap(), &row, 1);
        assert_eq!(ha, hb, "tick {t}: the world hashes diverged around the mount");
        let (sa, sb) = (a.sim.as_ref().unwrap(), b.sim.as_ref().unwrap());
        assert_eq!(mount_rows(sa), mount_rows(sb), "tick {t}: the mount rows differ");
        let owners = |s: &nf_view::scene::Sim| s.drones.props.gunimps.iter().map(|g| (g.active, if g.active { g.owner } else { 0 })).collect::<Vec<_>>();
        assert_eq!(owners(sa), owners(sb), "tick {t}: the mount owners differ");
        fired_a += sa.drones.props.gunimps.iter().filter(|g| g.beam.is_some()).count();
        fired_b += sb.drones.props.gunimps.iter().filter(|g| g.beam.is_some()).count();
        if t == PRESS {
            assert!(sa.drones.props.gunimps[0].active && sa.drones.props.gunimps[0].owner == 0, "slot 0's press mounts gun 0 on its own machine");
            assert!(sb.drones.props.gunimps[0].active && sb.drones.props.gunimps[0].owner == 0, "…and on the other machine");
            assert!(sa.local_gun_mounted() && !sb.local_gun_mounted(), "the mount is slot 0's: machine B's own player stays free");
        }
        if t == FIRE.end {
            let (ga, gb) = (&sa.drones.props.gunimps[0], &sb.drones.props.gunimps[0]);
            assert!(ga.heat > 0.0, "the held trigger fired the mount (heat {})", ga.heat);
            assert!(ga.yaw != 0.0, "slot 0's stick turned the mount");
            // The carried body [M 0x1320d8:175-194]: slot 0 sits on the seat on BOTH machines.
            let (pa, pb) = (sa.bond.pos, sb.mp_bonds[0].as_ref().unwrap().pos);
            assert_eq!(pa, pb, "slot 0's body agrees across the machines");
            let seat = ga.mounted_pose().0;
            assert!((0..3).all(|k| (pa[k] - seat[k]).abs() < 1e-4), "slot 0 rides the seat {seat:?}, got {pa:?}");
            let _ = gb;
        }
        if t == LEAVE {
            assert!(!sa.drones.props.gunimps[0].active && !sb.drones.props.gunimps[0].active, "slot 0's second press dismounts on both machines");
        }
    }
    eprintln!("{level:08x}: beam ticks A {fired_a} B {fired_b}");
    assert!(fired_a > 0 && fired_a == fired_b, "both machines drew the same shots (A {fired_a}, B {fired_b})");
}
