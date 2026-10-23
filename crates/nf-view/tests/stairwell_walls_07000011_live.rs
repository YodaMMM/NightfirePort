//! stairwell-walls-11: the 07000011 stairwell guards stopping against walls that are not there.
//!
//! The port's guard walk (`Drone::move_by`, the anim root-motion step) is gated on two horizontal
//! `world.line_of_sight` probes (feet y and +0.5, static + missile-cage + dynamic sets). The
//! game's own guard movement has NO world-collision gate: the anim move applies root motion
//! directly (`DroneAnim_PreTransCallback` 0x13b490 — no Collide_ call in the decompile), and the
//! per-tick constraints are the AI-network boundary push-out (`AINetwork_GetBoundsPushVector-
//! ForSphere` 0x157bd0, run last in `NDrone2_Collision` 0x140e90:169-174) and the nav/portal
//! structure (the game's node scans have no collision-world line of sight — the port's own note
//! on `node_scan_ok`). So every triangle the port's flat probes hit — missile cages, doors,
//! panes — is a wall the game's guards walk through: the invisible walls.
//!
//! This test wakes the stairwell's 0x13 guards through the real plate (park Bond on TouchOnce
//! #253 [D]), lets them approach through the real step path, and reports every stalled guard
//! with the exact hit its move_by probes take (t, point, normal, surface byte, owning object
//! bounds) plus the nav push-out — the evidence for which triangle is the wall. The dump is the
//! assertion-free probe; the pinned test follows once the wall is identified.
//! Evidence tags: [M] decompile address, [D] disc bytes.

use nf_view::scene::Scene;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "assertion-free stall dump; run with --ignored"]
fn dump_stairwell_guard_stalls() {    let mut sc: Scene = nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 3).expect("load d3");
    {
        let sim = sc.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [22.34, 29.37, 3.52], 0.0);
        sim.bond.vel = [0.0; 3];
    }
    let mut woke = None;
    for t in 0..120 {
        sc.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if sc.sim.as_ref().unwrap().drones.channels.on(0x13) { woke = Some(t); break; }
    }
    eprintln!("[walls] ch 0x13 fired at tick {:?}", woke);
    let mut last: std::collections::HashMap<usize, [f32; 3]> = std::collections::HashMap::new();
    for t in 0..900 {
        sc.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if t % 60 != 0 { continue; }
        let sim = sc.sim.as_ref().unwrap();
        for (i, dr) in sim.drones.list.iter().enumerate() {
            if !dr.present || !dr.enabled { continue; }
            let prev = last.insert(i, dr.pos);
            let moved = prev.map(|p| ((p[0] - dr.pos[0]).powi(2) + (p[2] - dr.pos[2]).powi(2)).sqrt());
            eprintln!("[walls] t{t} #{:03} st {:#04x} pos ({:.2},{:.2},{:.2}) moved {:.3} yaw {:.2}",
                dr.placement, dr.state, dr.pos[0], dr.pos[1], dr.pos[2], moved.unwrap_or(0.0), dr.desired_yaw);
            let stalled = moved.map(|m| m < 0.02).unwrap_or(false);
            if stalled && t > 120 {
                let world = &sim.world;
                let to_bond = sim.bond.pos;
                let dirs: [(&str, [f32; 2]); 2] = [
                    ("toBond", {
                        let d = [to_bond[0] - dr.pos[0], to_bond[2] - dr.pos[2]];
                        let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
                        if l < 1e-4 { [1.0, 0.0] } else { [d[0] / l, d[1] / l] }
                    }),
                    ("toYaw", [nf_game::trig::sinf(dr.desired_yaw), nf_game::trig::cosf(dr.desired_yaw)]),
                ];
                for (name, d2) in dirs {
                    for (label, y) in [("feet", dr.pos[1]), ("mid", dr.pos[1] + 0.5)] {
                        let a = [dr.pos[0], y, dr.pos[2]];
                        let b = [dr.pos[0] + d2[0] * 2.4, y, dr.pos[2] + d2[1] * 2.4];
                        match world.segment_surface(a, b, true) {
                            Some((tt, n, s)) => {
                                let p = [a[0] + (b[0] - a[0]) * tt, a[1] + (b[1] - a[1]) * tt, a[2] + (b[2] - a[2]) * tt];
                                let objs = world.objects_with_bound(p);
                                eprintln!("[walls]   #{:03} {} {} HIT t{:.3} surf {:#04x} n ({:.2},{:.2},{:.2}) at ({:.2},{:.2},{:.2}) bounds {:?}",
                                    dr.placement, name, label, tt, s, n[0], n[1], n[2], p[0], p[1], p[2],
                                    objs.iter().take(3).collect::<Vec<_>>());
                            }
                            None => eprintln!("[walls]   #{:03} {} {} clear", dr.placement, name, label),
                        }
                    }
                }
                let feet = [dr.pos[0], dr.pos[1] - 1.0, dr.pos[2]];
                let pushed = sim.nav.push_out(feet, 0.4);
                eprintln!("[walls]   #{:03} push_out delta ({:.3},{:.3})", dr.placement, pushed[0] - feet[0], pushed[2] - feet[2]);
            }
        }
    }
}

/// The pinned class test, at the exact stall the probe caught (guard #179 ran 14.45 → 17.09 and
/// stopped; the toBond probe hit (18.04,32.14,−1.84) surf 0x40 [D the walls dump]): the pane's
/// surface byte is the 0x40 pass flag, the game's 0xa27 LOS class passes it [M Collide_Intersect
/// 0x1ebaa8:301 — the port's `segment_masked`], and the raw ray — no game query class — took it
/// as a wall. 12 cm behind the pane sits plain 0x06 concrete (the flight-C structure [D seg walk]):
/// concrete still blocks, so the pane alone is no longer the guard's wall.
#[test]
fn the_pass_class_pane_is_no_wall_but_the_concrete_behind_it_still_blocks() {
    let sc: Scene = nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 3).expect("load d3");
    let sim = sc.sim.as_ref().unwrap();
    let world = &sim.world;
    // The move_by probe shape: 2.4 m toward Bond from the stall spot (the dump's probe).
    let from = [17.09f32, 32.14, -3.01];
    let d = [22.34f32 - from[0], 3.52f32 - from[2]];
    let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let to = [from[0] + d[0] / l * 2.4, 32.14, from[2] + d[1] / l * 2.4];
    let (t_raw, _, surf_raw) = world.segment_surface(from, to, true).expect("the raw ray hits the pane");
    assert_eq!(surf_raw, 0x40, "the stall surface's byte is the 0x40 pass flag [D collision materials, the walls dump]");
    let t_masked = world.segment_masked(from, to).expect("the concrete 0.12 m behind the pane still blocks [D seg walk]");
    assert!(t_masked > t_raw,
        "the masked class passes the 0x40 pane [M 0x1ebaa8:301] and blocks on the concrete behind it: raw t{t_raw:.3}, masked t{t_masked:.3}");
    // And the wake approach itself runs through the real path: the guards leave their placements
    // and cover metres before any wall matters (the pre-stall approach the raw-ray bug also gated).
    let mut sc2: Scene = nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 3).expect("load d3");
    {
        let sim = sc2.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [22.34, 29.37, 3.52], 0.0);
        sim.bond.vel = [0.0; 3];
    }
    for _ in 0..120 {
        sc2.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if sc2.sim.as_ref().unwrap().drones.channels.on(0x13) { break; }
    }
    assert!(sc2.sim.as_ref().unwrap().drones.channels.on(0x13), "the 0x13 plate #253 fires as Bond stands on it [D #253 out 0x13]");
    let mut furthest = 0.0f32;
    for _ in 0..300 {
        sc2.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = sc2.sim.as_ref().unwrap();
        if let Some(dr) = sim.drones.list.iter().find(|d| d.placement == 179 && d.present && d.enabled) {
            furthest = furthest.max(dr.pos[0]);
        }
    }
    assert!(furthest > 15.5, "guard #179 leaves its placement west strip and runs east (reached x {furthest:.2}) [D placement, the walls dump approach]");
}

/// Sister: the drops remove only the game's pass class — the concrete office wall (surf 0x15, the
/// #132 guard's own wall at x −14.98 [D probe]) still blocks both rays, so the fix cannot be read
/// as "guards walk through walls".
#[test]
fn the_concrete_walls_still_block_the_guard_step() {
    let sc: Scene = nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 3).expect("load d3");
    let sim = sc.sim.as_ref().unwrap();
    let world = &sim.world;
    // The #132 guard stands at (−16.07,47.78,12.01); its wall sits 1.09 m east (the probe hit at
    // (−14.98,47.78,11.78) surf 0x15, normal (−1,0,0) [D probe, walls dump]).
    let from = [-16.07f32, 47.78, 11.78];
    let to = [-14.0f32, 47.78, 11.78];
    let (_, _, surf) = world.segment_surface(from, to, true).expect("the raw ray hits the office wall");
    assert_eq!(surf, 0x15, "the office wall's surface byte is plain material 0x15 [D collision materials]");
    assert!(world.segment_masked(from, to).is_some(),
        "the game's 0xa27 LOS class still takes the concrete wall [M 0x1ebaa8 — only 0x40/0xD/0xE/0x10 drop]");
}
