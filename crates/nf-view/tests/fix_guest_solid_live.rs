//! fix-guest-solid (07000007): the party guests are solid to Bond.
//!
//! The game's chain, every step decompiled:
//! - the player's kind-0x800 hittest is built EVERY frame with mask 0 — no stance gate
//!   [M Player_Collision 0x194df0:88];
//! - a bot's hittest kind word +0x464 goes 0x800 when `NDrone2_DoCollision` returns 1
//!   [M NDrone2_Collision 0x140e90:95-101]; for the class word +0xd8 == 0xc DoCollision is
//!   UNCONDITIONAL once alive (+0x4f8&0x200 clear) and enabled (+0x20) [M
//!   NDrone2_DoCollision 0x149ec8:16-34]; every OTHER class (the 07000007 guests are +0xd8
//!   0xd/0xe — skins 0x5000015/0x5000019-0x21 [M DefaultInit 0x14b300:128-132,156-160]) returns
//!   1 through the same arms — NearPlayerCollision (< 2.0 [M DroneFunc_NearPlayerCollision
//!   0x14b190]) fires first [M 0x149ec8:23-26];
//! - the guest capsule is the walking arm, radius 0.55 for +0xd8 == 0xc else 0.4
//!   [M 0x140e90:91-94,102];
//! - `Collide_Jointy` 0x1ec180's SP arm (MPSettings+0x384 == 0) pushes the class-3 OWNER (Bond)
//!   off the class-2 bot with fVar12 = 1.0 and returns 0 when the OTHER is class 3 [M :229-233,
//!   251-266] — the bot is never displaced by Bond; Bond is displaced by the bot.

#[path = "common/walker.rs"]
mod walker;

use nf_game::drone::state;
use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn dxz(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }

fn load_party() -> Walker {
    Walker::new(nf_view::load(&iso(), "07000007.bin", false).expect("load 07000007"))
}

/// Bond walks (real input) into a standing enabled guest and is pushed off. The 07000007
/// guests carry class +0xd8 0xd/0xe (the near arm of DoCollision [M 0x149ec8:23-26]) with the
/// 0.4 walking capsule [M 0x140e90:91-94]; his own hittest radius is 0.55, so the spines rest
/// at 0.95 apart.
#[test]
fn bond_walking_into_a_standing_guest_is_pushed_off() {
    let mut w = load_party();
    w.wait_enabled(3000, "the party start");
    w.wait(60);
    // A standing, enabled, team-3 guest on the party floor. The idle ch-6/ch-21/ch-23 crowd
    // sits in WAIT_SWITCH with Enable(0) [M 0x15b960:14-16] — its collision is OFF in the game
    // too [M 0x149ec8:12] — so the pin uses a guest the game itself holds enabled (the probe:
    // 15 such at the party start, e.g. the PARTY_GIRL 0x2f pair at (44.3,44.0,-62.1)).
    let sim = w.scene.sim.as_mut().unwrap();
    for _ in 0..90 { sim.step_bond(Input::default(), false, false, false); }
    let candidates: Vec<usize> = sim.drones.list.iter().enumerate()
        .filter(|(_, d)| d.present && !d.corpse && d.enabled && d.dtype == 10 && d.team == 3
            && matches!(d.state, state::CIVILIAN | state::PARTY_GIRL | state::CIVILIAN_GUARD | state::CIVILIAN_MISSION_WAIT)
            && d.pos[1] > 42.0 && d.pos[1] < 45.0)
        .map(|(i, _)| i).collect();
    assert!(!candidates.is_empty(), "no enabled standing guest at the party start — guests {:?}",
        sim.drones.list.iter().filter(|d| d.present && d.dtype == 10)
            .map(|d| (d.state, d.enabled, d.pos[0], d.pos[2])).collect::<Vec<_>>());
    // The least-drifting candidate over a settle window (deterministic).
    let pick = {
        let mut best = candidates[0];
        let mut best_move = f32::MAX;
        for &i in &candidates {
            let p0 = sim.drones.list[i].pos;
            let mut p1 = p0;
            for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); p1 = sim.drones.list[i].pos; }
            let m = dxz(p0, p1);
            if m < best_move { best_move = m; best = i; }
        }
        best
    };
    let (gp, gstate) = (sim.drones.list[pick].pos, sim.drones.list[pick].state);
    eprintln!("[guest-solid] guest {pick} state {gstate:#x} pos {gp:?}");
    // Setup staging (the fix_guest_reactions pattern): Bond 1.1 units away, then the REAL walk.
    let start = [gp[0] - 1.1, gp[1], gp[2] + 0.35];
    sim.bond.pos = start;
    sim.bond.yaw = (gp[0] - start[0]).atan2(gp[2] - start[2]);
    let mut min_dist = f32::MAX;
    for t in 0..150u32 {
        let p = sim.bond.pos;
        let g = sim.drones.list[pick].pos;
        min_dist = min_dist.min(dxz(p, g));
        let desired = (g[0] - p[0]).atan2(g[2] - p[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 1.0 { 0.0 } else { 1.0 }, ..Default::default() };
        sim.step_bond(input, false, false, false);
        if t % 50 == 0 { eprintln!("[guest-solid] t{t} bond {:?} guest {gp:?} dxz {:.2}", sim.bond.pos, dxz(sim.bond.pos, gp)); }
    }
    let end = sim.bond.pos;
    eprintln!("[guest-solid] min spine dist {min_dist:.3} end {end:?} start {start:?}");
    assert!(dxz(start, end) > 0.2, "Bond walked (the approach moved him), start {start:?} end {end:?}");
    assert!(min_dist > 0.8, "Bond is held out of the guest's capsule (0.55 + 0.4 [M 0x140e90:91-94] with the tick's step slop), min spine dist {min_dist:.3} — he walked THROUGH the guest");
}

/// A walking guest meeting standing Bond: Bond is displaced (the SP Jointy arm pushes the
/// class-3 owner [M 0x1ec180:229-233,251-266]) and the guest keeps walking — bots never
/// self-push against a class-3 other [M :231-233].
#[test]
fn a_walking_guest_meeting_bond_pushes_bond_and_keeps_walking() {
    let mut w = load_party();
    w.wait_enabled(3000, "the party start");
    w.wait(60);
    // Wake the ch-6 walkers at plate #476 (the same anchors the wake test walks).
    for (name, at) in [
        ("west aisle south", [35.7f32, 43.6, -70.6]),
        ("west aisle north", [42.6, 43.6, -70.5]),
        ("the west wall run", [42.6, 43.6, -45.0]),
        ("the north door", [49.4, 43.6, -44.7]),
    ] { w.walk_nav_to(at, name, 1.3); }
    {
        let target = [49.26f32, 43.6, -46.71];
        let mut still = 0u32;
        let mut last = [0.0f32, 0.0];
        for _ in 0..900u32 {
            if w.scene.sim.as_ref().unwrap().drones.channels.on(6) { break; }
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let use_key = if (p[0] - last[0]).abs() < 0.01 && (p[2] - last[1]).abs() < 0.01 {
                still += 1; still % 30 == 0
            } else { still = 0; last = [p[0], p[2]]; false };
            let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let input = Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 1.0 { 0.0 } else { 1.0 }, ..Default::default() };
            w.tick(input, use_key);
        }
        assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(6), "plate #476 fired ch 6 [D placements]");
    }
    w.wait(60);
    // An awake walker: enabled, moving on its kind-8 route.
    let sim = w.scene.sim.as_mut().unwrap();
    let awake: Vec<usize> = sim.drones.list.iter().enumerate()
        .filter(|(_, d)| d.present && !d.corpse && d.enabled && d.switch_channel == 6 && d.state != state::WAIT_SWITCH && d.dtype == 10)
        .map(|(i, _)| i).collect();
    assert!(!awake.is_empty(), "the ch-6 walkers woke [M 0x1728c0/0x172730]");
    // The most-travelling walker over a window (deterministic pick).
    let pick = {
        let mut best = awake[0];
        let mut best_move = 0.0f32;
        for &i in &awake {
            let p0 = sim.drones.list[i].pos;
            for _ in 0..20 { sim.step_bond(Input::default(), false, false, false); }
            let p1 = sim.drones.list[i].pos;
            let m = dxz(p0, p1);
            if m > best_move { best_move = m; best = i; }
        }
        eprintln!("[guest-solid] walker pick {best} move {best_move:.2}");
        assert!(best_move > 0.05, "the picked walker is moving");
        best
    };
    // Stage Bond directly on the walker's path just ahead of it, then stand still.
    let g1 = sim.drones.list[pick].pos;
    for _ in 0..10 { sim.step_bond(Input::default(), false, false, false); }
    let g2 = sim.drones.list[pick].pos;
    let vel = [g2[0] - g1[0], g2[2] - g1[2]];
    let vl = (vel[0] * vel[0] + vel[1] * vel[1]).sqrt().max(1e-4);
    let ahead = [g2[0] + vel[0] / vl * 0.5, g2[1], g2[2] + vel[1] / vl * 0.5];
    sim.bond.pos = ahead;
    sim.bond.yaw = (g2[0] - ahead[0]).atan2(g2[2] - ahead[2]);
    let bond_stance = sim.bond.pos;
    let guest_start = sim.drones.list[pick].pos;
    let mut bond_max_push = 0.0f32;
    let mut min_dxz = f32::MAX;
    for t in 0..480u32 {
        sim.step_bond(Input::default(), false, false, false);
        bond_max_push = bond_max_push.max(dxz(bond_stance, sim.bond.pos));
        min_dxz = min_dxz.min(dxz(sim.bond.pos, sim.drones.list[pick].pos));
        if t % 120 == 0 {
            eprintln!("[guest-solid] t{t} bond {:?} (pushed {bond_max_push:.2}) guest {:?} dxz {:.2}",
                sim.bond.pos, sim.drones.list[pick].pos, dxz(sim.bond.pos, sim.drones.list[pick].pos));
        }
    }
    let guest_end = sim.drones.list[pick].pos;
    let guest_travel = dxz(guest_start, guest_end);
    eprintln!("[guest-solid] bond pushed {bond_max_push:.3} min dxz {min_dxz:.3} guest travel {guest_travel:.3}");
    assert!(min_dxz < 1.1, "the guest reached Bond's capsule (contact happened), min dxz {min_dxz:.3}");
    assert!(bond_max_push > 0.3, "Bond was displaced by the walking guest (the SP Jointy push [M 0x1ec180:229-233]), max {bond_max_push:.3}");
    assert!(guest_travel > 0.5, "the guest kept walking through the contact (bots never self-push vs class 3 [M 0x1ec180:231-233]), travel {guest_travel:.3}");
}

/// Sister: a guard still behaves as before — master's airborne-only push for the non-guest
/// bots is untouched, so a GROUNDED Bond walking into a bouncer meets no actor push.
/// [S] port choice pinned here (NOT game behaviour: the game's pass has no stance gate [M
/// Player_Collision 0x194df0:88] and a near guard's DoCollision is 1 through the near arm [M
/// 0x149ec8:23-26] — but +0x570&2/obj+0xfc can still turn a non-guest capsule OFF and are not
/// modelled). The extension to the other classes is the live-check item in the report.
#[test]
fn a_guard_still_behaves_as_before_grounded_bond_meets_no_push() {
    let mut w = load_party();
    w.wait_enabled(3000, "the party start");
    w.wait(60);
    let sim = w.scene.sim.as_mut().unwrap();
    for _ in 0..90 { sim.step_bond(Input::default(), false, false, false); }
    let guards: Vec<usize> = sim.drones.list.iter().enumerate()
        .filter(|(_, d)| d.present && !d.corpse && d.enabled && d.team == 1
            && matches!(d.state, state::CIVILIAN_GUARD | state::CIVILIAN_DOOR_GUARD | state::IDLE | state::CIVILIAN)
            && d.pos[1] > 42.0 && d.pos[1] < 45.0)
        .map(|(i, _)| i).collect();
    assert!(!guards.is_empty(), "no enabled bouncer near the party floor to pin the sister case");
    let pick = guards[0];
    let gp = sim.drones.list[pick].pos;
    eprintln!("[guest-solid] guard {pick} state {:#x} pos {:?}", sim.drones.list[pick].state, gp);
    let start = [gp[0] + 1.1, gp[1], gp[2]];
    sim.bond.pos = start;
    sim.bond.yaw = (gp[0] - start[0]).atan2(gp[2] - start[2]);
    let mut min_dist = f32::MAX;
    for _ in 0..150u32 {
        let p = sim.bond.pos;
        min_dist = min_dist.min(dxz(p, gp));
        let desired = (gp[0] - p[0]).atan2(gp[2] - p[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 1.0 { 0.0 } else { 1.0 }, ..Default::default() };
        sim.step_bond(input, false, false, false);
    }
    eprintln!("[guest-solid] guard min dist {min_dist:.3}");
    assert!(min_dist < 0.5, "grounded Bond meets NO actor push against a guard (master's port choice, unchanged), min dist {min_dist:.3}");
}
