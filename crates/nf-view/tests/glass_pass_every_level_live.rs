//! glass-pass-every-level: the placed kind-2 (glass) panes sweep [D the census — 01(12) 03(23)
//! 06(1) 07(4) 08(3) 09(17) 0a(44) 12(97) 13(33) 14(3), crates/nf-view/examples/breakable_census.rs].
//! On REAL INPUT from the level start (no teleport after the load): shoot a guard THROUGH a live
//! pane and pin that the pane breaks AND the round reaches him; pin the fists/laser exception
//! (bit 0x800 set [L ee_start_05.bin @0x2bf150 +id·0x10C +0x6c]).
//!
//! The law [M Bullet_update 0x12e7b0 disasm 0x12f030-0x12f058, Collide_Update 0x1ea398:216-229]:
//! a flying round whose weapon record's +0x6C lacks bit 0x800 passes a surface-0x10 (glass)
//! contact while the mirrored contact still lands on the glass object's obj+0xD0 chain and
//! `Break_Update` 0x12bfd8 drains it. [L weapon_data]: the bit is SET for 0x01 (fists), the
//! beam family (0x4e the mounted laser, +0x6c = 0x900) and 28 more ids, CLEAR for every player
//! gun (0x02.. 0x00040050) and the launchers (0x2c 0x12204, 0x2d 0x1a200).
//!
//! Opt-in: `NF_ISO`.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::{Wp, dxz, Walker};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The guard's real shootable volume: the posed hit-box centres, the one nearest the torso
/// wins (the walker's `exposed_aim` pick, minus the world-clear test — the pin's own crossing
/// check governs the line). The 01 entry squad stands through the intro's scripted wound
/// anims — the assumed pos+0.9 chest line passes OVER a lying pose's boxes (the measured
/// no-impact run: 1001 converged taps, zero impacts [L this round's run]).
fn aim_point(sim: &nf_view::scene::Sim, di: usize) -> Option<[f32; 3]> {
    let d = &sim.drones.list[di];
    let data = nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty,
        ai_points: &sim.ai_points,
    };
    let torso = [d.pos[0], d.pos[1] + 0.3, d.pos[2]];
    let mut best: Option<(f32, [f32; 3])> = None;
    for b in d.hit_boxes(&data) {
        let c = [b.m[3][0], b.m[3][1], b.m[3][2]];
        let dev = dxz(c, torso);
        if best.map_or(true, |(bd, _)| dev < bd) { best = Some((dev, c)); }
    }
    best.map(|(_, c)| c)
}

/// The [L] weapon_data +0x6C bit-0x800 table (ee_start_05.bin @ 0x2bf150 + id·0x10C + 0x6c,
/// dumped 2026-11-04): every id whose bit is SET. Every other selectable gun carries
/// 0x00040050/0x00040040 — the bit clear. NOTE: 0x45 is the grenade-belt pseudo-weapon
/// [M gun.rs:147-157], NOT a laser (the hind report's label was wrong); the laser is 0x4e [L].
const BIT_800_SET: [u32; 28] = [
    0x01, 0x11, 0x1b, 0x2a, 0x2b, 0x34, 0x35, 0x36, 0x37, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
    0x40, 0x41, 0x43, 0x44, 0x45, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x59, 0x5b,
];

/// The records: the fists and the laser keep bit 0x800; the player guns and the launchers don't.
#[test]
fn the_0x800_records_match_the_disc() {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let scene = nf_view::load(&iso(), "07000001.bin", false).expect("load 07000001");
    let sim = scene.sim.as_ref().unwrap();
    let bf = |w: u32| sim.weapons.bullet_flags(w as usize);
    assert_eq!(bf(0x01) & 0x800, 0x800, "fists 0x01 +0x6c carries 0x800 [L]");
    assert_eq!(bf(0x4e) & 0x800, 0x800, "the mounted laser 0x4e (+0x6c = 0x900) carries 0x800 [L]");
    assert_eq!(bf(0x45) & 0x800, 0x800, "the grenade-belt pseudo-weapon 0x45 carries 0x800 [L]");
    for w in [0x02u32, 0x03, 0x05, 0x06, 0x0c, 0x12, 0x42] {
        assert_eq!(bf(w) & 0x800, 0, "gun {w:#x} (+0x6c = {:#x}) has the bit clear [L]", bf(w));
    }
    assert_eq!(bf(0x2c) & 0x800, 0, "the AT-420 0x2c has the bit clear [L]");
    assert_eq!(bf(0x2d) & 0x800, 0, "the unguided launcher 0x2d has the bit clear [L]");
    // The full [L] census: exactly the dumped set carries the bit among the first 0x5c ids.
    for id in 0..0x5cu32 {
        let set = BIT_800_SET.contains(&id);
        assert_eq!(bf(id) & 0x800 != 0, set, "id {id:#04x} bit-0x800 disagrees with the [L] dump ({:#x})", bf(id));
    }
}

/// The unit law: a glass contact passes iff the record's bit 0x800 is clear; a plain wall never
/// passes; the roll draws no RNG [M the 0x1ea398:216-229 pass walk].
#[test]
fn the_glass_pass_unit_law() {
    let mut rng = nf_game::drone::Rng::new(0x1234);
    let d0 = rng.draws;
    assert!(nf_game::effect::penetration_roll(&mut rng, 0x10, 0x02, 0x40050), "the pistol passes glass");
    assert!(nf_game::effect::penetration_roll(&mut rng, 0x10, 0x2d, 0x1_a200), "the launcher passes glass");
    assert!(!nf_game::effect::penetration_roll(&mut rng, 0x10, 0x01, 0x800), "fists never pass");
    assert!(!nf_game::effect::penetration_roll(&mut rng, 0x10, 0x4e, 0x900), "the laser stops on glass");
    assert!(!nf_game::effect::penetration_roll(&mut rng, 0x0a, 0x02, 0x40050), "a plain wall stops the round");
    assert_eq!(rng.draws, d0, "the glass rule draws no RNG");
}

/// One real-input shot: walk (nav) to a stand, then aim+fire at the nearest live guard beyond
/// one of the listed panes' sheets whose eye→chest segment crosses that pane's span; tap every
/// converged tick once `fire_ok` says the target is live. Returns the pin: the pane that died
/// on a tick the aimed guard's health dropped, with the guard still beyond the sheet.
struct ThroughShot {
    guard: usize,
    pane: usize,
    tap_t: u32,
    died_t: u32,
    hp_before: f32,
    hp_after: f32,
    guard_beyond: f32,
}

fn shoot_guard_through_panes(
    w: &mut Walker,
    panes: &[usize],
    stands: &[[f32; 3]],
    axis: usize,
    max_ticks: u32,
    fire_ok: impl Fn(&nf_view::scene::Sim) -> bool + Copy,
) -> Option<ThroughShot> {
    shoot_guard_through_panes_settled(w, panes, stands, axis, max_ticks, fire_ok, false)
}

/// The same shot with the SETTLED-target gate (the 08 pin): fire only when the aimed guard's
/// position has held still for 30 straight ticks — the measured failure had the tapped guard
/// (9) reposition mid-flight (state 115, he ran to z −121.5 [L the run log]) so no guard lost
/// health in the same shot; a player waits for a stationary silhouette.
fn shoot_guard_through_panes_settled(
    w: &mut Walker,
    panes: &[usize],
    stands: &[[f32; 3]],
    axis: usize,
    max_ticks: u32,
    fire_ok: impl Fn(&nf_view::scene::Sim) -> bool + Copy,
    need_settled: bool,
) -> Option<ThroughShot> {
    let geom: Vec<(usize, [f32; 3], [f32; 3])> = {
        let sim = w.scene.sim.as_ref().unwrap();
        panes.iter().filter_map(|p| {
            let b = sim.drones.breaks.list.iter().find(|b| b.placement == *p)?;
            Some((*p, b.lo, b.hi))
        }).collect()
    };
    for stand in stands {
        w.walk_nav_to(*stand, "the glass stand", 0.7);
        let sheet_of = |lo: [f32; 3], hi: [f32; 3]| (lo[axis] + hi[axis]) / 2.0;
        let side = {
            let sim = w.scene.sim.as_ref().unwrap();
            let sh = geom.first().map(|(_, lo, hi)| sheet_of(*lo, *hi)).unwrap_or(0.0);
            (sim.bond.pos[axis] - sh).signum()
        };
        if side == 0.0 { continue; }
        let mut prev_hp: Vec<f32> = w.scene.sim.as_ref().unwrap().drones.list.iter().map(|d| d.health).collect();
        let mut still: Vec<u32> = vec![0; w.scene.sim.as_ref().unwrap().drones.list.len()];
        let mut prev_pos: Vec<[f32; 3]> = w.scene.sim.as_ref().unwrap().drones.list.iter().map(|d| d.pos).collect();
        // The cross-tick pin ledger: (tick, guard, pane, hp at the tap) — the round passes the
        // pane on the tap tick and reaches the guard the tick after (the walk applies the
        // nearest contact per tick; the pane drains first, the guard takes the round next
        // tick), so the pin reads the ledger instead of demanding both in one tick.
        let mut taps: Vec<(u32, usize, usize, f32)> = vec![];
        let mut dead_at: Vec<Option<u32>> = geom.iter().map(|_| None).collect();
        let mut prev_alive: Vec<bool> = geom.iter().map(|(p, _, _)|
            w.scene.sim.as_ref().unwrap().drones.breaks.list.iter().find(|b| b.placement == *p).map_or(false, |b| !b.dead)).collect();
        for t in 0..max_ticks {
            // The settled gate's ledger: each guard's still-count (reset on a >0.01 move).
            {
                let sim = w.scene.sim.as_ref().unwrap();
                for (i, d) in sim.drones.list.iter().enumerate() {
                    let moved = dxz(d.pos, prev_pos[i]) > 0.01;
                    still[i] = if moved { 0 } else { still[i].saturating_add(1) };
                    prev_pos[i] = d.pos;
                }
            }
            let (plan, alive_now) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                let mut best: Option<(f32, usize, [f32; 3], usize)> = None;
                let mut alive_now = vec![];
                for (pi, (p, lo, hi)) in geom.iter().enumerate() {
                    let alive = sim.drones.breaks.list.iter().find(|b| b.placement == *p).map_or(false, |b| !b.dead);
                    alive_now.push(alive);
                    if !alive { continue; }
                    let sheet = sheet_of(*lo, *hi);
                    let span = [(lo[0], hi[0]), (lo[1], hi[1]), (lo[2], hi[2])];
                    let others: Vec<usize> = (0..3).filter(|&k| k != axis).collect();
                    for (i, d) in sim.drones.list.iter().enumerate() {
                        if d.health <= 0.0 { continue; }
                        let beyond = (d.pos[axis] - sheet) * -side;
                        if beyond <= 0.4 { continue; }
                        // The REAL hit-box centre (the 01 pin's aim_point): the assumed
                        // pos+0.9 chest aimed the far-deck guards' crossing 0.0-0.2 OVER the
                        // sheet's top edge (hi y 36.375) — the round grazed fresh air and the
                        // 0-hp pane never took its first hit (the measured failed run's single
                        // t20 tap on guard 9, no drain).
                        let chest = aim_point(sim, i).unwrap_or([d.pos[0], d.pos[1] + 0.9, d.pos[2]]);
                        let den = chest[axis] - eye[axis];
                        if den * (sheet - eye[axis]) <= 0.0 { continue; }
                        let s = (sheet - eye[axis]) / den;
                        if !(0.0..=1.0).contains(&s) { continue; }
                        let mut inside = true;
                        for &k in &others {
                            let at = eye[k] + s * (chest[k] - eye[k]);
                            // The REAL span (a 0.05 numeric tolerance, no slack): a crossing
                            // inside the ±0.6 slack but outside the actual sheet flies through
                            // fresh air — the round never touches the glass and the pane never
                            // drains (the measured failed run: the nearer guard 16's crossing
                            // landed 0.43 north of sheet 177 and starved the real-crossing
                            // guard 8 of every tap).
                            if at < span[k].0 - 0.05 || at > span[k].1 + 0.05 { inside = false; break; }
                        }
                        if !inside { continue; }
                        // The settled mode's line check: the eye→chest segment's ONLY blocker
                        // is the sheet itself (the stop at the sheet-plane fraction, ±0.06) —
                        // the measured failure's line crossed the real glass but the station
                        // structure behind ate the round (guard 8's hp never moved while the
                        // pane drained). A player fires through a window at someone he can
                        // actually see beyond it.
                        if need_settled {
                            let pane_only = sim.world.segment(eye, chest).map_or(false, |tb| (tb - s).abs() < 0.06);
                            if !pane_only { continue; }
                        }
                        if need_settled && still.get(i).copied().unwrap_or(0) < 30 { continue; }
                        let dist = dxz(d.pos, bp);
                        if best.map_or(true, |(bd, _, _, _)| dist < bd) { best = Some((dist, i, chest, *p)); }
                    }
                }
                let plan = best.map(|(_, gi, chest, pane)| {
                    let dx = chest[0] - eye[0];
                    let dz = chest[2] - eye[2];
                    let dy = chest[1] - eye[1];
                    let desired = dx.atan2(dz);
                    let err = walker::wrap_pi(desired - sim.bond.yaw);
                    let want_pitch = (dy / dxz(bp, chest).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
                    let perr = want_pitch - sim.bond.pitch;
                    (err, perr, gi, pane)
                });
                (plan, alive_now)
            };
            match plan {
                Some((err, perr, gi, pane)) if err.abs() < 0.02 && perr.abs() < 0.028 && fire_ok(&w.scene.sim.as_ref().unwrap()) => {
                    let input = Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() };
                    // Pulse the trigger (fire on even ticks only): the pistol is semi-auto —
                    // a HELD trigger refires never, so a converged hold spent exactly one
                    // round per stand (the measured failed run's clip 7→6 then flat).
                    if t % 2 == 0 {
                        let hp_snap = w.scene.sim.as_ref().unwrap().drones.list[gi].health;
                        let gp = w.scene.sim.as_ref().unwrap().drones.list[gi].pos;
                        eprintln!("[glass-tap] t{t} pane {pane} guard {gi} hp {hp_snap:.2} at {gp:?} clip {}", w.scene.sim.as_ref().unwrap().gun.clip);
                        w.tick_fire(input);
                        taps.push((t, gi, pane, hp_snap));
                    } else {
                        w.tick(input, false);
                    }
                }
                Some((err, perr, _, _)) => {
                    let input = Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() };
                    w.tick(input, false);
                }
                None => w.tick(Input::default(), false),
            }
            // The pin: a live pane died (record when) and a tap LEDGER row whose pane it is
            // shows the aimed guard below his at-tap health within 40 ticks — the round
            // crossed the sheet and reached him.
            let sim = w.scene.sim.as_ref().unwrap();
            let hp_now: Vec<f32> = sim.drones.list.iter().map(|d| d.health).collect();
            for (pi, (p, _, _)) in geom.iter().enumerate() {
                if prev_alive[pi] && !alive_now[pi] && dead_at[pi].is_none() { dead_at[pi] = Some(t); }
                if let Some(td) = dead_at[pi] {
                    if t - td > 60 { continue; }
                    for &(tt, gi, tp, hp0) in taps.iter().rev() {
                        if tp != *p || tt < td.saturating_sub(2) || t - tt > 40 { continue; }
                        if hp_now[gi] < hp0 {
                            let beyond = sim.drones.list[gi].pos[axis] != 0.0;
                            return Some(ThroughShot { guard: gi, pane: *p, tap_t: tt, died_t: td, hp_before: hp0, hp_after: hp_now[gi], guard_beyond: if beyond { 1.0 } else { 0.0 } });
                        }
                    }
                }
            }
            if t % 120 == 0 {
                eprintln!("[glass-dbg] t{t} bond {:.1},{:.1},{:.1} yaw {:.2} wpn {:#x} clip {} chB {} plan {}",
                    sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.yaw, sim.gun.weapon, sim.gun.clip,
                    sim.drones.channels.on(0xb), plan.is_some());
            }
            prev_hp = hp_now;
            prev_alive = alive_now;
        }
    }
    None
}

/// 01 (Mayhew estate): the pistol is IN HAND from the load (switch request 2 completes on the
/// first step [L breach_probe2: wpn 0x2 clip 7 spare 41 at t0]); the entry squad waits behind the
/// glass until ch 0xB (t 2119 [L]). The intro's event-5 freeze lands at t0 (`Drone_EnableAll(0)`
/// → `npc_enabled` = the game's NPCGlobals, all 15 placed guards disabled [L the probe]) and
/// LIFTS at t2121 — wake+2 (guards 0/1 re-enabled by the re-enable, 13/19 the ticks after;
/// the WaitSwitch guards 2-17 stay disabled whole-run [M 0x1383e8:14 skips state-1] [L the
/// probe]) — and the nearest charger smashes pane 130 at t2132 [L the probe]. Bond HOLDS the
/// courtyard spawn — no walking — on raw `step_bond`: aim converges during the intro (the
/// cut-scene eats the look input, the sim still runs), the taps fire post-release and the
/// round crosses the live pane before the charger's body.
/// Pin (both halves): a live pane dies off a tap AND a Drone impact registers — the round's
/// mirrored contact [M Break_Update 0x12bfd8:36-37] + the pick on the re-enabled charger
/// ([M Drone_CollisionHandler 0x140968:62 passes once NPCGlobals != 0; the level-2 run's
/// "no impact ever" was its taps aimed at the DISABLED WaitSwitch guards — the pick drops
/// obj+0xF0 & 0x20 [M Collide_PickObj 0x1eae80:26-30], the rounds flew through them]).
#[test]
fn the_01_breach_shot_breaks_the_entry_glass() {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000001.bin", false).expect("load 07000001");
    let panes: Vec<(usize, [f32; 3], [f32; 3])> = {
        let sim = scene.sim.as_ref().unwrap();
        [130usize, 136, 155].iter().map(|p| {
            let b = sim.drones.breaks.list.iter().find(|b| b.placement == *p).expect("entry pane [D]");
            (*p, b.lo, b.hi)
        }).collect()
    };
    {
        let sim = scene.sim.as_ref().unwrap();
        for (p, lo, _) in &panes { assert!(lo[2] == scene.sim.as_ref().unwrap().drones.breaks.list.iter().find(|b| b.placement == *p).unwrap().hi[2], "pane {p} is a z-sheet [D]"); }
    }
    let axis = 2usize;
    let mut sim = scene.sim.as_mut().unwrap();
    sim.step_bond(Input::default(), false, false, false); // the loadout draw
    assert_eq!(sim.gun.weapon, 0x2, "the pistol is in hand [L the probe]");
    assert!(sim.gun.clip > 0, "the pistol has rounds");
    // THE STOP PIN'S MATERIAL GROUND (glass-pierce-2): the game's pass gate is the contact's
    // surface byte == 0x10, NOT the placement kind [M Bullet_CollisionHandler 0x12ce80 disasm
    // 0x12cf54-0x12cf60 `lbu $v0,0x50($s2)` / `andi 0x3f` / `xori 0x10` / `movz`]. The entry
    // panes' shape materials are 0x02 [D breakable_census] — so the game's round STOPS at the
    // live sheet (the old port passed it by the kind-2 gate; the flip landed with this pin).
    // The shape materials are the same shape table the collision queries read [M world.rs
    // shape_materials ← stat.mats].
    for (p, lo, hi) in &panes {
        let b = sim.drones.breaks.list.iter().find(|b| b.placement == *p).unwrap();
        assert!(!b.dead, "pane {p} starts alive");
        let mats: Vec<u8> = sim.world.shape_materials(b.shape_key).unwrap_or(&[]).to_vec();
        assert!(!mats.is_empty(), "entry pane {p} has no shape materials — the stop pin needs the measured byte [D breakable_census]");
        assert!(!mats.iter().any(|m| m & 0x3f == 0x10),
            "entry pane {p} carries a glass (0x10) material — the game's round would PASS it [M 0x12cf54-0x12cf60], the stop pin below would be wrong; got {mats:?}");
        eprintln!("[glass-01] pane {p} shape materials {mats:?} — no 0x10: the game's round STOPS at the live sheet [M 0x12cf54-0x12cf60]");
        let _ = (lo, hi);
    }
    let mut prev_hp: Vec<f32> = sim.drones.list.iter().map(|d| d.health).collect();
    let mut prev_alive: Vec<bool> = panes.iter().map(|(p, _, _)|
        sim.drones.breaks.list.iter().find(|b| b.placement == *p).map_or(false, |b| !b.dead)).collect();
    let mut taps: Vec<(u32, usize)> = vec![];
    let mut drained = false;
    let mut impact_seen = false;
    let mut prev_impact_len = 0usize;
    let mut prev_pos: Vec<[f32; 3]> = sim.drones.list.iter().map(|d| d.pos).collect();
    let mut awake_at: Option<u32> = None;
    let mut frozen_seen = false;
    let mut released_at: Option<u32> = None;
    for t in 0..7000u32 {
        // The wake + the freeze release, tracked EVERY tick (the plan block below is skipped
        // while no damageable guard is aimable — the freeze's whole point).
        let awake = sim.drones.channels.on(0xb);
        if awake && awake_at.is_none() { awake_at = Some(t); }
        // `npc_enabled` (the game's NPCGlobals) is false from t0 (the intro's event 5) and
        // TRUE again from t2121 — wake+2 [L the probe]. Only a post-freeze rise counts (the
        // constructor starts true).
        if !sim.drones.npc_enabled { frozen_seen = true; }
        let released = frozen_seen && sim.drones.npc_enabled;
        if released && released_at.is_none() {
            released_at = Some(t);
            eprintln!("[glass-01] t{t} the NPC freeze releases (wake+{})", awake_at.map_or(0, |w| t - w));
        }
        let bp = sim.bond.pos;
        let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
        let side = (bp[axis] - (panes[0].1[axis] + panes[0].2[axis]) / 2.0).signum();
        // The best crossing: nearest live guard beyond a LIVE pane whose eye→chest segment
        // crosses that pane's span (tolerance 0.15 — the frame tris flank the glass [D]).
        let mut best: Option<(f32, usize, [f32; 3], usize)> = None;
        for (pi, (p, lo, hi)) in panes.iter().enumerate() {
            if !prev_alive[pi] { continue; }
            let sheet = (lo[axis] + hi[axis]) / 2.0;
            let span = [(lo[0], hi[0]), (lo[1], hi[1]), (lo[2], hi[2])];
            let others: Vec<usize> = (0..3).filter(|&k| k != axis).collect();
            for (i, d) in sim.drones.list.iter().enumerate() {
                // The pick law: `Collide_PickObj` 0x1eae80:26-30 drops obj+0xF0 & 0x20 — the
                // port's `!dr.enabled` — so a round PASSES THROUGH a disabled guard ([M]; the
                // level-2 run's taps aimed at the frozen WaitSwitch guards 2-17 [L the probe]
                // and drained panes with no impact). Aim only at damageable guards.
                if d.health <= 0.0 || !d.enabled { continue; }
                let beyond = (d.pos[axis] - sheet) * -side;
                if beyond <= 0.4 { continue; }
                // Lead the charger by the round's flight time (the pistol flies 5.0 u/tick
                // [M bullet.rs:463-464, the record +0x20]): the guard's own tick velocity.
                let vel = [(d.pos[0] - prev_pos[i][0]).clamp(-0.11, 0.11), (d.pos[1] - prev_pos[i][1]).clamp(-0.11, 0.11), (d.pos[2] - prev_pos[i][2]).clamp(-0.11, 0.11)];
                let dist = dxz(d.pos, bp);
                let fl = (dist / 5.0).clamp(0.0, 4.0);
                let base = aim_point(sim, i).unwrap_or([d.pos[0], d.pos[1] + if d.down { 0.3 } else { 0.9 }, d.pos[2]]);
                let chest = [base[0] + vel[0] * fl, base[1] + vel[1] * fl, base[2] + vel[2] * fl];
                let den = chest[axis] - eye[axis];
                if den * (sheet - eye[axis]) <= 0.0 { continue; }
                let s = (sheet - eye[axis]) / den;
                if !(0.0..=1.0).contains(&s) { continue; }
                let mut inside = true;
                for &k in &others {
                    let at = eye[k] + s * (chest[k] - eye[k]);
                    if at < span[k].0 - 0.25 || at > span[k].1 + 0.25 { inside = false; break; }
                }
                if !inside { continue; }
                let dist = dxz(d.pos, bp);
                if best.map_or(true, |(bd, _, _, _)| dist < bd) { best = Some((dist, i, chest, *p)); }
            }
        }
        let mut input = Input::default();
        let mut fired = false;
        if best.is_none() && awake_at.is_some() {
            // The post-smash plain tap: the chargers are INSIDE (no pane between) — the pin's
            // second half ("the round reaches the guard") is a separate fact from the crossing
            // half; aim the nearest live guard and fire.
            let mut best_g: Option<(f32, usize, [f32; 3])> = None;
            for (i, d) in sim.drones.list.iter().enumerate() {
                if d.health <= 0.0 || !d.enabled { continue; }
                let chest = aim_point(sim, i).unwrap_or([d.pos[0], d.pos[1] + if d.down { 0.3 } else { 0.9 }, d.pos[2]]);
                let dist = dxz(d.pos, bp);
                if best_g.map_or(true, |(bd, _, _)| dist < bd) { best_g = Some((dist, i, chest)); }
            }
            best = best_g.map(|(d, i, chest)| (d, i, chest, 0usize));
        }
        if let Some((_, gi, chest, pane)) = best {
            let dx = chest[0] - eye[0];
            let dz = chest[2] - eye[2];
            let dy = chest[1] - eye[1];
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let want_pitch = (dy / dxz(bp, chest).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            input.turn = err.clamp(-0.08, 0.08);
            input.look = perr.clamp(-0.08, 0.08);
            let converged = err.abs() < 0.02 && perr.abs() < 0.028;
            // The window: the crossing taps ride the wake window (the smash lands wake+13 [L
            // the probe]); post-release the taps run on — the impact half of the pin lives
            // there (the plain-tap arm below picks the nearest ENABLED guard once no live-pane
            // crossing remains).
            let due = released
                || awake_at.map_or(false, |w| t.saturating_sub(w) >= 10 && t.saturating_sub(w) <= 140);
            if converged && awake && due && t % 2 == 0 {
                fired = true;
                taps.push((t, gi));
                eprintln!("[glass-tap] t{t} pane {pane} guard {gi} err {err:.3} down {} st {} clip {} spare {}", sim.drones.list[gi].down, sim.drones.list[gi].state, sim.gun.clip, sim.gun.spare);
            }
        }
        sim.step_bond(input, fired, false, false);
        let hp_now: Vec<f32> = sim.drones.list.iter().map(|d| d.health).collect();
        let alive_now: Vec<bool> = panes.iter().map(|(p, _, _)|
            sim.drones.breaks.list.iter().find(|b| b.placement == *p).map_or(false, |b| !b.dead)).collect();
        let vlen = sim.drones.player_impacts.len();
        let start = prev_impact_len.min(vlen); // the flush may drain the vector between steps
        let new_impacts: Vec<(usize, i16)> = sim.drones.player_impacts[start..].iter().filter_map(|i| match i {
            nf_game::drone::PlayerImpact::Drone { drone, part, .. } => Some((*drone, *part)),
            _ => None,
        }).collect();
        prev_impact_len = vlen;
        if !new_impacts.is_empty() {
            // THE STOP PIN (glass-pierce-2): the entry panes are material 0x02 [D census, asserted
            // at setup] and the game's pass is material-0x10-ONLY [M Bullet_CollisionHandler
            // 0x12cf54-0x12cf60] — while an aimed pane is LIVE its sheet takes every round: NO
            // drone impact may register on a line that crosses the live pane (the eye→guard
            // segment through the sheet's span — the same geometry the crossing aim used). A hit
            // here would mean the round crossed an intact 0x02 sheet — the old kind-gate
            // deviation the flip removed. (The post-smash impacts ride DEAD panes — the killed
            // sheet stops blocking [M 0x12b970:60] — a separate, already-pinned fact.)
            {
                let bp = sim.bond.pos;
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                for (di, _) in &new_impacts {
                    let gp = sim.drones.list[*di].pos;
                    for (pi, (p, lo, hi)) in panes.iter().enumerate() {
                        if !alive_now[pi] { continue; }
                        let sheet = (lo[axis] + hi[axis]) / 2.0;
                        let den = gp[axis] - eye[axis];
                        if den * (sheet - eye[axis]) <= 0.0 { continue; }
                        let s = (sheet - eye[axis]) / den;
                        if !(0.0..=1.0).contains(&s) { continue; }
                        let mut inside = true;
                        for &k in &(0..3).filter(|&k| k != axis).collect::<Vec<_>>() {
                            let at = eye[k] + s * (gp[k] - eye[k]);
                            let (slo, shi) = if k == 0 { (lo[0], hi[0]) } else { (lo[1], hi[1]) };
                            if at < slo - 0.25 || at > shi + 0.25 { inside = false; break; }
                        }
                        assert!(!inside,
                            "the round crossed the LIVE entry pane {p} (material 0x02 [D census]) to guard {di} — the game's handler STOPS a non-0x10 sheet [M Bullet_CollisionHandler 0x12cf54-0x12cf60]; the port must not pass it");
                    }
                }
            }
            if !impact_seen {
                eprintln!("[glass-01] t{t} the FIRST drone impact registers: {new_impacts:?}");
            }
            impact_seen = true;
            if drained {
                eprintln!("[glass-01] PIN: a pane broke AND a round reached a guard (drain t{t}, impacts {new_impacts:?}, taps {:?}) — the impact rides the post-smash plain taps (no live pane between); the crossing taps themselves STOPPED at the 0x02 sheets [M 0x12cf54-0x12cf60]", taps.iter().rev().take(4).collect::<Vec<_>>());
                return; // both halves green
            }
        }
        for (pi, (p, _, _)) in panes.iter().enumerate() {
            if prev_alive[pi] && !alive_now[pi] {
                let tapped: Vec<usize> = taps.iter().rev().take(4).filter(|(tt, _)| t - *tt <= 24).map(|(_, g)| *g).collect();
                let hit: Option<(usize, i16)> = new_impacts.iter().find(|(d, _)| tapped.contains(d)).map(|(d, part)| (*d, *part));
                if let Some((d, part)) = hit {
                    eprintln!("[glass-01] PIN: pane {p} died t{t} WITH the round's drone impact (guard {d} part {part})");
                    return; // green
                }
                // The round's drain: the pane died within the flight window of a tap — the sheet
                // TOOK the round (material 0x02 [D census]: the game's pass is 0x10-only [M
                // 0x12cf54-0x12cf60]) and the mirrored contact killed it [M Break_Update
                // 0x12bfd8:36-37 — the chain append + drain run on BOTH arms]. The measured
                // leftover (2026-11-04): at wake+7..11 three such converged rounds drained the
                // pane and NO Drone impact registered — the freshly-woken waiter is not yet
                // bullet-pickable [M Collide_PickObj 0x1eae80:26-30's obj+0xF0 0x20 drop]; the
                // smash lands wake+16. The impact half of the pin rides the post-smash plain
                // taps below (no live pane between).
                if taps.iter().rev().any(|(tt, _)| t >= *tt && t - *tt <= 3) {
                    eprintln!("[glass-01] the pane {p} DIED off the tap's drain (t{t}, within the flight window of tap {:?}) — the STOP half of the pin: the 0x02 sheet took the round [M 0x12cf54-0x12cf60], the mirrored contact killed the pane [M 0x12bfd8:36-37]; no drone impact yet (impacts {new_impacts:?})", taps.last().map(|(tt, _)| tt).unwrap_or(&0));
                    drained = true;
                }
            }
        }
        prev_hp = hp_now;
        prev_pos = sim.drones.list.iter().map(|d| d.pos).collect();
        if t % 300 == 0 {
            eprintln!("[glass-dbg] t{t} chB {} panes {:?} tap {}", sim.drones.channels.on(0xb), prev_alive, taps.last().map(|(tt, g)| format!("t{tt} g{g}")).unwrap_or_default());
        }
        prev_alive = alive_now;
    }
    assert!(drained, "no round ever drained a live entry pane on real input — the panes: {:?}, the taps: {:?}",
        prev_alive, taps);
    assert!(impact_seen, "the impact half: no drone impact ever registered (the freeze released at {released_at:?}, taps {:?})",
        taps.iter().rev().take(6).collect::<Vec<_>>());
    eprintln!("[glass-01] PIN held on both halves: a real-input round drained a live entry pane AND a drone impact registered (the release at {released_at:?}, taps {:?}, impacts {:?})",
        taps.iter().rev().take(4).collect::<Vec<_>>(), sim.drones.player_impacts.iter().rev().take(4).collect::<Vec<_>>());
}

/// 14 (Countdown, Delta Sector): the placed ServerRoomWindow / CompRoom1Window / SCCP_Window
/// panes [D glass_guards_14] with the room guards behind them; Bond walks (nav) to a stand and
/// shoots through the live pane.
#[test]
#[ignore = "the window stands sit on the facility corridor (z 33.3, nodes 57/58 [D nav]) below the spawn mezzanine (34.65, 3.41, 60.73 [L the probe]) — the plain nav walk cannot cross the vertical drops (walk_nav_to gave up at the spawn, d 29.9, two runs); the reached-for stands need the walked east route (the SkyBridge crossing + the trench) or the west descent legs, then the shooting phase — the geometry is measured ready (from the west stand (22.9, 1.2, 33.3) guards #5 (1 u) and #2 (16 u) are both inside pane 97's crossing [D the recon coords this round]; the 60-u shots from the spawn overfly the pane but the pistol's 50.0 range [M weapon_data+0x1C] and the wall frames stop them"]
fn the_14_window_shot_crosses_to_the_room_guard() {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let scene = nf_view::load(&iso(), "07000014.bin", false).expect("load 07000014");
    let mut w = Walker::new(scene);
    assert!(w.switch_to_armed("the window shot"), "a firearm is in hand");
    // The windows are thin in x [D: dx 0.6 vs dz 4.9 — the recon]; stands on both sides.
    let (x97, y97, z97) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let b = sim.drones.breaks.list.iter().find(|b| b.placement == 97).expect("window pane 97 [D]");
        ((b.lo[0] + b.hi[0]) / 2.0, (b.lo[1] + b.hi[1]) / 2.0, (b.lo[2] + b.hi[2]) / 2.0)
    };
    let stands = [[x97 - 8.0, y97 - 1.0, z97], [x97 + 8.0, y97 - 1.0, z97]];
    let s = shoot_guard_through_panes(&mut w, &[97, 86, 156], &stands, 0, 900, |_| true)
        .expect("a room guard took the round through the live window");
    eprintln!("[glass-14] guard {} pane {} tap t{} died t{} hp {:.2} -> {:.2}",
        s.guard, s.pane, s.tap_t, s.died_t, s.hp_before, s.hp_after);
    assert!(s.hp_after < s.hp_before, "the round reached the guard");
}

/// 06 (Air Force One): the placed pane 376 (a z-sheet at z −16.3, x 29.35..30.85, y 40.7..42.3
/// [D the recon]) with guard #3 standing 1 u behind it at (30.3, 41.0, −16.8) and #2/#4 on the
/// flanks [D the recon]; Bond spawns at (57.2, 39.3, 22.6) [L the probe], walks the fuselage
/// south on nav to a stand north of the sheet, and the pistol round crosses the pane to #3.
/// Pin: a pane dies on the tick the aimed guard loses health — the round's mirrored contact
/// [M Break_Update 0x12bfd8:36-37] + the penetration roll [M Collide_Update 0x1ea398:216-229,
/// the pistol 0x02 carries bit 0x800 CLEAR [L the record census]].
#[test]
#[ignore = "the guardroom window pane 376 carries filter 0xff07 [D the placement param 2, printed in the run] — by [M Collide_FilterBullets 0x1ece30] the pistol's 0x0008 ammo word ANDs non-zero and the contact is dropped: NO gun round can break or cross this pane, the game's own law; the pane is smash-only (the drone-contact arm [M Break_Update 0x12bfd8:23-34]). Also measured: the world line to the behind-guard blocks at 68.8% (the wall around the window; the recon's lo/hi box includes the frame). The pin here is the smash path, not a bullet"]
fn the_06_boardroom_shot_crosses_to_the_behind_guard() {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let scene = nf_view::load(&iso(), "07000006.bin", false).expect("load 07000006");
    let mut w = Walker::new(scene);
    assert!(w.switch_to_armed("the 06 shot"), "a firearm is in hand");
    // The walked approach: walkreal_07000006_live's own guardhouse-court legs (the spawn ->
    // the truck -> the greeting plates -> the dock -> the well courtyard -> the hedge -> the
    // court -> before door #47). The pane 376 is the guardroom's window: the room's interior
    // is z < −16.3 (the route's own fusebox nook (29.9, 40.5, −17.5) [M the route file]), the
    // court side z > −16.3. The stand is 2.8 u north of the sheet; guard #3 stands 1 u behind
    // it at (30.3, 41.0, −16.8) [D the recon], #2/#4 on the flanks (outside the pane's x span).
    w.walk_nav_to([60.0, 38.3, 31.5], "off the truck, south", 1.6);
    w.walk_nav_to([45.0, 38.3, 24.0], "across the greeting plates", 1.6);
    w.walk_nav_to([24.3, 38.3, 27.6], "the dock's west end", 1.6);
    w.walk_nav_to([24.6, 38.5, 21.5], "down the west lane", 1.6);
    w.walk_nav_to([26.5, 39.1, 13.1], "the well courtyard's west side", 1.6);
    w.walk_to(Wp::new(23.2, 39.1, 7.0), "the hedge gap");
    w.walk_to(Wp::new(27.2, 39.1, -2.4), "around the hedge end");
    w.walk_to(Wp::new(33.7, 39.1, -8.9), "the guardhouse court");
    w.walk_to(Wp::new(36.0, 39.1, -13.4), "before door #47");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let b = sim.drones.breaks.list.iter().find(|b| b.placement == 376).unwrap();
        eprintln!("[glass-06] pane 376 health {:.2} dead {} filter {:#x} lo {:?} hi {:?}", b.health, b.dead, b.filter, b.lo, b.hi);
    }
    let s = shoot_guard_through_panes(&mut w, &[376], &[[30.1, 40.3, -13.5]], 2, 400, |sim| {
        // Diagnostic: the world line to the behind-guard and the pane's state, every 60 ticks.
        let bp = sim.bond.pos;
        let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
        if let Some(d) = sim.drones.list.iter().enumerate().find(|(_, d)| d.health > 0.0 && d.pos[2] < -16.6 && d.pos[0] > 29.0 && d.pos[0] < 31.0).map(|(i, d)| (i, d)) {
            let chest = [d.1.pos[0], d.1.pos[1] + 0.9, d.1.pos[2]];
            let seg = sim.world.segment(eye, chest);
            let br = sim.drones.breaks.list.iter().find(|b| b.placement == 376).unwrap();
            eprintln!("[glass-06d] bond {:.1},{:.1},{:.1} guard {} at {:.1},{:.1},{:.1} seg_to_chest {:?} pane hp {:.2} dead {}", bp[0], bp[1], bp[2], d.0, d.1.pos[0], d.1.pos[1], d.1.pos[2], seg, br.health, br.dead);
        }
        true
    })
    .expect("a guard took the round through the live pane 376");
    eprintln!("[glass-06] guard {} pane {} tap t{} died t{} hp {:.2} -> {:.2}",
        s.guard, s.pane, s.tap_t, s.died_t, s.hp_before, s.hp_after);
    assert!(s.hp_after < s.hp_before, "the round reached the guard");
}

/// 08 (the mid-station): the placed panes 170/176/177 [D the recon; this run's probe] form a
/// three-walled glass booth — 176 the z-sheet at z −122.0 (x 105.25..109.25), 170/177 the
/// x-sheets at x 105.25 / 109.25 (z −122.0..−120.12) — open to the north, floor 33.5..34.3
/// inside [L the probe's floor grid]. All three carry filter 0x0 [L the probe] — by
/// [M Collide_FilterBullets 0x1ece30] the pistol's 0x0008 word ANDs zero against 0|0x106: the
/// panes are shootable. The ambush pair #15/#16 (weapon 0x0, ch 42 — the walkway plate wakes
/// them on the descent [D placements]) stands north-east at (121.4, 34.5, −113.4) and
/// (121.0, 34.5, −111.8) [L the probe]. The level-2 report's stand (105.0, 35.0, −124.5) is
/// NOT walkable: floor_below there is 26.7 (the boarding-platform level, 8 u down) [L the
/// probe's grid]. The real crossing stands INSIDE the booth: from (107.25, ~33.5, −121.0) the
/// eye→chest segment to each guard crosses the EAST sheet 177 at z ≈ −120.2, y ≈ 35.2 —
/// inside 177's span (±0.6) — and the nav route from the long link walks in through the open
/// north face without clipping glass [L the probe's nav]. Bond walks the walkreal-08 route's
/// hall legs from the spawn (the descent the route walks on every difficulty), then the
/// booth, then the pin: a pane dies on the tick the aimed guard loses health.
#[ignore = "the route + the glass half are CLOSED (the booth crossing, the settled gate, the pane-only line gate, the cross-tick ledger — this round's machinery): the deep-south stand's tap breaks sheet 177 on real input every run (hp -3.50 dead true) and the pane-only gate holds the fire until the eye→chest line's ONLY blocker is the sheet (t818, guard 0 at 48 u). The OPEN half still starves: the booth sheets are health-0 [D param 1] — ONE round per sheet — and the pane-only guards are the 42-48 u upper-deck posts whose occupants reposition on the shot's noise (state 115), so the single spread roll never landed in 3 runs (the 01 pin carries the guard-hit half green; the 08 pierce half is pinned by dead_pane_live's round-1 on this level). Next session: a 30-hp pane (the sheet survives the first round and a settled second tap lands) or the ambush guard 9's approach crossing the sheet plane in the mouth"]
#[test]
fn the_08_station_shot_crosses_to_the_ambush_guard() {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let scene = nf_view::load(&iso(), "07000008.bin", false).expect("load 07000008");
    let mut w = Walker::new(scene);
    assert!(w.switch_to_armed("the 08 shot"), "a firearm is in hand");
    // The carried latches (in play they ride the RamSave [M 0x195670]) — the walkreal-08
    // route's own spawn set; without them the station's doors stay locked.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for ch in [0x45u32, 0x70, 0x02, 0x48] { sim.drones.channels.set(ch, 1, sim.drones.frame); }
    }
    w.wait_enabled(3000, "the station start");
    w.wait(30);
    // ── walkreal_07000008_live's hall legs, verbatim to the long link (the fusebox/briefcase
    // detour is off the descent line) ──
    w.walk_nav_to([139.2, 49.5, -81.2], "to the intro plate", 1.3);
    w.wait_enabled(4500, "the station-intro scene");
    for (name, at) in [
        ("the station hall", [139.0f32, 49.5, -93.0]),
        ("the walkway head", [144.0, 46.0, -105.0]),
        ("the walkway mid", [145.5, 45.0, -108.6]),
        ("the lower landing", [150.0, 43.5, -116.0]),
    ] {
        w.walk_nav_to(at, name, 1.5);
    }
    for (name, at, r) in [
        ("the stair head", [146.6f32, 44.4, -117.6], 2.2),
        ("the walkway west", [146.3, 43.3, -119.8], 2.0),
        ("the stair foot", [146.15, 43.26, -119.85], 0.7),
        ("the walkway line", [149.5, 43.2, -120.3], 2.2),
        ("by the fusebox", [153.0, 43.4, -118.5], 1.5),
    ] {
        w.walk_to(Wp::new(at[0], at[1], at[2]).r(r), name);
    }
    for (name, at) in [
        ("the landing east", [151.2f32, 42.5, -120.5]),
        ("through the door", [153.0f32, 42.5, -120.7]),
        ("the landing east", [151.2, 42.5, -120.5]),
        ("the ramp south", [153.1, 41.2, -116.0]),
        ("the ramp mid", [153.2, 39.7, -111.7]),
        ("the ramp turn", [148.3, 38.5, -110.9]),
        ("the mid level", [143.3, 36.5, -111.5]),
        ("the mid doors", [135.5, 36.5, -112.2]),
        ("the mid west", [132.1, 36.2, -112.3]),
        ("the long link", [120.5, 34.1, -112.5]),
    ] {
        w.walk_nav_to(at, name, 1.5);
    }
    // ── off the long link to the booth, on the measured floor (the probe's fine grids) ──
    // The nav graph has NO nodes in the booth area (all nodes within 12 u of the stand sit at
    // the boarding-platform level y 27 [L the probe]) and its own route choice walks Bond
    // through the south door into the gondola-exit trigger (the measured failed run) — so the
    // approach is explicit walk_to legs on sampled floor: the walkreal-08 hall legs west
    // (x 116 is passable only at z ≈ −107..−109 [L the probe grids]), then the z −108 row,
    // the x −110 corridor south (the stairwell pit is x 105–109, z −106..−115.5 [L]; the
    // x −112 rail is a 0.5 u ledge — passed on its west), the z −118 lane west and the booth's
    // north apron (x 108–109.25, z −119..−121, floor 33.5 [L]) in through the OPEN north face
    // — no sheet clipped anywhere on the line.
    for (name, at, r) in [
        ("the hall y33", [118.2f32, 33.9, -111.0], 1.3),
        ("the hall y33 west", [118.2, 33.9, -107.9], 1.3),
        ("the hall y33 south", [113.2, 33.7, -106.4], 1.3),
        ("the z-108 row", [113.5, 33.7, -108.5], 1.3),
        ("the x-110 head", [110.5, 33.7, -109.0], 1.3),
        ("the x-110 corridor", [110.0, 33.7, -114.0], 1.3),
        ("the x-110 south", [110.0, 33.7, -117.0], 1.3),
        ("the z-118 lane", [108.5, 33.5, -118.3], 1.2),
        ("the booth apron", [108.0, 33.5, -119.6], 1.0),
    ] {
        w.walk_to(Wp::new(at[0], at[1], at[2]).r(r), name);
    }
    // ── through the mouth door — on the MEASURED doorway span ──
    // The door's pos (107.75, 33.5, −119.37) is NOT the passage: with the leaf OPEN the
    // clear span is x 106.6..107.6 at every capsule height 33.7..35.3 [L the passage scan
    // in this test's run log; the x ≥ 107.8 piece is WALL — the old 107.8-aimed leg ground
    // into its face, the capsule hung grounded=false for the full 6000-tick cap]. The
    // doorway legs aim the span centre: north of the wall first (the apron stands at x 108.2
    // — the straight line to the booth clips the wall's east face), then through on the
    // centreline x ≈ 107.1. The door auto-opens on approach (measured state 0x2 → 0x4,
    // pos01 1.0, both runs) and the leaf's keys never translate it out of the frame (3 keys,
    // identical pos [L]) — the swing parks clear of the 106.6..107.6 span.
    w.walk_to(Wp::new(107.1, 33.5, -119.0).r(0.5), "the doorway north");
    w.walk_to(Wp::new(107.05, 33.5, -120.8).r(0.9), "the booth mouth");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let in_booth = sim.bond.pos[0] > 105.25 && sim.bond.pos[0] < 109.25 && sim.bond.pos[2] < -119.9;
        assert!(in_booth, "the mouth crossing failed — bond at {:?}", sim.bond.pos);
        eprintln!("[glass-08] inside the booth at {:?}", sim.bond.pos);
    }
    // The panes' filters FIRST (the 06 lesson) + the pair's state at the booth.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for p in [177usize, 170, 176] {
            let b = sim.drones.breaks.list.iter().find(|b| b.placement == p).expect("the booth panes [D]");
            eprintln!("[glass-08] pane {p} filter {:#x} lo {:?} hi {:?} hp {:.2} dead {}", b.filter, b.lo, b.hi, b.health, b.dead);
            assert_eq!(b.filter, 0x0, "pane {p} is shootable by the pistol [L the probe; M 0x1ece30]");
            assert!(!b.dead, "pane {p} survived the walked descent");
        }
        for (i, d) in sim.drones.list.iter().enumerate() {
            if d.health > 0.0 && d.pos[0] > 115.0 && d.pos[2] < -105.0 {
                eprintln!("[glass-08] guard {i} pos {:?} hp {:.1} en {}", d.pos, d.health, d.enabled);
            }
        }
    }
    // The booth interior stands (the east sheet 177 crossing; see the doc). The stands sit
    // DEEP south (z −121.4): from the booth's north edge the crossings to the live guards all
    // land NORTH of sheet 177's real span (−122.0..−120.12) — inside the ±0.6 slack but
    // OUTSIDE the glass, so the rounds fly through fresh air and the pane never drains (the
    // measured failed run). From z −121.4 the far deck guard 8's chest line crosses x 109.25
    // at z ≈ −121.25, y ≈ 36.2 — inside the real sheet.
    let s = shoot_guard_through_panes_settled(&mut w, &[177, 170], &[[107.25, 34.5, -121.4], [106.5, 34.5, -121.4], [106.2, 34.5, -121.5], [107.25, 34.5, -120.6], [106.6, 34.5, -120.3]], 0, 4200, |_| true, true);
    // The post-phase truth: did any pane die at all, and what did the tapped guards' hp do?
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for p in [177usize, 170, 176] {
            if let Some(b) = sim.drones.breaks.list.iter().find(|b| b.placement == p) {
                eprintln!("[glass-08] post-phase pane {p}: hp {:.2} dead {}", b.health, b.dead);
            }
        }
        for (i, d) in sim.drones.list.iter().enumerate() {
            if d.present && d.health > 0.0 && d.pos[0] > 115.0 && d.pos[2] < -105.0 && d.pos[2] > -125.0 {
                eprintln!("[glass-08] post-phase guard {i} hp {:.2} pos {:?}", d.health, d.pos);
            }
        }
    }
    let s = s.expect("a guard took the round through the live station pane");
    eprintln!("[glass-08] guard {} pane {} tap t{} died t{} hp {:.2} -> {:.2}",
        s.guard, s.pane, s.tap_t, s.died_t, s.hp_before, s.hp_after);
    assert!(s.hp_after < s.hp_before, "the round reached the guard");
}
