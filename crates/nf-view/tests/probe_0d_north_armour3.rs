//! probe 0700000d's north-armour APPROACH LANES (retune-nb-0d-20, v3): v2 measured the
//! pocket touch GOOD (settled 1.58 from the centre, inside the sphere — armour 0 -> 50 [L])
//! but the direct north approach walled at z -208.95 for x 202-208.5 [L]. The exits north
//! (z -202) and east (x 208.7) are open [L v2]. This probe tests the AROUND paths from the
//! route's own band: the east lane (x 209-218) north, the far-east lane via the #374 ladder
//! base, and the west lane (x 196-201). Teleport probe (assertion-free diagnostic; never
//! runs in CI) — run --ignored --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn bond_pos(w: &Walker) -> [f32; 3] { w.scene.sim.as_ref().unwrap().bond.pos }

fn snap(w: &mut Walker, ticks: u32) {
    for _ in 0..ticks { w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); w.total_ticks += 1; }
}

fn walk_chain(w: &mut Walker, legs: &[(&str, [f32; 3], f32)]) {
    w.lenient = true;
    for (label, at, r) in legs {
        let before = bond_pos(w);
        let ok = w.try_walk_to(Wp::new(at[0], at[1], at[2]).r(*r), label);
        snap(w, 25);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[leg] {label}: {before:?} -> {:?} ok {ok} grounded {} armour {:.1}",
            bond_pos(w), sim.bond.grounded, sim.drones.health.armour);
        if !sim.bond.grounded { eprintln!("[leg] AIRBORNE — stop"); break; }
    }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe_0d_north_armour3() {
    let iso = iso();
    // d2 = the Agent difficulty (the 00 Agent twin refused the take [L the 0d run3 note];
    // d1 measured the take GOOD [L 0d-4] and load_with_carry(None) defaults to 1 — so the
    // earlier variants rode d1. THIS probe pins the take question at d2.)
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "0700000d.bin", false, None, 2).expect("load d2"));
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.difficulty, 2, "the probe runs d2");
    w.wait_enabled(3000, "the intro");
    w.wait(30);

    // A: the EAST lane — from the band at the ladder #374 east, north to z -204, west in.
    eprintln!("=== A. the east lane (x ~213) north ===");
    {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [196.0, 0.5, -228.5];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(30);
        walk_chain(&mut w, &[
            ("east to the #374 base", [213.5, 0.5, -227.5], 1.4),
            ("north on the east lane", [213.5, 1.03, -210.0], 1.5),
            ("north past the wall line", [213.5, 1.03, -204.0], 1.5),
            ("west into the pocket", [205.41, 1.18, -205.31], 1.3),
        ]);
    }

    // B: the FAR-EAST lane — hug x 210 first (the wall's east end at ~208.5?).
    eprintln!("=== B. the x-210 lane ===");
    {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [196.0, 0.5, -228.5];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(30);
        walk_chain(&mut w, &[
            ("east on the band", [209.0, 0.5, -227.5], 1.2),
            ("north at x 209.5", [209.5, 1.03, -212.0], 1.4),
            ("north past the wall line", [209.8, 1.03, -205.5], 1.4),
            ("west into the pocket", [205.41, 1.18, -205.31], 1.3),
        ]);
    }

    // C: the WEST lane — from the band west to x ~199, north, east in.
    eprintln!("=== C. the west lane (x ~199) ===");
    {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [196.0, 0.5, -228.5];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(30);
        walk_chain(&mut w, &[
            ("west on the band", [199.0, 0.5, -227.5], 1.2),
            ("north at x 199", [199.0, 1.03, -212.0], 1.4),
            ("north past the wall line", [199.5, 1.03, -205.0], 1.4),
            ("east into the pocket", [205.41, 1.18, -205.31], 1.3),
        ]);
    }

    // D: the band -> pocket -> back OUT to the band (the full detour's return leg): from the
    // pocket, back south the way the exits showed (north then east then south?) — map the
    // return path explicitly.
    eprintln!("=== D. the return: pocket -> band ===");
    {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [205.41, 1.18, -205.31];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(30);
        walk_chain(&mut w, &[
            ("east out", [209.5, 1.03, -204.5], 1.2),
            ("south on the east lane", [209.8, 1.03, -212.0], 1.4),
            ("south to the band", [209.8, 0.5, -227.0], 1.4),
            ("west to the strip", [196.0, 0.5, -228.5], 1.4),
        ]);
    }
}
