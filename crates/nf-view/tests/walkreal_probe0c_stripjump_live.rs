//! walkreal probe #18 (level-0c-gate-band-crossing): THE STRIP JUMP TEST. Teleports Bond onto
//! the west band (diagnostic), walks east, steps north onto the 10.85 west strip (x 49.3..53.6,
//! z −206.3..−205.1 [D room_turbine02 tris]), runs east at full speed and jumps at the strip's
//! east lip (x 53.57 [D]) toward the east slope tip (x 58.86, 10.17 [D room_turbine02 tris]).
//! Prints the flight and the landing. Diagnostic only, --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_strip_jump() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    // Diagnostic teleport onto the west band (the walkreal reaches this leg for real; the probe
    // only tests the jump itself).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [50.0, 11.0, -207.6];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    // Walk east along the band to its east end, then north onto the strip.
    w.walk_to(walker::Wp::new(53.0, 11.0, -207.6).r(0.9), "the band east to the strip corner");
    w.walk_to(walker::Wp::new(52.8, 11.0, -205.8).r(0.7), "step north onto the west strip");
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("[strip] on the west strip at ({:.2},{:.2},{:.2}) grounded {}", p[0], p[1], p[2], w.scene.sim.as_ref().unwrap().bond.grounded);

    // The run-up + jump: run east from x 52.8, fire the jump the LAST grounded tick before the
    // lip (x >= 53.5), then coast and print the flight.
    let mut jumped = false;
    let mut jump_x = 0.0f32;
    for t in 0..400u32 {
        let (p, grounded) = {
            let sim = w.scene.sim.as_ref().unwrap();
            (sim.bond.pos, sim.bond.grounded)
        };
        if jumped && grounded && t > 20 {
            eprintln!("[strip] LANDED at ({:.2},{:.2},{:.2}) grounded, t+{}", p[0], p[1], p[2], t);
            let under = w.scene.sim.as_ref().unwrap().world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 3.0, p[2]]);
            eprintln!("[strip] floor under: {:?} (want the east slope 10.17 x 58.86..63.32 [D])", under.map(|(tt, _)| p[1] - 3.0 * tt));
            break;
        }
        if jumped && !grounded && p[1] < 8.0 {
            eprintln!("[strip] FELL into the pit at ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
            break;
        }
        let (turn, forward) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            let desired = (60.0f32 - b.pos[0]).atan2(-205.7f32 - b.pos[2]);
            let err = walker::wrap_pi(desired - b.yaw);
            (err.clamp(-0.08, 0.08), if err.abs() > 0.6 { 0.0 } else { 1.0 })
        };
        let jump = !jumped && grounded && p[0] >= 53.45;
        if jump { jumped = true; jump_x = p[0]; eprintln!("[strip] jump fired at x {:.2} y {:.2}", p[0], p[1]); }
        w.tick(Input { turn, forward, jump, ..Default::default() }, false);
        if t % 6 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            eprintln!("[strip] t{t} ({:.2},{:.2},{:.2}) grounded {} vel ({:.2},{:.2},{:.2})", b.pos[0], b.pos[1], b.pos[2], b.grounded, b.vel[0], b.vel[1], b.vel[2]);
        }
    }
    eprintln!("ok (jump_x {jump_x})");
}
