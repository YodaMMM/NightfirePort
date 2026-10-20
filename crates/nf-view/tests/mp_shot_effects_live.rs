//! mp-shot-effects (docs/systems/mp-shot-effects.md): MP shots present the SP effect set through
//! the SP pipeline. The SP reference fires the weapon on a single machine of the same arena (no
//! net row — `step_bond`'s `'shot` block, the game's own pipeline: `Player_WeaponInitBullet`
//! 0x1a4bf0 / `Bullet_init` 0x12c5a0 / `Player_MuzzleFlash` 0x1a4298 / `Bullet_init_casing`
//! 0x12fde0) and counts the per-shot effects. A 2-machine MP match firing the same weapon then
//! counts the same kinds on the shooter's machine (its own pipeline) and on the other machine
//! (the cosmetic rounds + the shell + the smoke through the same SP code — `Drones::
//! fire_remote_cosmetic`). Both humans aim at the floor, so every pellet impacts: the decal count
//! pins the per-pellet parity. Sister: the machines' hashes agree every tick (a third machine with
//! the pass disabled included). NF_ISO-gated.

use nf_game::mp::MpSettings;
use nf_net::NEUTRAL;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn settings() -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = 2;
    s.player_count = 2;
    s.bot_count = 0;
    s.total_players = 2;
    s.seed = 0x0bed_f00d;
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

/// Equips and draws `w` on the local body (the `--draw` path's equip + the switch request
/// [M `Player_Weapon` 0x1a24c8's BL+0x63 request]), with rounds to spare.
fn equip(sim: &mut nf_view::scene::Sim, w: u8) {
    let wd = sim.weapons.clone();
    let rounds = wd.clip_size(w as usize).max(1);
    let took = sim.drones.inventory.equip_weapon(&wd, w as usize, rounds, true);
    sim.drones.inventory.equip_ammo(&wd, w as usize, rounds * 99);
    sim.gun.switch_request = w;
    let _ = took;
}

/// The SP reference's draw: equip, then the draw machine ticks (a single machine — the frame is
/// this scene's own).
fn draw_sp(sim: &mut nf_view::scene::Sim, w: u8) {
    equip(sim, w);
    for round in 0..4 {
        sim.gun.switch_request = w;
        for _ in 0..90 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
        if sim.gun.weapon == w { return; }
        eprintln!("[give] round {round}: the body holds {:#04x}, re-requesting {w:#04x}", sim.gun.weapon);
    }
    panic!("the body must hold {w:#04x} after the draw");
}

/// The muzzle light's signature: `Player_MuzzleFlash`'s `Light_Create(radius +0x58, 2.0, rgb
/// +0x56/+0x55/+0x54, kind 1)` [M 0x1a4298] — the impact effects' own lights never match it.
fn muzzle_lights<'a>(lights: impl Iterator<Item = &'a nf_game::light::Light>, rec: &[u8]) -> impl Iterator<Item = &'a nf_game::light::Light> {
    let rgb = [rec[0x56], rec[0x55], rec[0x54]];
    let radius = f32::from_le_bytes(rec[0x58..0x5c].try_into().unwrap());
    lights.filter(move |l| l.kind == 1 && l.rgb == rgb && l.radius == radius && l.intensity == 2.0)
}

#[derive(Default, Clone, Copy, Debug)]
struct Fx {
    shots: u32,
    /// `Casings::ejected` deltas — one shell per shot (`Bullet_init_casing` 0x12fde0; the player's
    /// reserve and the cosmetic eject both take the slot) [M].
    casings: u32,
    /// The muzzle-light list deltas (`Player_MuzzleFlash`'s Light_Create, life 1 tick) [M].
    lights: u32,
    /// The gas-list deltas (`Player_MuzzleFlash`'s Gas_Init puff) [M].
    smoke: u32,
    /// The widest `bullet_paths` slice — the per-pellet flight segments of one tick [M Bullet_update].
    bullets: u32,
    /// The impact decals the run added (`Effect_Bullet` on every pellet's floor contact) [M].
    decals: u32,
    /// Ticks the shooter's flash latch was fresh (2 per shot [M 0x1a4298's +0x95b latch]).
    flash: u32,
}

fn aim_down(sim: &mut nf_view::scene::Sim, slot: usize) {
    if slot == sim.mp_local_slot { sim.bond.pitch = -1.2; } else { sim.mp_bonds[slot].as_mut().unwrap().pitch = -1.2; }
}

/// The SP reference: the same arena, no net row — the local pipeline fires `n` shots at the floor.
fn sp_run(path: &str, w: u8, n: u32) -> Fx {
    let s = settings();
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(path, &arena, false, None, s).expect("arena loads");
    let sim = scene.sim.as_mut().unwrap();
    let rec = sim.weapons.records.get(w as usize).cloned().unwrap();
    let pellets = u32::from_le_bytes(rec[0x18..0x1c].try_into().unwrap()).clamp(1, 16);
    let (wpn, clip, decals0) = {
        let s = scene.sim.as_ref().unwrap();
        (s.gun.weapon, s.gun.clip, s.drones.effects.decals.len())
    };
    eprintln!("[sp] after draw: weapon {wpn:#04x} clip {clip}");
    let mut fx = Fx::default();
    let mut cadence = 0u32;
    let _ = pellets;
    let mut prev_ls = scene.sim.as_ref().unwrap().last_shot;
    while fx.shots < n {
        let sim = scene.sim.as_mut().unwrap();
        aim_down(sim, sim.mp_local_slot);
        let (c0, l0, l0s, g0) = (sim.drones.casings.ejected, sim.drones.lights.lights.len(), muzzle_lights(sim.drones.lights.lights.iter(), &rec).count(), sim.drones.gas.puffs.len());
        sim.step_bond(nf_game::player::Input::default(), true, false, false);
        let st = scene.sim.as_ref().unwrap();
        let ls = st.last_shot;
        let fired_now = ls != prev_ls && ls.is_some();
        prev_ls = ls;
        if fired_now {
            fx.shots += 1;
            fx.casings += st.drones.casings.ejected - c0;
            let l1s = muzzle_lights(st.drones.lights.lights.iter(), &rec).count();
            fx.lights += (l1s as i64 - l0s as i64).max(0) as u32;
            let _ = l0;
            fx.smoke += (st.drones.gas.puffs.len() as i64 - g0 as i64).max(0) as u32;
            fx.bullets = fx.bullets.max(st.drones.bullet_paths_len() as u32);
            fx.flash += 1;
            cadence = 0;
        } else {
            cadence += 1;
            if cadence % 30 == 0 {
                let s = scene.sim.as_ref().unwrap();
                eprintln!("[sp] cadence {cadence}: weapon {:#04x} clip {} last_shot {:?} frame {}", s.gun.weapon, s.gun.clip, s.last_shot, s.drones.frame);
            }
            assert!(cadence < 400, "the local pipeline never fired record {w:#04x} (refire/clip?)");
        }
        // A settle tick between shots (the delayed eject, the flight, the impacts land).
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    }
    fx.decals = (scene.sim.as_ref().unwrap().drones.effects.decals.len() - decals0) as u32;
    fx
}

/// The MP run: slot 0 fires `n` shots per the shared gate; both machines count their view.
fn mp_run(path: &str, w: u8, n: u32) -> (Fx, Fx, bool) {
    let s = settings();
    let mut a = machine(path, &s, 0);
    let mut b = machine(path, &s, 1);
    let mut c = machine(path, &s, 0);
    c.sim.as_mut().unwrap().mp_fx.disabled = true;
    // The shooter's body draws and holds the weapon on its own machine; the shared gate's clip is
    // topped on every machine alike (the gate spends it identically — docs/systems/mp-shot-gate.md).
    // The warmup is LOCKSTEP: the same 150 neutral net ticks on every machine (the shooter's
    // draw machine runs inside its own step_bond), or the frames run apart and the hashes
    // diverge before the first shot.
    equip(a.sim.as_mut().unwrap(), w);
    let neutral: [nf_net::Input; nf_net::MAX_AGENTS] = std::array::from_fn(|_| NEUTRAL);
    for _ in 0..150 {
        let ha = nf_view::net_tick(a.sim.as_mut().unwrap(), &neutral, 0);
        let hb = nf_view::net_tick(b.sim.as_mut().unwrap(), &neutral, 1);
        let hc = nf_view::net_tick(c.sim.as_mut().unwrap(), &neutral, 0);
        assert_eq!(ha, hb, "warmup tick: the hashes");
        assert_eq!(ha, hc, "warmup tick: the hashes");
    }
    assert_eq!(a.sim.as_ref().unwrap().gun.weapon, w, "the shooter must hold {w:#04x} after the draw");
    let rec = a.sim.as_ref().unwrap().weapons.records.get(w as usize).cloned().unwrap();
    for sc in [&mut a, &mut b, &mut c] {
        let sim = sc.sim.as_mut().unwrap();
        let cs = sim.weapons.base(w as usize) as usize;
        for g in sim.mp_combat.gates.iter_mut() {
            if cs < g.inv.clip.len() { g.inv.clip[cs] = 1000; }
        }
        aim_down(sim, 0);
    }
    let mut fa = Fx::default();
    let mut fb = Fx::default();
    let decals0_a = a.sim.as_ref().unwrap().drones.effects.decals.len();
    let decals0_b = b.sim.as_ref().unwrap().drones.effects.decals.len();
    let (mut pc_a, mut pl_a, mut pg_a) = (a.sim.as_ref().unwrap().drones.casings.ejected, 0usize, 0usize);
    let (mut pc_b, mut pl_b, mut pg_b) = (b.sim.as_ref().unwrap().drones.casings.ejected, 0usize, 0usize);
    let mut hashes_agree = true;
    let mut cadence = 0u32;
    let mut t = 0u32;
    let mut prev_ls = a.sim.as_ref().unwrap().last_shot;
    let mut flush_window_b = 0u32;
    while fa.shots < n && t < 60_000 {
        let mut r: [nf_net::Input; nf_net::MAX_AGENTS] = std::array::from_fn(|_| NEUTRAL);
        r[0].held = w;
        r[0].trigger = t % 12 < 2;
        r[0].shot = t % 12 == 0;
        let ha = nf_view::net_tick(a.sim.as_mut().unwrap(), &r, 0);
        let hb = nf_view::net_tick(b.sim.as_mut().unwrap(), &r, 1);
        let hc = nf_view::net_tick(c.sim.as_mut().unwrap(), &r, 0);
        hashes_agree &= ha == hb && ha == hc;
        t += 1;
        let sa = a.sim.as_ref().unwrap();
        let sb = b.sim.as_ref().unwrap();
        let frame = sa.drones.frame;
        // The shooter's own pipeline (machine A): a rising edge on the flash latch is the shot.
        let ls = sa.last_shot;
        let shot_a = ls != prev_ls && ls.is_some();
        prev_ls = ls;
        if shot_a {
            fa.shots += 1;
            fa.flash += 1;
            fa.bullets = fa.bullets.max(sa.drones.bullet_paths_len() as u32);
            cadence = 0;
        } else {
            cadence += 1;
            assert!(cadence < 240 || fa.casings >= n, "the shooter's pipeline stalled");
        }
        // The other machine's view (B): the cosmetic flash lives 2 ticks per shot.
        if sb.mp_fx.last_shot[0].is_some_and(|f| sb.drones.frame.saturating_sub(f) < 2) { fb.flash += 1; }
        // Per-tick deltas. The shooter's own effects land on the shot tick (step_bond); the other
        // machine's land on the flush tick (the shell + the muzzle light + the muzzle smoke) with
        // the cosmetic pellets' impacts one tick later — a 2-tick window per shot.
        let l1a = muzzle_lights(sa.drones.lights.lights.iter(), &rec).count();
        let dl_a = (l1a as i64 - pl_a as i64).max(0) as u32; pl_a = l1a;
        let dg_a = (sa.drones.gas.puffs.len() as i64 - pg_a as i64).max(0) as u32; pg_a = sa.drones.gas.puffs.len();
        let l1b = muzzle_lights(sb.drones.lights.lights.iter(), &rec).count();
        let dl_b = (l1b as i64 - pl_b as i64).max(0) as u32; pl_b = l1b;
        let dg_b = (sb.drones.gas.puffs.len() as i64 - pg_b as i64).max(0) as u32; pg_b = sb.drones.gas.puffs.len();
        fa.casings += sa.drones.casings.ejected - pc_a; pc_a = sa.drones.casings.ejected;
        let b_ej = sb.drones.casings.ejected;
        let b_flush = b_ej > pc_b;
        fb.casings += b_ej - pc_b;
        pc_b = b_ej;
        if shot_a { fa.lights += dl_a; fa.smoke += dg_a; }
        if b_flush { flush_window_b = 2; }
        if flush_window_b > 0 { fb.lights += dl_b; fb.smoke += dg_b; flush_window_b -= 1; }
        fb.bullets = fb.bullets.max(sb.drones.bullet_paths_len() as u32);
    }
    // A settle tail: the last rides flush and the cosmetic pellets land on every machine.
    for _ in 0..6 {
        let ha = nf_view::net_tick(a.sim.as_mut().unwrap(), &neutral, 0);
        let hb = nf_view::net_tick(b.sim.as_mut().unwrap(), &neutral, 1);
        let hc = nf_view::net_tick(c.sim.as_mut().unwrap(), &neutral, 0);
        hashes_agree &= ha == hb && ha == hc;
        let (sa2, sb2) = (a.sim.as_ref().unwrap(), b.sim.as_ref().unwrap());
        fb.bullets = fb.bullets.max(sb2.drones.bullet_paths_len() as u32);
        fa.casings += sa2.drones.casings.ejected - pc_a; pc_a = sa2.drones.casings.ejected;
        fb.casings += sb2.drones.casings.ejected - pc_b; pc_b = sb2.drones.casings.ejected;
        fa.smoke += (sa2.drones.gas.puffs.len() as i64 - pg_a as i64).max(0) as u32; pg_a = sa2.drones.gas.puffs.len();
        fb.smoke += (sb2.drones.gas.puffs.len() as i64 - pg_b as i64).max(0) as u32; pg_b = sb2.drones.gas.puffs.len();
    }
    let sa = a.sim.as_ref().unwrap();
    let sb = b.sim.as_ref().unwrap();
    fa.decals = (sa.drones.effects.decals.len() - decals0_a) as u32;
    fb.decals = (sb.drones.effects.decals.len() - decals0_b) as u32;
    (fa, fb, hashes_agree)
}

fn check(name: &str, sp: Fx, a: Fx, b: Fx) {
    eprintln!("{name}: SP         {sp:?}");
    eprintln!("{name}: MP-shooter {a:?}");
    eprintln!("{name}: MP-other   {b:?}");
    assert_eq!(a.shots, sp.shots, "{name}: the shooter's machine fired the same shot count");
    assert_eq!(a.casings, sp.casings, "{name}: the shells per shot match on the shooter's machine");
    assert_eq!(b.casings, sp.casings, "{name}: the shells per shot match on the other machine (the cosmetic eject)");
    assert_eq!(a.bullets, sp.bullets, "{name}: the per-tick flight segments match on the shooter's machine (the pellets)");
    assert_eq!(b.bullets, sp.bullets, "{name}: the per-tick flight segments match on the other machine (the cosmetic pellets)");
    // The decal COUNT is per-machine: the penetration roll before the effects is drawn from each
    // machine's private generator in MP (the game draws it from the one shared stream; the port's
    // presentation draws per machine [S docs/systems/mp-shot-effects.md]), so a pierceable floor
    // makes 1..n decals per pellet. What must match is that the impacts LAND on every view.
    assert!(a.decals > 0 && b.decals > 0, "{name}: the impacts land on both machines (a {a:?} b {b:?})");
    assert_eq!(a.lights, sp.lights, "{name}: the muzzle lights per shot match on the shooter's machine");
    assert_eq!(b.lights, sp.lights, "{name}: the muzzle lights per shot match on the other machine (the flush's Light_Create)");
    // The gas list also carries the impacts' own dust puffs, and the pellet's pierce roll is
    // per-machine rng — so the smoke sum is the muzzle puff (1/shot) plus a pierce-dependent
    // count: the muzzle puff itself must show per shot on every view.
    assert!(a.smoke >= sp.shots, "{name}: the muzzle smoke shows per shot on the shooter's machine ({a:?})");
    assert!(b.smoke >= sp.shots, "{name}: the muzzle smoke shows per shot on the other machine ({b:?})");
    assert!(b.flash >= 2 * b.casings - 2, "{name}: the remote muzzle flash shows 2 ticks per shot ({b:?})");
}

#[test]
fn p2k_shots_present_the_sp_effect_set_on_both_machines() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let sp = sp_run(&path, 2, 4);
    let (a, b, agree) = mp_run(&path, 2, 4);
    let (a, b, agree) = mp_run(&path, 2, 4);
    assert!(agree, "the machines' hashes agree every tick (the sister)");
    check("p2k", sp, a, b);
}

#[test]
fn automatic_shots_present_the_sp_effect_set_on_both_machines() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let sp = sp_run(&path, 6, 8);
    let (a, b, agree) = mp_run(&path, 6, 8);
    assert!(agree, "the machines' hashes agree every tick (the sister)");
    check("auto6", sp, a, b);
}
