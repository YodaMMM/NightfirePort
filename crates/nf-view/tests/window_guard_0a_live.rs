//! 0700000a: the patroller behind the lounge window animates while Bond watches him.
//!
//! The game's "drawn" byte (`obj+0xFC`) is written by the renderer's object walk
//! (`View_AddObjects` 0x1e62d8 [M]) and `NDrone2_DoAnimation` 0x149f88 [M] slides an UNDRAWN
//! drone along its yaw at the danim's speed with no gait. The port stands in for the renderer
//! with rays; those rays stopped on glass (surface byte 0x10 [D the lounge pane at z −4.71]),
//! so the SP3 patroller (template 2, released by plate #716 → ch 0x86 [D]) walked his whole
//! loop past the window as an unanimated statue. Sister: with Bond turned away the same guard
//! is undrawn and slides, as in the game.

#[path = "common/walker_b2.rs"]
mod walker;

use nf_game::player::Input;
use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Turn Bond with the real turn input until his yaw is within 0.02 of `yaw`.
fn turn_to(w: &mut Walker, yaw: f32) {
    for _ in 0..600 {
        let err = walker::wrap_pi(yaw - w.scene.sim.as_ref().unwrap().bond.yaw);
        if err.abs() < 0.02 { return; }
        let input = Input { turn: err.clamp(-0.08, 0.08), ..Input::default() };
        w.tick(input, false);
    }
    panic!("could not turn to yaw {yaw}");
}

#[test]
fn the_patroller_behind_the_lounge_window_walks_with_his_gait() {
    let mut w = Walker::new(nf_view::load(&iso(), "0700000a.bin", false).expect("load 0700000a"));
    // The stand-in transit onto plate #716 (TOUCH_ONCE, out ch 0x86 = SP3's start [D]); the
    // release itself is the game's spawner path.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-11.7, 1.2, -18.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(30);
    let guard = 2usize;
    assert!(w.scene.sim.as_ref().unwrap().drones.list[guard].enabled, "plate #716 released SP3's template 2");
    // The stand-in transit into the lounge, then the real walk to the window spot.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [4.59, 1.2, 5.5];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(12);
    w.walk_to(Wp::new(4.59, 1.1, 2.43).r(0.3), "the lounge window");
    turn_to(&mut w, -2.70);
    // Watch him cross the pane's span (x 7 → −5 at z ≈ −7) while Bond looks through it.
    let (mut seen, mut moved_drawn, mut gait_drawn) = (0u32, 0.0f32, 0.0f32);
    for _ in 0..6000 {
        let (p0, f0) = { let d = &w.scene.sim.as_ref().unwrap().drones.list[guard]; (d.pos, d.anim.ftime) };
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[guard];
        let behind_glass = d.pos[2] < -6.0 && d.pos[2] > -8.5 && d.pos[0] > -2.0 && d.pos[0] < 6.0;
        if !behind_glass { continue; }
        seen += 1;
        if d.drawn {
            moved_drawn += walker::dxz(p0, d.pos);
            gait_drawn += (d.anim.ftime - f0).abs();
        }
        if seen >= 240 { break; }
    }
    assert!(seen >= 240, "the patroller crossed behind the lounge window ({seen} ticks)");
    assert!(moved_drawn > 2.0, "he is drawn while he walks behind the pane (moved {moved_drawn:.2} drawn)");
    assert!(gait_drawn > 100.0, "his walk animation advances while he crosses (clock moved {gait_drawn:.1})");
    // Sister: turned away from the window, the same patroller is undrawn and slides.
    turn_to(&mut w, 0.44);
    let mut undrawn = 0;
    for _ in 0..120 {
        w.tick(Input::default(), false);
        if !w.scene.sim.as_ref().unwrap().drones.list[guard].drawn { undrawn += 1; }
    }
    assert_eq!(undrawn, 120, "with Bond facing away the patroller is not drawn");
}
