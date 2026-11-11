//! dead-body-collision: the game gives a dead guard a collision body and the player's capsule
//! push MEETS it — a walking Bond is shoved off a fresh corpse. The chain, all [M]:
//! `NDrone2_Collision` 0x140e90's dead branch (+0x4F8 & 0x200) arms the HITTEST unconditionally
//! (+0x464 = 0x800, :142) and rebuilds the capsule from the ragdoll bone chain (palette bones
//! 5/0x37/0x33, radius min(+0xA0, 0.4)) every tick; Bond's body hittest mask is 0 every tick
//! (`Player_Collision` 0x194df0:88 — the walk arm leaves it cleared), so Bond PICKS the corpse
//! (`Collide_PickObj` 0x1eae80 — mask 0 excludes nothing); the contact lands on Bond's list and
//! `Collide_Intersect` 0x1ebaa8:100-107 dispatches `Collide_Jointy` 0x1ec180 (the corpse's
//! +0xFA & 8 is set at creation, `AnimObjectNew` 0x1be910:58; the SP class gates pass — Bond is
//! class 3 `Player_Init` 0x1910f8:76, the corpse class 2, and only a class-3 CANDIDATE is
//! refused); Jointy pushes Bond's capsule arms and fills the +0x10 accumulator
//! (BLData+0x750), which `Player_CollisionHandler` 0x1949a8:136-140 adds to obj+0x30. No gate
//! anywhere is grounded-airborne: a WALKING Bond is blocked exactly like a dropping one.
//!
//! The repro walks the real input path: spawn → the intro cut-scene → Bond's own gun kills the
//! corridor guard → Bond walks straight at the fresh corpse. On the old code (corpses excluded
//! from `bot_capsules`) he walks through the body; with the port's dead capsule he is held on
//! the standoff. The pinned bound is the standoff, not the exact shape: the capsule's exact
//! arms/height ride the death anim's palette (a live-check item), so the assert allows the
//! per-tick penetration sliver but not a crossing.

#[path = "common/walker.rs"]
mod walker;

use walker::{dxz, wrap_pi, Walker};

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Squared distance from `p` to the segment `a`–`b`.
fn seg_dist(a: [f32; 3], b: [f32; 3], p: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let ab2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
    let t = if ab2 <= 1e-9 { 0.0 } else { (ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / ab2 }.clamp(0.0, 1.0);
    let c = [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t];
    ((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2)).sqrt()
}

/// Distance between two segments (the clamped closest-points iteration — Bond's capsule is
/// vertical, the corpse's lies along the body, so two clamped rounds converge as Jointy's own
/// `vecutil_calculate_closest_point_on_line` rounds do [M 0x1ec180:243-248]).
fn seg_seg_dist(a1: [f32; 3], b1: [f32; 3], a2: [f32; 3], b2: [f32; 3]) -> f32 {
    let mut p = a2;
    let mut d = seg_dist(a1, b1, p);
    for _ in 0..3 {
        let q = closest_on(a2, b2, closest_on(a1, b1, p));
        let nd = seg_dist(a1, b1, q);
        if nd > d { break; }
        d = nd;
        p = q;
    }
    d
}

fn closest_on(a: [f32; 3], b: [f32; 3], p: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let ab2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
    let t = if ab2 <= 1e-9 { 0.0 } else { (ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / ab2 }.clamp(0.0, 1.0);
    [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t]
}

#[test]
fn walking_bond_is_pushed_off_a_fresh_corpse() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load 07000001"));
    // The intro cut-scene writes ch 0xB ~tick 2119 and its tail re-places Bond (walkreal-a's
    // wait-out).
    let mut enabled_at = None;
    for t in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) { enabled_at = Some(t); break; }
    }
    enabled_at.expect("the intro must write ch 0xB");
    w.wait_enabled(6000, "the intro tail");
    w.wait(300);

    // The nearest started enemy: Bond's own gun kills him through the real fire path.
    let gi = {
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        (0..sim.drones.list.len()).filter(|&i| {
            let d = &sim.drones.list[i];
            d.team == 1 && d.started && d.enabled && !d.corpse && d.health > 0.0
        }).min_by_key(|&i| dxz(sim.drones.list[i].pos, bp) as u32 as i64
            // sort key: XZ distance in fine units
            ).expect("a started guard near the intro tail")
    };
    assert!(w.shoot(gi, "the corridor guard"), "the kill must land");
    // The death anim plays out; the capsule follows the palette live — give it a beat to settle.
    let mut corpse_at = None;
    for _ in 0..600 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.list[gi].corpse { corpse_at = Some(()); break; }
    }
    corpse_at.expect("the kill must leave a corpse");
    w.wait(90);
    // The death anim may still be carrying the body (the capsule rides the posed bones every
    // tick [M NDrone2_Collision 0x140e90 dead branch]): a long fall moved the hips 1.3 u after
    // the 90-tick beat on one kill. Wait for the capsule to come to rest (unchanged for 30
    // ticks, cap 900) so the walk aims at the body where it lies.
    let mut still = 0;
    let mut prev = w.scene.sim.as_ref().unwrap().corpse_capsules(w.scene.sim.as_ref().unwrap().drones.list[gi].pos);
    for _ in 0..900 {
        if still >= 30 { break; }
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let now = sim.corpse_capsules(sim.drones.list[gi].pos);
        let same = now.len() == prev.len() && now.iter().zip(&prev).all(|(x, y)| (0..3).all(|k| (x.0[k] - y.0[k]).abs() < 1e-3 && (x.1[k] - y.1[k]).abs() < 1e-3));
        still = if same { still + 1 } else { 0 };
        prev = now;
    }
    assert!(still >= 30, "the corpse's capsule comes to rest");

    let cpos = w.scene.sim.as_ref().unwrap().drones.list[gi].pos;
    // The corpse's own capsule (the helper scopes to a point: scope it to the corpse itself).
    let caps = w.scene.sim.as_ref().unwrap().corpse_capsules(cpos);
    assert!(!caps.is_empty(), "the fresh corpse must present a dead capsule (0x140e90 dead branch)");
    let (r_sum, y_lo, y_hi) = caps.iter().fold((0.0f32, f32::MAX, f32::MIN), |(rs, lo, hi), (a, b, r)| {
        (rs + *r, lo.min(a[1]).min(b[1]), hi.max(a[1]).max(b[1]))
    });
    eprintln!("[dead-body] corpse at ({:.2},{:.2},{:.2}), {} capsule(s), r {:.2}, arm Y {:.2}..{:.2}, bond pos {:?}",
        cpos[0], cpos[1], cpos[2], caps.len(), r_sum / caps.len() as f32, y_lo, y_hi, w.scene.sim.as_ref().unwrap().bond.pos);

    // Step back out of the capsule's reach first, so the approach is a real walk into the body.
    let bp = w.scene.sim.as_ref().unwrap().bond.pos;
    if dxz(bp, cpos) < 1.6 {
        let away = [bp[0] + (bp[0] - cpos[0]) * 3.0, bp[1], bp[2] + (bp[2] - cpos[2]) * 3.0];
        for _ in 0..240 {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            if dxz(p, away) < 0.8 || dxz(p, cpos) > 2.2 { break; }
            let desired = (away[0] - p[0]).atan2(away[2] - p[2]);
            let err = wrap_pi(desired - sim.bond.yaw);
            w.tick(Input { turn: err.clamp(-0.07, 0.07), forward: 0.55, ..Default::default() }, false);
        }
    }

    // Walk straight at the corpse's centre, real input only, and record the worst penetration of
    // Bond's capsule (radius 0.55 [M Player_Collision 0x194df0 — BL+0x7CC = 0.55 walk arm]) into
    // every nearby dead capsule.
    let mut min_pen = f32::MAX;
    for k in 0..900 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let desired = (cpos[0] - p[0]).atan2(cpos[2] - p[2]);
            let err = wrap_pi(desired - sim.bond.yaw);
            let input = Input {
                turn: err.clamp(-0.07, 0.07),
                forward: if err.abs() > 1.2 { 0.0 } else { 0.55 },
                ..Default::default()
            };
            w.tick(input, false);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        if k % 120 == 0 {
            let (ja, jb) = sim.bond.capsule_seg();
            eprintln!("[dead-body] tick {k}: bond ({:.2},{:.2},{:.2}) dxz {:.2} capsule ({:.2},{:.2},{:.2})-({:.2},{:.2},{:.2})",
                p[0], p[1], p[2], dxz(p, cpos), ja[0], ja[1], ja[2], jb[0], jb[1], jb[2]);
        }
        let (ja, jb) = sim.bond.capsule_seg();
        for (a, b, r) in &caps {
            min_pen = min_pen.min(seg_seg_dist(*a, *b, ja, jb) - (r + 0.55));
        }
    }
    eprintln!("[dead-body] worst capsule penetration {min_pen:.3} (standoff held = >= ~-0.12)");
    // The push holds Bond on the standoff: the per-tick sliver may nick the surface, the walk
    // may NOT cross it. The old code's trail passes through the body's centre — deep negative.
    assert!(min_pen > -0.12,
        "Bond's capsule crossed the fresh corpse's dead capsule (worst penetration {min_pen:.3}) — the game's Jointy push (0x1ec180 → 0x1949a8:136-140) must hold him on the standoff");
}
