//! Guard stealth on the two Tower levels (07000009, 0700000b), driven through the real app path
//! (`Sim::step_bond` + `Input`; Bond is placed once per scenario at a measured standoff from the
//! subject guard, then only input moves him). Each scenario pins the decompile's numbers and has
//! a sister that flips one input and must flip the outcome.
//!
//! The pinned law [M]:
//! - visibility `DroneVision_OpponentVisibility` 0x175af0: `r = range·(2·alert + 1)`,
//!   `v = 0.7 − (d − r)/r`, × `sin((half + 0.3926991 − |Δyaw|)/half · π/2)` (0 outside the padded
//!   cone), × Bond's factor `+0x22C` unless the drone has alert bits AND alertness ≥ 1; under 2
//!   units with line of sight 1.0. The factor is `Drone_ProcessOpponents` 0x1390c0: the product
//!   of the AI visibility boxes holding Bond, ×0.75 in the crouch substate (obj+0xF6 == 4).
//! - sight `DroneVision_ProcessDroneSight` 0x176a28: a guard with v ≥ 0.7 gets the LOS ray; every
//!   LOS tick counts `+0x270`; `HaveOpponentSight` 0x175ef0 acks the sighting once the count
//!   reaches `DroneFunc_ReactionTime` 0x14ac60 — on 07000002/03/05/09/0A
//!   `(1 − alert)·30·0.01·(200 − stat)` (EE-chopped: 59 at alert 0, stat 0), 0 once `+0x228 & 8`
//!   is set or alert ≥ 1; every other level 10.
//! - hearing `Sound_Alertness` 0x1ddc48 → `HandleSoundAlerts` 0x148a30: weight × (1 inside the
//!   inner radius, else (outer − d)/(outer − inner)), alertness += 0.01·that, msg 0x14 to the
//!   drone; `NDrone2_ControlSTANDARD` 0x1490f0:65-67 clamps alertness to 1.
//! Opt-in: NF_ISO.

#[path = "common/walker_b2.rs"]
mod walker;

use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// 07000009's stationary lobby guard: placement g2, state Civilian 0x11, yaw 0, range 12, half
/// FOV π/2, alertness 0, props 0x3c/0x1f set [D roster probe of the level]. Reaction stat 80: the
/// guard is drone type 0xD (param 15), whose `drone_stats` record only the level's first type-0xD
/// placement carries (params 26-32, tag 0x80010006: kind 1 = words 0x1a800950 / 0x15050 →
/// acc 80, aggression 1, reaction 80, recover 80 [D, `behaviour_util_get` 0x1c7a98 at the
/// 0x303330 widths]); `NDrone2_DoModeSettingsNEW` 0x14c600:81-106 copies `+0x14` of
/// `getStats(type, 1)` into `+0xB8` [M]. The same layout reads type 2's 8/1/12/…/80 live
/// [L ee_start_07000001.bin, ee_02b.bin: drone_stats 0x31aa30].
const G09: usize = 2;

/// Load, settle 30 ticks (past the 15-frame vision wait), keep only guard `gi` in play: the other
/// drones leave the live set (`present == false`, which every shared pass skips), so no second
/// guard's sight chain, shout or alarm feeds the subject.
fn setup(lvl: &str, gi: usize) -> Walker {
    let mut w = Walker::new(nf_view::load(&iso(), lvl, false).expect("load"));
    w.wait(30);
    let sim = w.scene.sim.as_mut().unwrap();
    for (i, g) in sim.drones.list.iter_mut().enumerate() {
        if i != gi { g.present = false; g.enabled = false; }
    }
    w
}

/// Release one of 0b's garrison: the guards are spawner templates. Channel 0x56 both starts
/// spawner placement 267 (start = param 4 [D]) — which releases its templates into WaitSwitch
/// 0x01 [M DroneSpawner_Control 0x137e68 → 0x137cb0] — and is the templates' switch channel
/// (placement param 3 [D]) that wakes them [M NDrone2_DSTATE_WaitSwitch 0x15b960:21-37]. The
/// channel is held on here as the level's trigger holds it. The first woken guard is the subject
/// (any other leaves the live set). Returns its index.
fn release_0b(w: &mut Walker) -> usize {
    let tpl = [0usize, 1, 2, 3]; // placement 267's templates [D spawner dump]
    for _ in 0..600 {
        { let sim = w.scene.sim.as_mut().unwrap(); let f = sim.drones.frame; sim.drones.channels.set(0x56, 1, f); }
        w.wait(1);
        let sim = w.scene.sim.as_mut().unwrap();
        if let Some(&gi) = tpl.iter().find(|&&i| { let g = &sim.drones.list[i]; g.present && g.enabled && g.vision_on }) {
            for (i, g) in sim.drones.list.iter_mut().enumerate() { if i != gi { g.present = false; g.enabled = false; } }
            return gi;
        }
    }
    panic!("no 0b garrison guard woke");
}

/// Bond at `dist` from guard `gi` along the guard's facing turned by `ang` (0 = straight ahead),
/// on the guard's floor; `face` = Bond looks at the guard (else away).
fn place(w: &mut Walker, gi: usize, dist: f32, ang: f32, face: bool) -> [f32; 3] {
    let sim = w.scene.sim.as_mut().unwrap();
    let g = &sim.drones.list[gi];
    let a = g.yaw + ang;
    let p = [g.pos[0] + dist * a.sin(), g.pos[1] + 0.09, g.pos[2] + dist * a.cos()];
    sim.bond.pos = p;
    sim.bond.yaw = if face { a + std::f32::consts::PI } else { a };
    p
}

fn wrap(a: f32) -> f32 {
    let mut a = a;
    while a > std::f32::consts::PI { a -= std::f32::consts::TAU; }
    while a < -std::f32::consts::PI { a += std::f32::consts::TAU; }
    a
}

/// The decompile's visibility (0x175af0) from first principles. The yaw to Bond is the game's
/// `ATAN2_APPROX` 0x1e4528 (what `Drone_GetOpponentInfo` stores in +0x1c4 — its ~0.005 rad error
/// matters at the cone edge); the sine is host math, so the PS2 sine shows as a small tolerance.
fn predict(g: &nf_game::drone::Drone, bond: [f32; 3], factor: f32) -> f32 {
    let (dx, dy, dz) = (bond[0] - g.pos[0], bond[1] - g.pos[1], bond[2] - g.pos[2]);
    let d = (dx * dx + dy * dy + dz * dz).sqrt();
    if d < 2.0 { return 1.0; }
    let r = g.sight_range * (2.0 * g.alert + 1.0);
    let mut v = 0.7 - (d - r) / r;
    if g.half_fov != 0.0 {
        let m = g.half_fov + 0.392_699_1 - wrap(nf_game::trig::atan2_approx(dx, dz) - g.yaw).abs();
        if m < 0.0 { return 0.0; }
        v *= (m / g.half_fov * std::f32::consts::FRAC_PI_2).sin();
    }
    (v * factor).max(0.0)
}

#[derive(Debug, Default)]
struct Trace { first_los: Option<u32>, first_seen: Option<u32>, states: Vec<(u32, u8)>, max_vis: f32, max_err: f32, los_dist: Option<f32> }

/// Run `ticks` with a fixed input, recording the guard's sight timeline and the worst
/// |port visibility − predicted| (`factor` = Bond's expected `+0x22C`).
fn run(w: &mut Walker, gi: usize, ticks: u32, input: Input, factor: f32) -> Trace {
    let mut t = Trace::default();
    let mut last = w.scene.sim.as_ref().unwrap().drones.list[gi].state;
    for k in 0..ticks {
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        let g = &sim.drones.list[gi];
        let p = sim.bond.pos;
        if g.los && t.first_los.is_none() {
            t.first_los = Some(k);
            t.los_dist = Some(((p[0] - g.pos[0]).powi(2) + (p[1] - g.pos[1]).powi(2) + (p[2] - g.pos[2]).powi(2)).sqrt());
        }
        if g.seen && t.first_seen.is_none() { t.first_seen = Some(k); }
        t.max_vis = t.max_vis.max(g.visibility);
        if t.first_seen.is_none() { t.max_err = t.max_err.max((g.visibility - predict(g, p, factor)).abs()); }
        if g.state != last { t.states.push((k, g.state)); last = g.state; }
    }
    t
}

fn hold_crouch() -> Input { Input { crouch: true, crouch_hold: true, ..Default::default() } }

/// The head-on distance where the decompile's visibility crosses 0.7 for a factor `f`:
/// (1.7 − d/r)·sin(π/2 + 0.39)·f = 0.7 (alert 0, half FOV π/2).
fn crossing(range: f32, f: f32) -> f32 {
    let s = ((std::f32::consts::FRAC_PI_2 + 0.392_699_1) / std::f32::consts::FRAC_PI_2 * std::f32::consts::FRAC_PI_2).sin();
    range * (1.7 - 0.7 / (s * f))
}

/// Standing still 10 units in front of the 09 guard: v ≈ 0.80 ≥ 0.7, the LOS ray lands on the
/// first tick and the sighting is acked on the 35th LOS tick (ReactionTime at alert 0, stat 80:
/// ((1 − 0)·30)·0.01·(200 − 80), EE-chopped 35.999996 → 35 [M 0x14ac60]) — 34 ticks after the first. On 09 the security guard's `NearArmedPerson`
/// is prop 0x3c + the ack alone [M 0x1787c8:14-21], so the ack tick itself leaves Civilian
/// (CivilianScared → CivilianHiding 0x13 [M 0x15d118:46-72, 0x15d3a0]).
/// Sister: the same spot crouched — ×0.75 puts v at ≈ 0.60 < 0.7 [M 0x1390c0:40-42]: no ray, no
/// sighting in 200 ticks.
#[test]
fn vision_09_standing_vs_crouched_at_10() {
    let mut w = setup("07000009.bin", G09);
    let g = &w.scene.sim.as_ref().unwrap().drones.list[G09];
    assert_eq!((g.sight_range, g.half_fov, g.reaction, g.alert), (12.0, std::f32::consts::FRAC_PI_2, 80, 0.0), "the 09 lobby guard's sight row [D]");
    place(&mut w, G09, 10.0, 0.0, true);
    let tr = run(&mut w, G09, 200, Input::default(), 1.0);
    eprintln!("standing: {tr:?}");
    assert!(tr.max_err < 3e-3, "the port's visibility follows 0x175af0 (worst error {})", tr.max_err);
    assert!((tr.max_vis - 0.8).abs() < 0.01, "v at 10u head-on ≈ 0.80, got {}", tr.max_vis);
    assert_eq!(tr.first_los, Some(0), "the LOS ray lands on the first tick");
    assert_eq!(tr.first_seen.map(|s| s - tr.first_los.unwrap()), Some(34), "acked on the 35th LOS tick (ReactionTime 35 at stat 80)");
    assert_eq!(tr.states.first(), Some(&(tr.first_seen.unwrap(), nf_game::drone::state::CIVILIAN_HIDING)),
        "the ack tick leaves Civilian through CivilianScared to CivilianHiding");

    let mut w = setup("07000009.bin", G09);
    place(&mut w, G09, 10.0, 0.0, true);
    let tr = run(&mut w, G09, 200, hold_crouch(), 0.75);
    eprintln!("crouched: {tr:?}");
    assert!(w.scene.sim.as_ref().unwrap().bond.crouched, "Bond held the crouch");
    assert!(tr.max_err < 3e-3, "crouched visibility = 0.75 × the standing law (worst error {})", tr.max_err);
    assert!((tr.max_vis - 0.6).abs() < 0.01, "v crouched ≈ 0.60, got {}", tr.max_vis);
    assert_eq!((tr.first_los, tr.first_seen), (None, None), "crouched at 10u the guard never gets the ray");
    assert!(tr.states.is_empty(), "the guard stays in Civilian: {:?}", tr.states);
}

/// The cone pad: the half FOV π/2 plus 0.3927 (22.5°) [M 0x175af0:62-66]. At 4 units, 1.80 rad
/// off the guard's facing (beyond the bare half FOV) is inside the padded cone — v ≈ 0.32, too
/// small to see; the sister at 2.10 rad is outside — v exactly 0. Straight behind is 0 as well.
#[test]
fn vision_09_cone_pad_edge() {
    for (ang, inside) in [(1.80f32, true), (2.10, false), (std::f32::consts::PI, false)] {
        let mut w = setup("07000009.bin", G09);
        place(&mut w, G09, 4.0, ang, true);
        let tr = run(&mut w, G09, 30, Input::default(), 1.0);
        eprintln!("ang {ang}: {tr:?}");
        assert!(tr.max_err < 3e-3, "visibility law at {ang} rad (worst error {})", tr.max_err);
        assert_eq!(tr.max_vis > 0.0, inside, "{ang} rad inside the padded cone: {inside}");
        assert!(tr.max_vis < 0.7 && tr.first_seen.is_none(), "no sighting at the cone edge");
    }
}

/// Walking in from 14 units straight at the guard: the ray first lands where v crosses 0.7 —
/// standing at d ≈ 11.31, crouch-walking at d ≈ 8.28 [M 0x175af0 × 0x1390c0]. The walk speed
/// bounds the overshoot to one tick's step (≤ 0.2u).
#[test]
fn vision_09_walk_in_crossing_standing_vs_crouched() {
    for (crouch, f) in [(false, 1.0f32), (true, 0.75)] {
        let mut w = setup("07000009.bin", G09);
        place(&mut w, G09, 14.0, 0.0, true);
        if crouch { w.tick(hold_crouch(), false); }
        let input = Input { forward: 1.0, crouch, crouch_hold: crouch, ..Default::default() };
        let tr = run(&mut w, G09, 400, input, f);
        let want = crossing(12.0, f);
        eprintln!("crouch {crouch}: want {want:.3} {tr:?}");
        let d = tr.los_dist.expect("the ray lands on the walk-in");
        assert!(d <= want + 0.01 && d > want - 0.25, "first ray at {d:.3}, the law's crossing {want:.3}");
        assert!(tr.max_err < 3e-3, "visibility law on the walk (worst error {})", tr.max_err);
    }
}

fn cycle_to(w: &mut Walker, want: &dyn Fn(u8) -> bool) -> bool {
    for _ in 0..14 {
        { let sim = w.scene.sim.as_ref().unwrap(); if want(sim.drones.bond_weapon) && sim.gun.switch_state == 0 && sim.gun.clip > 0 { return true; } }
        w.tick(Input { weapon_cycle: 1, ..Default::default() }, false);
        for _ in 0..70 { w.tick(Input::default(), false); }
    }
    false
}

/// One trigger pull 10 units behind the 09 guard (out of the cone, facing away). The P2K's fire
/// script plays 0x403 at frame 1 [D 0x06000823 op 1]; its table row (inner/outer/weight [D
/// ACTION.ELF sound table]) sets the loudness exactly, alertness gains 0.01 of it and the drone
/// takes msg 0x14 → HeardNoise 0xa2 → HeardNoiseAware 0xa3 the same tick [M 0x177800 →
/// 0x173be8]. Sister: the K5 dart's 0x5ce (0x06000948 op 1 [D]) carries weight 0 — no loudness,
/// no alertness, no state change: the dart is silent to guards.
#[test]
fn hearing_09_gunshot_vs_dart() {
    for lethal in [true, false] {
        let mut w = setup("07000009.bin", G09);
        // Switch weapons parked at the spawn (out of the guard's sight), then step behind him.
        { let sim = w.scene.sim.as_mut().unwrap(); sim.bond.pos = [3.5873525, -375.9931, 36.755123]; }
        let ok = cycle_to(&mut w, &|b| if lethal { b == 2 } else { matches!(b, 0x43 | 0x44) });
        assert!(ok, "weapon in hand (lethal {lethal})");
        place(&mut w, G09, 10.0, std::f32::consts::PI, false);
        w.wait(5);
        let id = if lethal { 0x403 } else { 0x5ce };
        let (h, d) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let g = &sim.drones.list[G09];
            let e = sim.bond.head();
            (sim.sfx.as_ref().unwrap().get(id), ((e[0] - g.pos[0]).powi(2) + (e[1] - g.pos[1]).powi(2) + (e[2] - g.pos[2]).powi(2)).sqrt())
        };
        let want = if h.weight == 0.0 || d > h.outer { 0.0 } else if d < h.inner { h.weight } else { h.weight * (h.outer - d) / (h.outer - h.inner) };
        eprintln!("id {id:#x} row {h:?} d {d:.3} want {want:.3}");
        let mut heard = None;
        for k in 0..30u32 {
            if k < 2 { w.tick_fire(Input::default()); } else { w.tick(Input::default(), false); }
            let g = &w.scene.sim.as_ref().unwrap().drones.list[G09];
            if heard.is_none() && g.loudness > 0.0 { heard = Some((k, g.loudness, g.alert, g.log.clone())); }
        }
        let g = &w.scene.sim.as_ref().unwrap().drones.list[G09];
        if lethal {
            let (k, loud, alert, log) = heard.expect("the gunshot is heard");
            eprintln!("heard at k{k}: loud {loud} alert {alert} log {log:?}");
            assert!((loud - want).abs() < 0.05, "loudness {loud} = the 0x403 row at {d:.2}u ({want})");
            assert!((alert - 0.01 * loud).abs() < 1e-3, "alertness gains 0.01 × loudness (decay ≤ 1/3000 a tick)");
            let tail: Vec<u8> = log.iter().rev().take(2).map(|e| e.1).collect();
            assert_eq!(tail, vec![nf_game::drone::state::HEARD_NOISE_AWARE, nf_game::drone::state::HEARD_NOISE], "msg 0x14 → 0xa2 → 0xa3");
        } else {
            assert_eq!(h.weight, 0.0, "the dart's sound row carries no alert weight [D]");
            assert!(heard.is_none(), "the dart shot is silent to the guard");
            assert_eq!(g.alert, 0.0);
            assert_eq!(g.state, nf_game::drone::state::CIVILIAN, "no reaction to the dart");
        }
    }
}

/// Three P2K shots 3 units behind the guard: each share ≈ 0.6 sums past 1, but
/// `ControlSTANDARD` clamps alertness to 1 every tick [M 0x1490f0:65-67] — the range never grows
/// past ×3. Sister: the summed shares themselves exceed 1 (the clamp is what holds it).
#[test]
fn hearing_09_alertness_clamps_at_one() {
    let mut w = setup("07000009.bin", G09);
    { let sim = w.scene.sim.as_mut().unwrap(); sim.bond.pos = [3.5873525, -375.9931, 36.755123]; }
    assert!(cycle_to(&mut w, &|b| b == 2), "the P2K in hand");
    place(&mut w, G09, 3.0, std::f32::consts::PI, false);
    w.wait(5);
    let (mut shares, mut max_alert, mut last_heard) = (0.0f32, 0.0f32, 0u32);
    for k in 0..120u32 {
        if k % 30 < 2 && k < 90 { w.tick_fire(Input::default()); } else { w.tick(Input::default(), false); }
        let g = &w.scene.sim.as_ref().unwrap().drones.list[G09];
        if g.heard_frame != last_heard && g.heard_frame != 0 { last_heard = g.heard_frame; }
        if g.loudness > 0.0 && g.heard_frame == w.scene.sim.as_ref().unwrap().drones.frame { shares += g.sound_share; }
        max_alert = max_alert.max(g.alert);
    }
    eprintln!("shares {shares} max alert {max_alert}");
    assert!(shares > 1.0, "the heard shares sum past 1 ({shares})");
    assert!(max_alert <= 1.0, "alertness stays clamped at 1 ({max_alert})");
}

/// `DroneFunc_ReactionTime`'s level switch [M 0x14ac60:9-26]: off the stealth list (0700000b)
/// every sighting costs FRAME_RATE_DIV·10 = 10 LOS ticks whatever the alertness — 0b's garrison
/// (alertness 1.0 [D]) and the same guard at 0 are both acked on the 10th LOS tick. Sister: on
/// 07000009 (on the list) the guard at alertness 1 takes the zero arm — acked on the first.
#[test]
fn reaction_level_arms_0b_vs_09() {
    for (lvl, gi, alert, want) in [("0700000b.bin", 0, 1.0f32, 9u32), ("0700000b.bin", 0, 0.0, 9), ("07000009.bin", G09, 1.0, 0)] {
        let mut w = setup(lvl, gi);
        let gi = if lvl == "0700000b.bin" { release_0b(&mut w) } else { gi };
        { let g = &mut w.scene.sim.as_mut().unwrap().drones.list[gi]; g.alert = alert; g.alert_floor = alert; }
        w.wait(2); // the ENTER (vision on) with Bond still at the spawn
        place(&mut w, gi, 8.0, 0.0, true);
        let tr = run(&mut w, gi, 120, Input::default(), 1.0);
        eprintln!("{lvl} alert {alert}: {tr:?}");
        let (l, s) = (tr.first_los.expect("the ray lands"), tr.first_seen.expect("the sighting is acked"));
        assert_eq!(s - l, want, "{lvl} alert {alert}: ack latency");
    }
}


/// The deaf bit: CivilianScared's ENTER sets `+0x4F8 |= 0x400000` [M 0x15d3a0:28] and nothing on
/// the scared/hiding path clears it (RouteMsgDCV clears only 0x10000/0x2000 [M 0x171df8:182-183]),
/// so `HandleSoundAlerts` skips the guard for good [M 0x148a30:43-44]. Bond lets the 09 guard see
/// him (→ CivilianHiding), turns away and fires the P2K: no loudness, no msg 0x14. Sister: the
/// unscared guard at the same spot hears the same shot (`hearing_09_gunshot_vs_dart` pins the
/// loudness; here only that it is heard).
#[test]
fn hearing_09_scared_guard_is_deaf() {
    for scare in [true, false] {
        let mut w = setup("07000009.bin", G09);
        { let sim = w.scene.sim.as_mut().unwrap(); sim.bond.pos = [3.5873525, -375.9931, 36.755123]; }
        assert!(cycle_to(&mut w, &|b| b == 2), "the P2K in hand");
        if scare {
            place(&mut w, G09, 6.0, 0.0, true);
            let tr = run(&mut w, G09, 120, Input::default(), 1.0);
            assert!(tr.states.iter().any(|s| s.1 == nf_game::drone::state::CIVILIAN_HIDING), "the guard saw Bond and hid: {:?}", tr.states);
            assert!(w.scene.sim.as_ref().unwrap().drones.list[G09].deaf, "CivilianScared left the deaf bit");
        }
        // Behind him at 6u (he stands where he hid), facing away; one shot.
        place(&mut w, G09, 6.0, std::f32::consts::PI, false);
        w.wait(5);
        let h0 = w.scene.sim.as_ref().unwrap().drones.list[G09].heard_frame;
        for k in 0..20u32 { if k < 2 { w.tick_fire(Input::default()); } else { w.tick(Input::default(), false); } }
        let g = &w.scene.sim.as_ref().unwrap().drones.list[G09];
        eprintln!("scare {scare}: heard {h0} -> {} state {:#x} deaf {}", g.heard_frame, g.state, g.deaf);
        assert_eq!(g.heard_frame != h0, !scare, "heard the shot: {}", !scare);
    }
}
