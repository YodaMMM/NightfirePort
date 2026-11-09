//! batch-1109a-fix: a guard reaches a swing door's anti-crush only through his collide capsule
//! touching the leaf.
//!
//! The guard's collide HITTEST is mask 0x800 [M NDrone2_Collision 0x140e90:91-121 writes
//! +0x464 = 0x800, A = pos + up·0.4, B = pos + up·(0.4 − 1.0), r 0.4], and `Collide_Intersect`
//! 0x1ebaa8 picks the `Intersect_CylGeom` refine by that mask alone [M 0x1ebaa8:234-296], so a
//! guard lands on the door's `+0xD0` list — and `Door_ReactToHit` 0x135980 reverses an opening
//! leaf — only once the swinging leaf meets his capsule, as for the player.
//!
//! [L 2026-11-09, PCSX2, 07000008, ch 137 + 0x2a raised, Bond (156.3, 43.03, −120.7) facing
//! the GuardRoomDoor #10 (154.516, 41.999, −121.328), the nearest woken guard put at (153.97,
//! 43.0, −121.05), R2]: the leaf opened 0.5 → 0.4138 (five steps of 0.01724) before reversing to
//! CLOSING, in 3 of 4 trials with the guard standing (the fourth: 0.431, the guard strafing off).
//! It never reached OPEN while he stood there. The old stand-in (the guard's root point at the
//! sphere stage, no leaf refine) reversed it on the first step, 0.4828.
//!
//! Opt-in via `NF_ISO`.

fn tick(sim: &mut nf_view::scene::Sim, n: u32) {
    for _ in 0..n { sim.step_bond(Default::default(), false, false, false); }
}

#[test]
fn a_guard_beside_the_hinge_reverses_the_leaf_only_when_it_meets_him_07000008() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000008.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    let di = sim.drones.doors.list.iter()
        .position(|d| (d.pos[0] - 154.516).abs() < 0.2 && (d.pos[2] + 121.328).abs() < 0.2 && d.kind == 0)
        .expect("the GuardRoomDoor swing door #10 [D]");
    // The lock (ch 137, the fusebox) and the walkway plates (ch 0x2a) — the live recipe's pokes.
    let f = sim.drones.frame;
    sim.drones.channels.set(137, 1, f);
    sim.drones.channels.set(0x2a, 1, f);
    let stand = [156.3f32, 43.03, -120.7];
    sim.bond.pos = stand;
    sim.bond.vel = [0.0; 3];
    sim.bond.yaw = -std::f32::consts::FRAC_PI_2;
    sim.bond.pitch = 0.0;
    tick(sim, 30);
    // The live run found the leaf open after the pokes too; E closes it (the same press).
    for _ in 0..3 {
        if sim.drones.doors.list[di].state == 2 { break; }
        sim.step_bond(Default::default(), false, false, true);
        tick(sim, 60);
    }
    assert_eq!(sim.drones.doors.list[di].state, 2, "the door starts closed");
    let gi = (0..sim.drones.list.len())
        .filter(|&i| sim.drones.list[i].present && !sim.drones.list[i].corpse)
        .min_by(|&a, &b| {
            let d = |i: usize| { let p = sim.drones.list[i].pos; (p[0] - 151.77).powi(2) + (p[2] + 120.72).powi(2) };
            d(a).total_cmp(&d(b))
        })
        .expect("the guard west of the door [L 0x17dbb00 at (151.77, 42.99, −120.72)]");
    let g = &mut sim.drones.list[gi];
    g.pos = [153.97, 43.0, -121.05];
    g.enabled = true;
    g.health = g.health.max(1.0);
    let mut lowest = 1.0f32;
    let mut reversed = false;
    let mut opened = false;
    let mut turn = 1.0f32;
    for t in 0..60 {
        // Hold the guard where the live check stood him (his own move is not the question here).
        sim.drones.list[gi].pos = [153.97, 43.0, -121.05];
        sim.bond.pos = stand;
        sim.bond.yaw = -std::f32::consts::FRAC_PI_2;
        sim.bond.pitch = 0.0;
        sim.step_bond(Default::default(), false, false, t < 2);
        let d = &sim.drones.doors.list[di];
        if d.state & 8 != 0 { lowest = lowest.min(d.pos01); }
        if d.state & 0x10 != 0 && lowest < 0.5 { reversed = true; turn = d.pos01; }
        if d.state == 4 { opened = true; }
        eprintln!("[leaf] t{t} state {:#x} pos01 {:.4}", d.state, d.pos01);
        if reversed { break; }
    }
    assert!(lowest < 0.5, "Bond's E starts the swing (state 8)");
    assert!(reversed && !opened, "the leaf reverses on the guard and never opens past him [L]");
    assert!((turn - 0.4138).abs() < 0.005,
        "the leaf reverses where it meets the guard's capsule: the game's first CLOSING pos01 was 0.4138 [L, 3 of 4 trials]; here {turn:.4} (the root-point stand-in reversed on the first step)");
}
