//! gt-los-live: a wall gun must not track a target it cannot see.
//!
//! `GT_Track` 0x17f9c8 runs one `Collide_LineOfSight(from, cand+0x80, level, obj, cand, 0x72f)`
//! per scanned candidate [M 0x17f9c8:258,271-278] — a candidate behind geometry is untrackable.
//! The port supplied `los_ok = true` for every candidate (the parked "no collide ray for
//! turrets" gap): this test pins the real ray on 0700000a's wall gun #636
//! (facing 1, enable ch 18, scripts 0x06000041/0x06000042 [D placements dump]) —
//! a spot in the gun's front cone but behind a wall must hold the gun DORMANT (no deploy, no
//! shots); the open front spot must still deploy it into COMBAT (the pre-existing real path).
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere.

use nf_game::player::Input;

fn settle(scene: &mut nf_view::scene::Scene, cap: u32) {
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..cap {
        sim.step_bond(Default::default(), false, false, false);
        if !sim.disabled { break; }
    }
}

fn wall_gun(sim: &nf_view::scene::Sim) -> usize {
    sim.drones.gt_turrets.iter().position(|t| t.param4 == 18 && t.facing == 1)
        .expect("the fixture wall gun (ch 18, facing 1)")
}

/// The candidate sampling grid. The wall guns' create-time fire direction (ext+0x80) is
/// **straight down** — facing 1 negates the placement's up row [M 0017ffa0:141-147; D the
/// placements' identity-rot matrices] — so the front cone is the deck below the mount. The
/// clear spot is the proven open one (directly beneath the gun — the pre-existing live test
/// fires from there); the blocked spot is the first sampled deck point whose view ray hits
/// the bridge structure (the x ≈ −10 region is walled off from gun #636 [D the level]).
fn sample_spots(sim: &nf_view::scene::Sim, gun: usize) -> ([f32; 3], [f32; 3]) {
    let t = &sim.drones.gt_turrets[gun];
    let (p, fd) = (t.pos, t.fire_dir);
    let clear = [p[0], 0.0, p[2]];
    let mut blocked = None;
    'outer: for ox in [-20.0f32, -14.0, -8.0, -3.0, 3.0, 8.0, 14.0, 20.0] {
        for oz in [-14.0f32, -6.0, 0.0, 6.0, 14.0] {
            for y in [0.0f32, -4.0, -8.0] {
                let q = [p[0] + ox, y, p[2] + oz];
                let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
                let m = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                if m < 3.0 || m > 45.0 { continue; }
                let dot = fd[0] * d[0] / m + fd[1] * d[1] / m + fd[2] * d[2] / m;
                if dot < 0.05 { continue; } // inside the front-cone gate [M 0x17f9c8:252-257]
                if !sim.world.line_of_sight(p, q, nf_game::world::LOS_TURRET_TRACK) {
                    blocked = Some(q);
                    break 'outer;
                }
            }
        }
    }
    (blocked.expect("a blocked in-cone deck spot exists on the sky bridge — adjust the grid"), clear)
}

#[test]
fn a_wall_between_the_gun_and_bond_holds_the_gun_dormant() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso, "0700000a.bin", false).expect("load 0700000a");
    settle(&mut scene, 3000);
    let (blocked, clear) = {
        let sim = scene.sim.as_ref().unwrap();
        sample_spots(sim, wall_gun(sim))
    };
    let sim = scene.sim.as_mut().unwrap();
    // The real channel path (the game's switch/script write — `switch_channels[18] = 1`).
    let frame = sim.drones.frame;
    sim.drones.channels.set(18, 1, frame);
    // Bond behind the wall, inside the gun's front cone: GT_Track's per-candidate
    // `Collide_LineOfSight` rejects him, the list is empty, the gun stays DORMANT
    // [M 0x180498:205-230 — the case-0 gate needs a target to arm].
    sim.bond.pos = blocked;
    for _ in 0..900 { sim.step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    let ti = wall_gun(sim);
    assert_eq!(sim.drones.gt_turrets[ti].state, nf_game::gt::gt_state::DORMANT,
        "a wall-blocked candidate is untrackable [M 0x17f9c8:271-278] — the gun must not deploy");
    assert_eq!(sim.drones.gt_turrets[ti].hp, 0xffff, "no deploy → no shots → the 0xffff HP selector untouched");
    let _ = clear; // used by the sister test below
}

#[test]
fn the_open_front_spot_still_deploys_the_gun() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso, "0700000a.bin", false).expect("load 0700000a");
    settle(&mut scene, 3000);
    let (_, clear) = {
        let sim = scene.sim.as_ref().unwrap();
        sample_spots(sim, wall_gun(sim))
    };
    let sim = scene.sim.as_mut().unwrap();
    let frame = sim.drones.frame;
    sim.drones.channels.set(18, 1, frame);
    sim.bond.pos = clear;
    for _ in 0..900 { sim.step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    let ti = wall_gun(sim);
    assert_eq!(sim.drones.gt_turrets[ti].state, nf_game::gt::gt_state::COMBAT,
        "the LOS-clear candidate arms through DORMANT→ARMING→COMBAT [M 0x180498:205-230]");
}
