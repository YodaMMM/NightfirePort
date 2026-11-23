//! probe 0700000d's NORTH-EAST ground armour (retune-nb-0d-20): the 18-12 scan's third
//! armour (205.41, 0.11, -205.31 [L the scan]) — "far north", ground level, ~23 u north of
//! the route's own ground band (the strip legs (188/196/212, 0.5, -228..-230) [M the walk])
//! and directly under the warehouse start. The committed route never walks there; the detour
//! would ride the pre-wake/ground-band window. This probe walks real legs from the band to
//! the armour spot and tests the touch. Teleport probe (assertion-free diagnostic; never
//! runs in CI) — run --ignored --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const ARMOUR: [f32; 3] = [205.41, 1.18, -205.31];

fn bond_pos(w: &Walker) -> [f32; 3] { w.scene.sim.as_ref().unwrap().bond.pos }

fn snap(w: &mut Walker, ticks: u32) {
    for _ in 0..ticks { w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); w.total_ticks += 1; }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe_0d_north_armour() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(30);

    // The pickups near the spot (which pickup is it, is it present).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for (k, p) in sim.drones.pickups.iter().enumerate() {
            let (x, y, z) = (p.m[3][0], p.m[3][1], p.m[3][2]);
            if (x - 205.41).abs() < 6.0 && (z + 205.31).abs() < 6.0 {
                eprintln!("[pickup] #{k} at ({x:.2},{y:.2},{z:.2}) gone {}", p.gone);
            }
        }
    }

    // A: walk north from the band at x ~205 (the band at z -228.5 -> the armour z -205.31).
    eprintln!("=== A. from the band north at x ~205 ===");
    {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [196.0, 0.5, -228.5];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(30);
        w.lenient = true;
        let legs: &[(&str, [f32; 3], f32)] = &[
            ("east on the band", [205.0, 0.5, -227.0], 1.2),
            ("north 1", [205.4, 0.5, -218.0], 1.5),
            ("north 2", [205.4, 0.5, -211.0], 1.5),
            ("the armour spot", ARMOUR, 1.3),
        ];
        for (label, at, r) in legs {
            let before = bond_pos(&w);
            let ok = w.try_walk_to(Wp::new(at[0], at[1], at[2]).r(*r), label);
            snap(&mut w, 30);
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[leg] {label}: {before:?} -> {:?} ok {ok} grounded {} armour {:.1}",
                bond_pos(&w), sim.bond.grounded, sim.drones.health.armour);
            if !sim.bond.grounded { eprintln!("[leg] AIRBORNE — stopping"); break; }
        }
    }

    // B: the same walk with a wake — stand at the drop point first (the west-roof wake),
    // THEN walk the detour (the real run's order: the drop wakes the roof, the band walk
    // follows, the detour would ride after). This variant measures the exposure-level
    // geometry only (the guards fire; top-ups keep Bond alive).
    eprintln!("=== B. wake at the drop point, then the band north ===");
    {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [181.0, 7.5, -227.5];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(90);
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [196.0, 0.5, -228.5];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(30);
        w.lenient = true;
        let legs: &[(&str, [f32; 3], f32)] = &[
            ("east on the band", [205.0, 0.5, -227.0], 1.2),
            ("north 1", [205.4, 0.5, -218.0], 1.5),
            ("north 2", [205.4, 0.5, -211.0], 1.5),
            ("the armour spot", ARMOUR, 1.3),
        ];
        for (label, at, r) in legs {
            let before = bond_pos(&w);
            let ok = w.try_walk_to(Wp::new(at[0], at[1], at[2]).r(*r), label);
            snap(&mut w, 30);
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[leg] {label}: {before:?} -> {:?} ok {ok} grounded {} armour {:.1} health {:.1}",
                bond_pos(&w), sim.bond.grounded, sim.drones.health.armour, sim.drones.health.health);
            if !sim.bond.grounded { eprintln!("[leg] AIRBORNE — stopping"); break; }
        }
        // Guard census: who fired during the walk (the damage log).
        eprintln!("[end] min health seen {:.1}", w.dmg.min_health);
    }

    // C: the touch test — teleport right next to the armour and walk in.
    eprintln!("=== C. the touch test ===");
    {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [205.41, 1.18, -208.0];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(30);
        let armour0 = w.scene.sim.as_ref().unwrap().drones.health.armour;
        let ok = w.try_walk_to(Wp::new(ARMOUR[0], ARMOUR[1], ARMOUR[2]).r(1.3), "the armour touch");
        snap(&mut w, 40);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[touch] ok {ok}: armour {armour0:.1} -> {:.1} at {:?}", sim.drones.health.armour, bond_pos(&w));
    }
}
