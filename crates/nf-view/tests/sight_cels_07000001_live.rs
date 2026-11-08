//! agent-sight-cels: a guard's sight of Bond needs the cel walk, not just a clear ray.
//!
//! The game's sight probe `DroneVision_CanSeeObjectFrom` 0x176f18 [M disasm 0x176fa4-0x176ffc]:
//! when Bond's cel (`obj+0x20`) differs from the guard's, `AINetwork_TestRayCels` 0x15ab78 runs
//! before any collision ray — `Collide_StraddleCels` 0x1eac98 from Bond's cel along the segment
//! bone → guard eye (HITTEST flags 0x201: each portal's two triangles against the ray,
//! `Intersect_Portal` 0x1ea7b8) — and the guard's cel must be in the cels that walk reaches,
//! else the probe answers "not seen" [M 0x15ab78:63-77]. The guard ray's mask 0xa27 drops the
//! class-0x10 surfaces (bit 0x2), so the paper walls of the 07000001 house stop no ray: only the
//! cel walk keeps a guard in a room from seeing Bond on the veranda through the paper.
//!
//! The walk: Bond driven with input only from the spawn (after the intro) to (11.43, 6.47, -3.51),
//! then 600 ticks standing there. Every tick a guard's sight flag rises, the probe it ran is
//! re-checked against the cel rule for Bond's seven view bones. The port has no cel gate in its
//! sight probe (docs/reports/agent-sight-cels.md); this pins that on this walk its collision ray
//! never grants sight the game's cel walk would refuse.

#[path = "common/walker.rs"]
mod walker;

use nf_game::world::V3;
use std::sync::{Arc, Mutex};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const SPOT: V3 = [11.43, 6.47, -3.51];

/// The guard's cel: `build_FindCel` 0x1c8cb0 at his position (`CollisionWorld::find_cel`) — the
/// port keeps no stored drone cel; the game's obj+0x20 is relinked from the same search.
fn guard_cel(sim: &nf_view::scene::Sim, i: usize) -> Option<usize> { sim.world.find_cel(sim.drones.list[i].pos) }

#[test]
fn guards_see_bond_only_through_the_cels_the_ray_crosses() {
    let mut w = walker::Walker::new(nf_view::load(&iso(), "07000001.bin", false).expect("load 07000001"));
    nf_view::settle_start_draw(w.scene.sim.as_mut().unwrap());
    w.wait_enabled(6000, "the intro");
    // (tick, guard, guard cel, Bond cel)
    let bad: Arc<Mutex<Vec<(u32, usize, Option<usize>, Option<usize>)>>> = Arc::new(Mutex::new(Vec::new()));
    let rises = Arc::new(Mutex::new(0u32));
    let mut last: Vec<bool> = Vec::new();
    {
        let (bad, rises) = (bad.clone(), rises.clone());
        w.on_step = Some(Box::new(move |scene: &nf_view::scene::Scene, tick: u32| {
            let sim = scene.sim.as_ref().unwrap();
            last.resize(sim.drones.list.len(), false);
            for (i, g) in sim.drones.list.iter().enumerate() {
                let rose = g.los && !last[i];
                last[i] = g.los;
                if !rose { continue; }
                let Some(bones) = sim.drones.bond_view_bones else { continue };
                *rises.lock().unwrap() += 1;
                let (bc, gc) = (sim.bond.cel, guard_cel(sim, i));
                if bc == gc { continue; }
                let eye = g.head();
                let ok = match (bc, gc) {
                    (Some(b), Some(c)) => bones.iter().any(|&p| {
                        let from = nf_game::drone::clamp_to_cylinder(sim.bond.pos, p);
                        sim.world.debug_ray_cels(b, from, eye).contains(&c)
                    }),
                    _ => false,
                };
                if !ok {
                    eprintln!("t{tick} guard#{i} at ({:.2},{:.2},{:.2}) cel {gc:?} sees Bond at ({:.2},{:.2},{:.2}) cel {bc:?} with no cel walk between",
                        g.pos[0], g.pos[1], g.pos[2], sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
                    bad.lock().unwrap().push((tick, i, gc, bc));
                }
            }
        }));
    }
    w.walk_nav_to(SPOT, "spawn to the far room", 0.6);
    for _ in 0..600 { w.tick(Default::default(), false); }
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("Bond at ({:.2},{:.2},{:.2}) after {} ticks; {} sight rises", p[0], p[1], p[2], w.total_ticks, rises.lock().unwrap());
    assert!(walker::dxz(p, SPOT) < 1.5, "Bond reached the spot by walking");
    assert!(*rises.lock().unwrap() > 0, "guards did see Bond on the walk (the check is not vacuous)");
    let bad = bad.lock().unwrap();
    assert!(bad.is_empty(), "guards saw Bond with no cel walk between them [M 0x176f18 / 0x15ab78]: {bad:?}");
}
