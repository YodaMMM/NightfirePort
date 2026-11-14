//! probe14_pocket_settle: the 1.72-landing hover pocket, settled WITHOUT the walk — the
//! red/green proof for the port's fallback-hold bug (real-health-00agent-14-landing; the
//! diagnosis is level-14-landing-float's). The recorded wedge: tick 26340, Bond floating 0.41
//! over the scoped floor at (33.84, 0.82, 30.95), grounded=false, on the landing resupply leg.
//! The game's own push tail is DEAD in exactly this class [M Intersect_CylGeom 0x109da0:431-447:
//! the whole push sits inside `0 < dot(q − HITTEST+0x50, n)` AND `|q − ref| < radius`, ref = the
//! lifetime DeltaP accumulator [M Collide_Intersect 0x1ebaa8:353-373] — for a face under the
//! capsule q is below ref and the dot gate fails; there is no fallback in the decompile]. The
//! port's [S] spine fallback (dir = v̂ ≈ +Y, mag = r − dist) is the invention that holds the
//! capsule at the r-equilibrium forever. Two legs, one teleport each:
//!   LEG A (the bug, no env): the hover persists — grounded=false through the settle;
//!   LEG B (NF_HOLD_SKIP=1, the carried fix, walk path only): the pocket grounds —
//!   grounded=true, hist 0xf, rest ≈ (33.97, 0.63, 30.95) [L level-14-landing-float's probe
//!   run; re-measured on this tree]. LEG B is RED on the old code (no skip support → the
//!   hover persists). The skip stays OFF in every walk binary (the merged-tree A/B measured
//!   the skip losing the stairwell pit-band jump's east gate on the current stream — master's
//!   small-mag tail is load-bearing in the arcs); the scope that separates the pocket's
//!   never-works hold from the arcs' real work is the next window's measurement.
use nf_formats::iso9660::IsoImage;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn settle_leg(skip_on: bool) -> (bool, [f32; 3]) {
    let Some(iso) = iso() else { panic!("NF_ISO not set") };
    let _ = read_archive(&iso, "07000014.bin");
    // The d3 load (the walk's own difficulty — the collision query shape is the walk's).
    let mut scene = nf_view::load_with_carry_difficulty(&iso, "07000014.bin", false, None, 3).unwrap();
    // Settle at the spawn first so the prev_pos/vel/grounded state is coherent, then teleport.
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); }
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [33.84, 0.82, 30.95];
        sim.bond.prev_pos = sim.bond.pos;
        sim.bond.vel = [0.0, 0.0, 0.0];
    }
    let mut last_grounded = false;
    let mut last_pos = [0.0f32; 3];
    for i in 0..240 {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        if i % 30 == 0 || i == 239 {
            let sim = scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            eprintln!("[pocket] skip_on {skip_on} tick {i}: pos ({:.4}, {:.4}, {:.4}) grounded {} hist {:x} fall {} vel ({:.4},{:.4},{:.4})",
                p[0], p[1], p[2], sim.bond.grounded, sim.bond.grounded_history, sim.bond.fall_timer,
                sim.bond.vel[0], sim.bond.vel[1], sim.bond.vel[2]);
        }
        let sim = scene.sim.as_ref().unwrap();
        last_grounded = sim.bond.grounded;
        last_pos = sim.bond.pos;
    }
    (last_grounded, last_pos)
}

/// LEG A asserts the BUG (the port's fallback hold keeps Bond at the r-equilibrium above the
/// pocket's flat up-facing floor — grounded=false through the settle). LEG B asserts the FIX
/// (NF_HOLD_SKIP=1: the walk pass skips the never-works hold and the feet band grounds) —
/// this leg is RED on the old code, where no env support exists and the hover persists.
#[test]
fn probe14_pocket_settle() {
    // LEG A: the bug, unskipped.
    std::env::remove_var("NF_HOLD_SKIP");
    let (grounded, pos) = settle_leg(false);
    assert!(!grounded, "LEG A: the port's fallback hold must keep Bond hovering over the pocket floor (grounded=false), got grounded at {pos:?}");
    assert!(pos[1] > 0.6, "LEG A: the hover's r-equilibrium sits ~0.63 over the y≈0 floor, got {pos:?}");
    // LEG B: the carried fix (the walk pass's skip), same teleport, fresh scene.
    std::env::set_var("NF_HOLD_SKIP", "1");
    let (grounded, pos) = settle_leg(true);
    std::env::remove_var("NF_HOLD_SKIP");
    assert!(grounded, "LEG B: with NF_HOLD_SKIP=1 the pocket must ground (the game's dead tail gives no hold [M 0x109da0:431-447]; the skip lets the fall complete and the feet band grounds), got hover at {pos:?}");
    assert!((pos[1] - 0.6335).abs() < 0.1, "LEG B: the rest sits on the pocket floor (the float branch measured (33.9712, 0.6335, 30.9501) [L]), got {pos:?}");
}
