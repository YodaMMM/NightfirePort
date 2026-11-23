//! probe 0700000d's west-roof ARMOUR ASCENT v2 (retune-nb-0d-20): v1's dynamic facts — the
//! walk_to class CANNOT go north from the top-out diagonally (fell to 7.77 [L v1]) and the
//! north-edge hug stalls at z −266.8 [L v1] — but v1's columns show a surface ladder along
//! x ~154: 19.95 (z −268) -> 21.29 (z −264..−252.77) -> the guards' floor ~21.66 (the
//! guards' origin 22.70 [L the wake census]; the touch takes 0 -> 50 from it [L v1]). This
//! probe walks FINE STAGED LEGS hugging x ~154 (the walker walks straight lines, so each
//! leg's line stays on the band) and then steps west onto the guards' floor to the armour.
//! Teleport probe (assertion-free diagnostic; never runs in CI) — run --ignored --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const ARMOUR: [f32; 3] = [147.0, 22.73, -252.77];

fn bond_pos(w: &Walker) -> [f32; 3] { w.scene.sim.as_ref().unwrap().bond.pos }

fn snap(w: &mut Walker, ticks: u32) {
    for _ in 0..ticks { w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); w.total_ticks += 1; }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe_0d_roofarm_path2() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(30);

    // Variant A: fine legs hugging x ~154 from the top-out.
    eprintln!("=== A. fine staged legs along x ~154 ===");
    {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [155.5, 19.65, -267.5];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(30);
        w.lenient = true;
        let legs: &[(&str, [f32; 3], f32)] = &[
            ("west to the band mouth", [154.4, 19.65, -267.6], 0.8),
            ("ramp mid", [154.0, 20.6, -266.0], 0.9),
            ("strip south", [154.0, 21.6, -264.0], 0.9),
            ("strip north 1", [154.0, 22.3, -261.0], 0.9),
            ("strip north 2", [154.0, 22.3, -257.0], 0.9),
            ("west onto the guards floor", [152.5, 22.73, -255.5], 0.9),
            ("the armour approach", [149.0, 22.73, -253.2], 0.9),
            ("the armour spot", ARMOUR, 1.3),
        ];
        for (label, at, r) in legs {
            let before = bond_pos(&w);
            let ok = w.try_walk_to(Wp::new(at[0], at[1], at[2]).r(*r), label);
            snap(&mut w, 30);
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[leg] {label}: {before:?} -> {:?} ok {ok} grounded {} armour {:.1}",
                bond_pos(&w), sim.bond.grounded, sim.drones.health.armour);
            if !sim.bond.grounded { eprintln!("[leg] AIRBORNE — stopping the variant"); break; }
        }
    }

    // Variant B: ONE long leg up the band (the walker's own climb over the ramp), from the
    // top-out straight to the strip's north end.
    eprintln!("=== B. one long leg (154, -256) from the top-out ===");
    {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [155.5, 19.65, -267.5];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(30);
        w.lenient = true;
        let ok = w.try_walk_to(Wp::new(154.0, 22.3, -256.0).r(1.0), "the long climb");
        snap(&mut w, 30);
        eprintln!("[leg] the long climb: -> {:?} ok {ok} armour {:.1}", bond_pos(&w), w.scene.sim.as_ref().unwrap().drones.health.armour);
        if w.scene.sim.as_ref().unwrap().bond.grounded && (bond_pos(&w)[0] - 154.0).abs() < 3.0 && bond_pos(&w)[2] < -257.0 && bond_pos(&w)[1] > 21.0 {
            let ok2 = w.try_walk_to(Wp::new(ARMOUR[0], ARMOUR[1], ARMOUR[2]).r(1.3), "the armour from the strip");
            snap(&mut w, 30);
            eprintln!("[leg] the armour from the strip: -> {:?} ok {ok2} armour {:.1}", bond_pos(&w), w.scene.sim.as_ref().unwrap().drones.health.armour);
        }
    }

    // Variant C: the JUMP mount — if the ramp mouth needs a hop, press jump while walking
    // north-west from the top-out (the game's jump is vertical-only [M 0x1916b0], so the
    // mount is a hop ONTO the band's south tip).
    eprintln!("=== C. the hop onto the band mouth ===");
    {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [154.6, 19.65, -267.2];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(30);
        w.lenient = true;
        // Walk north with the jump pressed each 20 ticks until grounded ABOVE y 20.5.
        for k in 0..60 {
            let mut input = nf_game::player::Input { forward: 1.0, ..Default::default() };
            if k % 20 == 3 { input.jump = true; }
            w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
            w.total_ticks += 1;
            let sim = w.scene.sim.as_ref().unwrap();
            if k % 10 == 0 { eprintln!("[hop] +{k}: {:?} grounded {}", sim.bond.pos, sim.bond.grounded); }
            if sim.bond.grounded && sim.bond.pos[1] > 20.5 { break; }
        }
        eprintln!("[hop] settled {:?}", bond_pos(&w));
        if bond_pos(&w)[1] > 20.5 {
            let ok = w.try_walk_to(Wp::new(154.0, 22.3, -260.0).r(1.0), "north on the band");
            snap(&mut w, 30);
            eprintln!("[leg] north on the band: -> {:?} ok {ok}", bond_pos(&w));
        }
    }
}
