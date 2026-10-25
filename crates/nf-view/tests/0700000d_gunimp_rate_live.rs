//! 0d mounted gun: the mount must turn at the player's own view rate.
//!
//! [M `GunImp_Update` 0x1320d8:124-135]: the game's aim integrates `0.015 * Input_Actionf(...)` per
//! tick — 0.015 rad per tick per STICK UNIT (full deflection = 1). The port's `Input.turn`/`Input.look`
//! are not stick units: they are the player's own view deltas already in radians per tick (mouse:
//! `dx * MOUSE_SENSITIVITY`, `lib.rs:6066`; pad: the game's own stick mapping before merging,
//! `gamepad.rs:105-106`), applied 1:1 by Bond's view (`yaw += turn`). A `* 0.015` left in made
//! the same mouse movement turned the mount 1/0.015 = 67x slower than the view — "slow to turn for mouse
//! movement". The fix feeds the view delta through unchanged; the game's stick rate is preserved by
//! construction (a full-deflection stick contributes its converted 0.015 rad/tick through `turn`).
//!
//! Fail-first: on the round-3 tree a `turn` of 0.02 moves the gun 0.0003; the test wants 0.02 (with the
//! pad-sized and mouse-sized deltas both exercised).

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn mounted() -> Option<nf_view::scene::Scene> {
    let iso = iso()?;
    let mut scene = nf_view::load(&iso, "0700000d.bin", false).expect("load 0700000d");
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..2000 { sim.step_bond(Default::default(), false, false, false); if !sim.disabled { break; } }
        sim.bond.pos = [137.27, 9.33, -323.0];
        sim.bond.yaw = std::f32::consts::PI;
        sim.bond.pitch = 0.0;
        for _ in 0..6 {
            sim.step_bond(Default::default(), false, false, true);
            if sim.drones.props.gunimps.iter().any(|g| g.active) { break; }
        }
    }
    scene.sim.as_ref().unwrap().drones.props.gunimps.iter().any(|g| g.active).then_some(scene)
}

fn aim(scene: &nf_view::scene::Scene) -> (f32, f32) {
    let g = scene.sim.as_ref().unwrap().drones.props.gunimps.iter().find(|g| g.active).unwrap();
    (g.yaw, g.pitch)
}

#[test]
fn the_mount_turns_at_the_views_own_rate() {
    let Some(mut scene) = mounted() else { eprintln!("NF_ISO not set or the mount is unreachable — skipping"); return };
    for (name, turn, look) in [("mouse-sized", 0.02f32, 0.02f32), ("pad-sized", 0.015, 0.015)] {
        let (y0, p0) = aim(&scene);
        scene.sim.as_mut().unwrap().step_bond(
            nf_game::player::Input { turn, look, ..Default::default() }, false, false, false);
        let (y1, p1) = aim(&scene);
        assert!((y1 - y0 - turn).abs() < 1e-5,
            "{name}: the mount must turn by the view's own delta ({turn}) — the port's turn is already \
             radians/tick, not a stick unit [M 0x1320d8 gives 0.015 per stick unit]; got {}",
            y1 - y0);
        assert!((p1 - p0 + look).abs() < 1e-5,
            "{name}: the down-positive pitch must fall by the look delta ({look}); got {}", p1 - p0);
    }
}
