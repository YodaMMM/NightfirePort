//! wade-probe: the assertion-free measurement for shallow-water-wade. Settles AT the PCSX2-measured
//! 07000002 shallows spot (-79, 4.68, 28) [L: byte 0xE, step 0x28], walks WITHIN the shelf, prints
//! the wade speed vs the dry speed, the ripple puffs per step, the decal table for 0xE and the eye.
//! For 07000006 it confirms candidate spots with the SCOPED feet ray from Bond's own cel (the
//! driver's (24,-7) plain-ray point is another surface in the game). Run:
//!   NF_ISO=... cargo test --offline --release -p nf-view --test wade_probe -- --nocapture --test-threads=1
use nf_game::player::Input;
use nf_game::world_fx::RIPPLE_MESH;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn dry_walk_speed(sim: &mut nf_view::scene::Sim) -> f32 {
    let mut steps = 0.0f32;
    for _ in 0..60 {
        let p0 = sim.bond.pos;
        sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
        steps += ((sim.bond.pos[0] - p0[0]).powi(2) + (sim.bond.pos[2] - p0[2]).powi(2)).sqrt();
    }
    steps / 60.0
}

#[test]
fn probe_02_wade_behaviour() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..2000 { sim.step_bond(Default::default(), false, false, false); if !sim.disabled { break; } }
    let dry = dry_walk_speed(sim);
    eprintln!("DRY: speed {:.4}/tick eye {:.3} substate {}", dry, sim.bond.eye, sim.bond.substate);
    // The 0xE surface record (the decal/gfx table the port and the game share [D]).
    if let Some(s) = sim.surfaces.as_ref() {
        eprintln!("surfaces[0xE] = {:?}", s.effects.get(0xE));
        eprintln!("surfaces[0xD] = {:?}", s.effects.get(0xD));
        if let Some(f) = nf_formats::sfx::SurfaceSounds::get(&s.footstep, 0xE) { eprintln!("footstep sound for 0xE = {:#x}", f); }
    }
    // Settle AT the measured shallows spot, then walk within the shelf.
    sim.bond.pos = [-79.0, 4.68, 28.0];
    sim.bond.prev_pos = sim.bond.pos;
    sim.bond.vel = [0.0; 3];
    for _ in 0..40 { sim.step_bond(Default::default(), false, false, false); }
    eprintln!("settle: pos {:?} grounded {} substate {}", sim.bond.pos, sim.bond.grounded, sim.bond.substate);
    let mut wet_steps = 0.0f32;
    let mut rip_marks = Vec::new();
    let mut decals0 = sim.drones.effects.made;
    for i in 0..120 {
        let p0 = sim.bond.pos;
        let r0 = sim.drones.gas.puffs.iter().filter(|g| g.mesh == RIPPLE_MESH).count();
        sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
        wet_steps += ((sim.bond.pos[0] - p0[0]).powi(2) + (sim.bond.pos[2] - p0[2]).powi(2)).sqrt();
        let r1 = sim.drones.gas.puffs.iter().filter(|g| g.mesh == RIPPLE_MESH).count();
        if r1 > r0 { rip_marks.push(i); }
        if i % 30 == 0 {
            let below = sim.world.segment_surface(sim.bond.pos, [sim.bond.pos[0], sim.bond.pos[1] - 2.5, sim.bond.pos[2]], false);
            eprintln!("wet t{i}: pos {:?} grounded {} sub {} byte {:?} puffs {} decals+{}",
                sim.bond.pos, sim.bond.grounded, sim.bond.substate, below.map(|h| h.2 & 0x3f),
                r1, sim.drones.effects.made - decals0);
        }
    }
    eprintln!("WET: walked {:.3} over 120 ticks ({:.4}/tick) dry {:.4} ripple ticks {:?} decals+{} eye {:.3}",
        wet_steps, wet_steps / 120.0, dry, rip_marks, sim.drones.effects.made - decals0, sim.bond.eye);
}

#[test]
fn probe_06_confirm_spot() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000006.bin", false).expect("load 07000006");
    // Wide settle-scan: teleport Bond over the whole pond region (Object_Pondwater cel #351 at
    // 27.5, 38.6, -7 [D docs/levels/07000006.md]), settle, and read the feet ray's byte at HIS
    // settled root. Prints every distinct byte seen so the pond's true surface map is visible.
    let mut by_byte: std::collections::BTreeMap<u8, Vec<(f32, f32, f32)>> = Default::default();
    for xi in -12..13 {
        for zi in -12..13 {
            let (x, z) = (27.5 + xi as f32 * 1.5, -7.0 + zi as f32 * 1.5);
            let sim = scene.sim.as_mut().unwrap();
            for _ in 0..2000 { sim.step_bond(Default::default(), false, false, false); if !sim.disabled { break; } }
            sim.bond.pos = [x, 39.9, z];
            sim.bond.prev_pos = sim.bond.pos;
            sim.bond.vel = [0.0; 3];
            for _ in 0..40 { sim.step_bond(Default::default(), false, false, false); }
            let feet = [sim.bond.pos[0], sim.bond.pos[1] - (sim.bond.cc + 0.6), sim.bond.pos[2]];
            let b = sim.world.segment_surface(sim.bond.pos, feet, false).map(|h| h.2 & 0x3f);
            if let Some(by) = b {
                let e = by_byte.entry(by).or_default();
                if e.len() < 6 { e.push((sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2])); }
            }
        }
    }
    for (by, spots) in &by_byte {
        eprintln!("06 feet byte {by:#04x}: {} spots e.g. {:?}", spots.len(), &spots[..spots.len().min(3)]);
    }
}
