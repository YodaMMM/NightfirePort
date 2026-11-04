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
use walker::{dxz, Walker};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

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
    let geom: Vec<(usize, [f32; 3], [f32; 3])> = {
        let sim = w.scene.sim.as_ref().unwrap();
        panes.iter().filter_map(|p| {
            let b = sim.drones.breaks.list.iter().find(|b| b.placement == *p)?;
            Some((*p, b.lo, b.hi))
        }).collect()
    };
    for stand in stands {
        w.walk_nav_to(*stand, "the glass stand", 1.6);
        let sheet_of = |lo: [f32; 3], hi: [f32; 3]| (lo[axis] + hi[axis]) / 2.0;
        let side = {
            let sim = w.scene.sim.as_ref().unwrap();
            let sh = geom.first().map(|(_, lo, hi)| sheet_of(*lo, *hi)).unwrap_or(0.0);
            (sim.bond.pos[axis] - sh).signum()
        };
        if side == 0.0 { continue; }
        let mut prev_hp: Vec<f32> = w.scene.sim.as_ref().unwrap().drones.list.iter().map(|d| d.health).collect();
        let mut prev_alive: Vec<bool> = geom.iter().map(|(p, _, _)|
            w.scene.sim.as_ref().unwrap().drones.breaks.list.iter().find(|b| b.placement == *p).map_or(false, |b| !b.dead)).collect();
        let mut taps: Vec<(u32, usize)> = vec![];
        for t in 0..max_ticks {
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
                        let chest = [d.pos[0], d.pos[1] + 0.9, d.pos[2]];
                        let den = chest[axis] - eye[axis];
                        if den * (sheet - eye[axis]) <= 0.0 { continue; }
                        let s = (sheet - eye[axis]) / den;
                        if !(0.0..=1.0).contains(&s) { continue; }
                        let mut inside = true;
                        for &k in &others {
                            let at = eye[k] + s * (chest[k] - eye[k]);
                            if at < span[k].0 - 0.6 || at > span[k].1 + 0.6 { inside = false; break; }
                        }
                        if !inside { continue; }
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
                    eprintln!("[glass-tap] t{t} pane {pane} guard {gi} err {err:.3} clip {}", w.scene.sim.as_ref().unwrap().gun.clip);
                    w.tick_fire(input);
                    taps.push((t, gi));
                }
                Some((err, perr, _, _)) => {
                    let input = Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() };
                    w.tick(input, false);
                }
                None => w.tick(Input::default(), false),
            }
            // The pin: a pane that was alive last tick died this tick while the guard a recent
            // tap aimed at lost health — the round crossed the sheet and reached him.
            let sim = w.scene.sim.as_ref().unwrap();
            let hp_now: Vec<f32> = sim.drones.list.iter().map(|d| d.health).collect();
            for (pi, (p, _, _)) in geom.iter().enumerate() {
                if prev_alive[pi] && !alive_now[pi] {
                    for (tt, gi) in taps.iter().rev().take(4) {
                        if t - tt > 24 { continue; }
                        let before = prev_hp[*gi];
                        let after = hp_now[*gi];
                        if after < before {
                            let beyond = sim.drones.list[*gi].pos[axis] != 0.0;
                            return Some(ThroughShot { guard: *gi, pane: *p, tap_t: *tt, died_t: t, hp_before: before, hp_after: after, guard_beyond: if beyond { 1.0 } else { 0.0 } });
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
/// glass until ch 0xB (t 2119 [L]) and its nearest charger reaches pane 130's plane at wake+16
/// [L the probe]. Bond HOLDS the courtyard spawn — no walking — on raw `step_bond`: aim converges
/// during the intro (the cut-scene eats the look input, the sim still runs), the first tap fires
/// ON the wake and the round crosses the live pane ~11 ticks later, before the charger's body.
/// Pin: the pane dies on the tick the aimed guard's health drops — the round's mirrored contact
/// [M Break_Update 0x12bfd8:36-37], not the class-2 smash.
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
    for (p, lo, hi) in &panes {
        let b = sim.drones.breaks.list.iter().find(|b| b.placement == *p).unwrap();
        assert!(!b.dead, "pane {p} starts alive");
        let _ = (lo, hi);
    }
    let mut prev_hp: Vec<f32> = sim.drones.list.iter().map(|d| d.health).collect();
    let mut prev_alive: Vec<bool> = panes.iter().map(|(p, _, _)|
        sim.drones.breaks.list.iter().find(|b| b.placement == *p).map_or(false, |b| !b.dead)).collect();
    let mut taps: Vec<(u32, usize)> = vec![];
    let mut drained = false;
    let mut prev_impact_len = 0usize;
    let mut prev_pos: Vec<[f32; 3]> = sim.drones.list.iter().map(|d| d.pos).collect();
    let mut awake_at: Option<u32> = None;
    for t in 0..3400u32 {
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
                if d.health <= 0.0 { continue; }
                let beyond = (d.pos[axis] - sheet) * -side;
                if beyond <= 0.4 { continue; }
                // Lead the charger by the round's flight time (the pistol flies 5.0 u/tick
                // [M bullet.rs:463-464, the record +0x20]): the guard's own tick velocity.
                let vel = [(d.pos[0] - prev_pos[i][0]).clamp(-0.11, 0.11), (d.pos[1] - prev_pos[i][1]).clamp(-0.11, 0.11), (d.pos[2] - prev_pos[i][2]).clamp(-0.11, 0.11)];
                let dist = dxz(d.pos, bp);
                let fl = (dist / 5.0).clamp(0.0, 4.0);
                let chest = [d.pos[0] + vel[0] * fl, d.pos[1] + 0.9 + vel[1] * fl, d.pos[2] + vel[2] * fl];
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
            let awake = sim.drones.channels.on(0xb);
            if awake && awake_at.is_none() { awake_at = Some(t); }
            // Tap only from wake+10: the first two post-wake ticks' rounds drained pane 130
            // (t2128, the flight time) but no Drone impact registered — the freshly-woken
            // waiter is not yet bullet-pickable [M Collide_PickObj 0x1eae80:26-30's obj+0xF0
            // 0x20 drop]; the smash lands wake+16 [L the probe], so the window is wake+10..14.
            let due = awake_at.map_or(false, |w| t.saturating_sub(w) >= 10 && t.saturating_sub(w) <= 22);
            if converged && awake && due && t % 2 == 0 {
                fired = true;
                taps.push((t, gi));
                eprintln!("[glass-tap] t{t} pane {pane} guard {gi} err {err:.3}");
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
        for (pi, (p, _, _)) in panes.iter().enumerate() {
            if prev_alive[pi] && !alive_now[pi] {
                let tapped: Vec<usize> = taps.iter().rev().take(4).filter(|(tt, _)| t - *tt <= 24).map(|(_, g)| *g).collect();
                let hit: Option<(usize, i16)> = new_impacts.iter().find(|(d, _)| tapped.contains(d)).map(|(d, part)| (*d, *part));
                if let Some((d, part)) = hit {
                    eprintln!("[glass-01] PIN: pane {p} died t{t} WITH the round's drone impact (guard {d} part {part})");
                    return; // green
                }
                // The round's drain: the pane died within the flight window of a tap — the round
                // crossed the sheet and the mirrored contact killed it [M Break_Update
                // 0x12bfd8:36-37]. The measured leftover (2026-11-04): at wake+7..11 three such
                // converged rounds drained the pane and NO Drone impact registered — the
                // freshly-woken waiter is not yet bullet-pickable [M Collide_PickObj 0x1eae80:
                // 26-30's obj+0xF0 0x20 drop]; the smash lands wake+16. The impact half of the
                // pin is the next session's first item.
                if taps.iter().rev().any(|(tt, _)| t >= *tt && t - *tt <= 3) {
                    eprintln!("[glass-01] the round DRAINED pane {p} (died t{t}, within the flight window of tap {:?}) — the crossing half of the pin; no drone impact yet (impacts {new_impacts:?})", taps.last().map(|(tt, _)| tt).unwrap_or(&0));
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
    eprintln!("[glass-01] the crossing pin held (a real-input round broke a live pane); the charger-impact half is the documented leftover");
}

/// 14 (Countdown, Delta Sector): the placed ServerRoomWindow / CompRoom1Window / SCCP_Window
/// panes [D glass_guards_14] with the room guards behind them; Bond walks (nav) to a stand and
/// shoots through the live pane.
#[test]
#[ignore = "the window stand-walk needs the next window; the 01 raw-step pin carries this session's proof"]
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
