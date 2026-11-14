//! probe14_pocket_settle: the 1.72-landing hover pocket, settled WITHOUT the walk (the driver's
//! game-side check, level-14-landing-float). The recorded wedge: tick 26340, Bond floating 0.41
//! over the scoped floor at (33.84, 0.82, 30.95), grounded=false, on the landing resupply leg.
//! This probe teleports Bond to that spot, settles with NO input for 240 ticks, and prints
//! grounded / the scoped floor ray / the capsule pushes every 15 ticks. Run:
//!   NF_ISO=... NF_CAPS_LOG=1 cargo test --offline --release -p nf-view \
//!     --test probe14_pocket_settle -- --ignored --nocapture --test-threads=1
//! The [cap] PUSH lines on stderr are the capsule contacts (crates/nf-game/src/world.rs).
use nf_formats::iso9660::IsoImage;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe14_pocket_settle() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let _ = read_archive(&iso, "07000014.bin");
    // The d3 load (the walk's own difficulty — the collision query shape is the walk's).
    let mut scene = nf_view::load_with_carry_difficulty(&iso, "07000014.bin", false, None, 3).unwrap();
    // Settle at the spawn first so the prev_pos/vel/grounded state is coherent, then teleport.
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); }
    {
        let sim = scene.sim.as_mut().unwrap();
        eprintln!("spawn settled at ({:.3}, {:.3}, {:.3})", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
        sim.bond.pos = [33.84, 0.82, 30.95];
        sim.bond.prev_pos = sim.bond.pos;
        sim.bond.vel = [0.0, 0.0, 0.0];
    }
    for i in 0..240 {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        if i % 15 == 0 || i == 239 {
            let sim = scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let floor = sim.world.segment_hit_scoped(sim.bond.cel, p, [p[0], p[1] - 2.0, p[2]]);
            let below = sim.world.floor_below(p, 6.0);
            eprintln!("tick {i}: pos ({:.4}, {:.4}, {:.4}) vel ({:.4},{:.4},{:.4}) grounded {} ny {:.2} hist {:x} fall {} cc {:.3} sub {} crouch {} cel {:?} health {:.1} scoped_floor_down2 {:?} floor_below {:?}",
                p[0], p[1], p[2], sim.bond.vel[0], sim.bond.vel[1], sim.bond.vel[2],
                sim.bond.grounded, sim.bond.floor_ny, sim.bond.grounded_history, sim.bond.fall_timer,
                sim.bond.cc, sim.bond.substate, sim.bond.crouched,
                sim.bond.cel, sim.drones.health.health, floor, below);
        }
    }
}
