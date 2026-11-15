//! probe (assertion-free, `--ignored`): the level-0d-operative-walk-10 #27 ground-line scan.
//! The trench-a ground answer killed #29 pre-climb at zero incoming (the run-1 ledger), but
//! #27 at her south spawn (151.35,−255.80) delivered 3 × 16.0 and no route stand's ground ray
//! clears to her (probe_0d_ground_wake: all 14 stands blocked to g27, 0.83-0.90). The main
//! agent's 11-14 scan counted 63 clear spots to #27 in (186..216, −240..−222) — this probe
//! walks that grid at 2 u spacing against the SETTLED live roof guards (wake + 700-tick
//! settle), printing every spot whose eye→chest ray is CLEAR to any roof guard, with the
//! incoming live-shooter lines per spot (the exposure check). Run: NF_ISO=… cargo test -p
//! nf-view --test probe_0d_ground_wake scan27 -- --ignored --nocapture

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn scan27() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    w.wait_enabled(3000, "the 0d intro");
    w.wait(60);

    // THE WAKE: the west roof wakes only when Bond stands at the drop point
    // [probe_0d_platform part E].
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [181.0, 7.5, -227.5];
        sim.bond.vel = [0.0; 3];
        sim.bond.substate = 0;
    }
    for _ in 0..8 { w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    w.wait(700);

    for round in 0..2 {
        let guards: Vec<(usize, [f32; 3], u32, bool)> = {
            let sim = w.scene.sim.as_ref().unwrap();
            let mut v = Vec::new();
            for (i, d) in sim.drones.list.iter().enumerate() {
                if d.corpse || d.health <= 0.0 || !d.present { continue; }
                if !(140.0..=230.0).contains(&d.pos[0]) || !(-300.0..=-215.0).contains(&d.pos[2]) { continue; }
                v.push((i, d.pos, d.skin, d.enabled));
            }
            v
        };
        let roofs: Vec<usize> = guards.iter().filter(|(i, _, _, _)| {
            w.scene.sim.as_ref().unwrap().drones.list[*i].pos[1] > 18.0
        }).map(|(i, _, _, _)| *i).collect();
        eprintln!("== round {round} roofs ==");
        for gi in &roofs {
            let d = &w.scene.sim.as_ref().unwrap().drones.list[*gi];
            eprintln!("[roof] g{gi} skin {:#x} pos ({:.2},{:.2},{:.2}) health {:.0} state {:#x}",
                d.skin, d.pos[0], d.pos[1], d.pos[2], d.health, d.state);
        }
        // THE GRID: the main agent's scan region, 2 u spacing.
        let mut x = 186.0f32;
        while x <= 216.0 {
            let mut z = -240.0f32;
            while z <= -222.0 {
                {
                    let sim = w.scene.sim.as_mut().unwrap();
                    sim.bond.pos = [x, 0.5, z];
                    sim.bond.vel = [0.0; 3];
                    sim.bond.substate = 0;
                }
                for _ in 0..8 { w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                if (bp[0] - x).abs() > 1.0 || (bp[2] - z).abs() > 1.0 { z += 2.0; continue; } // not walkable/settled here
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                let mut out = Vec::new();
                let mut bad_in = Vec::new();
                for (gi, gp, _, gen) in &guards {
                    let chest = [gp[0], gp[1] + 0.3, gp[2]];
                    let bullet = sim.world.segment_bullet_scoped(sim.world.find_cel(eye), eye, chest);
                    if roofs.contains(gi) {
                        if bullet.is_none() { out.push(format!("g{gi}")); }
                    } else if bullet.is_none() {
                        // a non-roof guard who sees this spot (incoming if enabled)
                        bad_in.push(format!("g{gi}:en{gen}"));
                    }
                }
                if !out.is_empty() {
                    eprintln!("[spot] ({x:.0},{z:.0}) OUT-roofs [{}] incoming-clear [{}]", out.join(" "), bad_in.join(" "));
                }
                z += 2.0;
            }
            x += 2.0;
        }
        if round == 0 { w.wait(400); }
    }
    eprintln!("== done ==");
}
