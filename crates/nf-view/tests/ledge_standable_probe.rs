//! context-icon-arm-radius probe: the west-ledge standable grid on 07000008 around the cable
//! icon (133.4257, 48.3264, −101.2065). Assertion-free (opt-in NF_ISO, --ignored): for each
//! grid spot Bond is placed, settled 90 ticks, and the printout carries the stood position,
//! the live icon_ctx (the master sphere rule), and the distances the game's own tests use:
//! the [L] walk-pick segment distance (segment [eye−0.4847, eye+0.275] over the eye,
//! picker 0.55 — Player_Collision 0x194df0 default branch, verified [L] on ee_start_05) and
//! the sphere-centre distance. Numbers only.
use nf_game::player::{Bond, Input};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load_08(iso: &str) -> nf_view::scene::Scene {
    let mut scene = nf_view::load(iso, "07000008.bin", false).expect("load 07000008");
    let idle = Input::default();
    for _ in 0..600 { if !scene.sim.as_ref().unwrap().disabled { break; } scene.sim.as_mut().unwrap().step_bond(idle, false, false, false); }
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(idle, false, false, false); }
    scene
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe_west_ledge_grid() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut scene = load_08(&iso);
    let icon: [f32; 3] = scene.sim.as_ref().unwrap().icons[0].centre;
    println!("icon centre ({:.4},{:.4},{:.4})", icon[0], icon[1], icon[2]);
    println!("x,z, stood_y, eye_y, seg_dist, sphere_dist, icon_ctx(master rule)");
    // The walk-pick constants [L ee_start_05]: seg = [eye−0.4847, eye+0.275], picker 0.55.
    const SEG_LO: f32 = 0.55 - 1.0347183;
    const SEG_HI: f32 = 0.275;
    let eye_off = 0.7f32; // the port's EYE over the root
    for x in [130.5f32, 131.0, 131.5, 132.0, 132.5, 133.0, 133.5, 134.0] {
        for z in [-105.0f32, -104.5, -104.0, -103.5, -103.0, -102.5, -102.0, -101.5, -101.0, -100.5, -100.0] {
            let sim = scene.sim.as_mut().unwrap();
            sim.bond = Bond::new([x, 48.5, z], 0.0);
            for _ in 0..90 { sim.step_bond(Input::default(), false, false, false); }
            let p = sim.bond.pos;
            // discard spots that fell off (below the ledge floor band 47.0..48.2)
            let stood = if p[1] < 46.0 { format!("{:.2} FALL", p[1]) } else { format!("{:.4}", p[1]) };
            let eye_y = p[1] + eye_off;
            let lo = eye_y + SEG_LO;
            let hi = eye_y + SEG_HI;
            let dy = if icon[1] < lo { lo - icon[1] } else if icon[1] > hi { icon[1] - hi } else { 0.0 };
            let dxz2 = (icon[0] - p[0]).powi(2) + (icon[2] - p[2]).powi(2);
            let seg_dist = (dxz2 + dy * dy).sqrt();
            let sc = sim.bond.sphere_centre();
            let sphere_dist = ((icon[0] - sc[0]).powi(2) + (icon[1] - sc[1]).powi(2) + (icon[2] - sc[2]).powi(2)).sqrt();
            println!("{:.1},{:.1}, {}, {:.4}, {:.4}, {:.4}, {}", p[0], p[2], stood, eye_y, seg_dist, sphere_dist, sim.bond.icon_ctx);
        }
    }
}
