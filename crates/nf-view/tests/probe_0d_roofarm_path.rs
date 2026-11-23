//! probe 0700000d's west-roof ARMOUR ASCENT (retune-nb-0d-20): the 18-12 scan's fourth body
//! armour (147.0, 21.76, -252.77 [L the scan]) sits on the west-roof structure (the settled
//! guards' own floor, their origin y 22.7 [L probe_0d_roof_rays census]) ~15 u north-west of
//! the second ladder's roof top-out (155ish, 19.65, -267.5 [L the walk's own assert]). The
//! round-19 grab measured DEAD as wired — `walk_nav_to` (the nav web) walked off the roof edge
//! [L 19-6]. This probe maps, from the MAIN roof (18.58):
//!   1. vertical surface columns along the corridor between the top-out and the armour
//!      (the stepped ascent question: 18.58 -> 20.19 -> 21.6?);
//!   2. the pickups near (147.0, -252.77) (the armour pickup's own matrix);
//!   3. a teleport ONTO the west-roof floor + a real walk INTO the armour spot (does the
//!      touch take at all from that floor?);
//!   4. staged real walk legs from the top-out toward the armour (how far does the walk_to
//!      class get, where does it stall/fall).
//! Teleport probe (assertion-free diagnostic; never runs in CI) — run --ignored --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const ARMOUR: [f32; 3] = [147.0, 21.76, -252.77];

fn bond_pos(w: &Walker) -> [f32; 3] { w.scene.sim.as_ref().unwrap().bond.pos }

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe_0d_roofarm_path() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(30);

    // 1. Surface columns along the ascent corridor: x 146..158, z -268..-250.
    eprintln!("=== 1. surface columns (x, z) -> y surfaces 17..24 ===");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for x in [146.0f32, 148.0, 150.0, 152.0, 154.0, 156.0, 158.0] {
            for z in [-268.0f32, -264.0, -260.0, -256.0, -252.77, -248.0, -244.0] {
                let mut hits = Vec::new();
                let mut bot = 17.0f32;
                for _ in 0..10 {
                    match sim.world.segment_hit([x, bot, z], [x, 24.0, z]) {
                        Some((f, _)) => { let y = bot + (24.0 - bot) * f; hits.push(y); bot = y + 0.05; }
                        None => break,
                    }
                }
                if !hits.is_empty() {
                    eprintln!("[col] x {x} z {z}: {:?}", hits.iter().map(|y| format!("{y:.2}")).collect::<Vec<_>>());
                }
            }
        }
        // Diagonals across the corridor: the straight top-out -> armour line.
        for t in [0.0f32, 0.15, 0.3, 0.45, 0.6, 0.75, 0.9, 1.0] {
            let x = 155.5 + (147.0 - 155.5) * t;
            let z = -267.5 + (-252.77 - -267.5) * t;
            let mut hits = Vec::new();
            let mut bot = 17.0f32;
            for _ in 0..10 {
                match sim.world.segment_hit([x, bot, z], [x, 24.0, z]) {
                    Some((f, _)) => { let y = bot + (24.0 - bot) * f; hits.push(y); bot = y + 0.05; }
                    None => break,
                }
            }
            eprintln!("[diag] t {t:.2} ({x:.1},{z:.1}): {:?}", hits.iter().map(|y| format!("{y:.2}")).collect::<Vec<_>>());
        }
    }

    // 2. The pickups near the armour spot.
    eprintln!("=== 2. pickups near (147.0, -252.77) ===");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for (k, p) in sim.drones.pickups.iter().enumerate() {
            let (x, y, z) = (p.m[3][0], p.m[3][1], p.m[3][2]);
            if (x - 147.0).abs() < 8.0 && (z + 252.77).abs() < 8.0 {
                eprintln!("[pickup] #{k} at ({x:.2},{y:.2},{z:.2}) gone {}", p.gone);
            }
        }
    }

    // 3. Teleport ONTO the west-roof floor (just north-east of the armour spot) and walk
    //    INTO the armour spot: does the touch take from that floor at all?
    eprintln!("=== 3. the touch test from the west-roof floor ===");
    {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [149.0, 22.7, -252.8];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(30);
        eprintln!("[touch] settled at {:?}", bond_pos(&w));
        let armour0 = w.scene.sim.as_ref().unwrap().drones.health.armour;
        let _ = w.try_walk_to(Wp::new(ARMOUR[0], ARMOUR[1], ARMOUR[2]).r(1.3), "the armour spot from the floor");
        for _ in 0..40 { w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); w.total_ticks += 1; }
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[touch] after: armour {armour0:.1} -> {:.1} at {:?}", sim.drones.health.armour, bond_pos(&w));
    }

    // 4. Staged real legs from the main roof's top-out toward the armour. The dismount spot
    //    per the walk's assert: x 153..158, y > 19 (19.65). The legs use the route's own
    //    walk_to class, generous arrival radii, lenient (a stall is a measured answer).
    eprintln!("=== 4. staged walk legs from the top-out ===");
    {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [155.5, 19.65, -267.5];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(30);
        w.lenient = true;
        let legs: &[(&str, [f32; 3], f32)] = &[
            ("north to the shed corner", [152.0, 19.65, -262.0], 1.5),
            ("north-west mid 1", [150.0, 20.7, -257.0], 1.5),
            ("north-west mid 2", [148.5, 21.76, -254.5], 1.5),
            ("the armour spot", ARMOUR, 1.3),
        ];
        for (label, at, r) in legs {
            let before = bond_pos(&w);
            let ok = w.try_walk_to(Wp::new(at[0], at[1], at[2]).r(*r), label);
            for _ in 0..40 { w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); w.total_ticks += 1; }
            eprintln!("[leg] {label}: from {before:?} -> {:?} ok {ok} armour {:.1} grounded {}",
                bond_pos(&w), w.scene.sim.as_ref().unwrap().drones.health.armour,
                w.scene.sim.as_ref().unwrap().bond.grounded);
            if !w.scene.sim.as_ref().unwrap().bond.grounded { break; }
        }
        eprintln!("[leg] final: {:?} armour {:.1}", bond_pos(&w), w.scene.sim.as_ref().unwrap().drones.health.armour);
    }

    // 5. A second staged variant: hug the WEST rim (the slab-edge law — the settled guards
    //    walk x ~151, the armour x 147) from the roof's north-west corner.
    eprintln!("=== 5. staged walk legs, the west-hug variant ===");
    {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [155.5, 19.65, -267.5];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(30);
        w.lenient = true;
        let legs: &[(&str, [f32; 3], f32)] = &[
            ("west along the roof edge", [150.5, 19.65, -267.5], 1.5),
            ("north-west up 1", [149.5, 20.7, -262.0], 1.5),
            ("north-west up 2", [148.0, 21.76, -257.5], 1.5),
            ("the armour spot", ARMOUR, 1.3),
        ];
        for (label, at, r) in legs {
            let before = bond_pos(&w);
            let ok = w.try_walk_to(Wp::new(at[0], at[1], at[2]).r(*r), label);
            for _ in 0..40 { w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); w.total_ticks += 1; }
            eprintln!("[leg] {label}: from {before:?} -> {:?} ok {ok} armour {:.1} grounded {}",
                bond_pos(&w), w.scene.sim.as_ref().unwrap().drones.health.armour,
                w.scene.sim.as_ref().unwrap().bond.grounded);
            if !w.scene.sim.as_ref().unwrap().bond.grounded { break; }
        }
        eprintln!("[leg] final: {:?} armour {:.1}", bond_pos(&w), w.scene.sim.as_ref().unwrap().drones.health.armour);
    }
}
