//! s-auto-aim: Bond's auto-aim per tick as `Player_AutoAim` 0x1a7c98 runs it (spec:
//! docs/systems/auto-aim.md). Each test drives the real `Sim::step_bond` path and fails on the
//! port's previous angle-unit model:
//! 1. one ease per aimed tick toward the cursor, in screen units [M disasm 0x1a7e2c-0x1a7e74] — the
//!    port eased twice (step_bond + the fire path's dir pick) in angle units;
//! 2. with the option off the lock pointer `BL+0x114` is left alone [M 0x1a7e78-0x1a7ecc: that arm
//!    only halves the offsets] — the port cleared it;
//! 3. the Q-Claw forces the scan on whatever the option says [M 0x1a7cfc-0x1a7d18: `movz` sets the
//!    option byte to 1 for 0x50/0x51];
//! 4. a Q-Claw scan never releases its lock: the release compares the cap with `Autoaim_Range` (25)
//!    and the claw's cap starts at its record's +0x1c (50) [M 0x1a7d64, 0x1a7e0c-0x1a7e20].

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn tick_idle(sim: &mut nf_view::scene::Sim, n: u32) {
    for _ in 0..n { sim.step_bond(Default::default(), false, false, false); }
}

fn load05(iso: &str) -> nf_view::scene::Scene {
    let mut scene = nf_view::load_weapons(iso, "07000005.bin", false, &[0x02]).expect("load 07000005");
    for _ in 0..7200 {
        if scene.sim.as_ref().map_or(false, |s| !s.disabled) { break; }
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    }
    assert!(scene.sim.as_ref().map_or(false, |s| !s.disabled), "07000005 never enabled");
    scene
}

/// Gives `id` through the panel and rides the switch to the idle machine (fix_aim_raise_live's helper).
fn give_and_settle(scene: &mut nf_view::scene::Scene, id: u8, rounds: i32) {
    let (status, sw) = nf_view::panel::apply(scene, &[
        ("do".to_string(), "weapon".to_string()),
        ("id".to_string(), format!("0x{id:02x}")),
        ("rounds".to_string(), rounds.to_string()),
    ]);
    assert_eq!(sw, Some(id), "give refused: {status}");
    for _ in 0..60 {
        if scene.sim.as_ref().unwrap().gun.weapon == id { break; }
        scene.sim.as_mut().unwrap().step_bond(Input { switch_to: Some(id), ..Default::default() }, false, false, false);
    }
    for _ in 0..7200 {
        let s = scene.sim.as_ref().unwrap();
        if s.gun.switch_state == 0 && s.gun.reload_state == 0 && !s.gun.fire_state && s.view_script == s.weapons.idle_script(id as usize) { break; }
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    }
}

#[test]
fn an_aimed_tick_eases_the_offset_once_toward_the_cursor() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load05(&iso);
    give_and_settle(&mut scene, 0x02, 41);
    let sim = scene.sim.as_mut().unwrap();
    sim.auto_aim = true;
    let aim = Input { aiming: true, ..Default::default() };
    for _ in 0..120 {
        if sim.aiming { break; }
        sim.step_bond(aim.clone(), false, false, false);
    }
    assert!(sim.aiming && sim.free_aim, "precondition: aimed (the P2K has no raise script)");
    sim.cursor = [0.2, 0.1];
    sim.drones.aim_offset = [0.0, 0.0];
    sim.step_bond(aim.clone(), false, false, false);
    assert_eq!(sim.cursor, [0.2, 0.1], "precondition: the cursor held still");
    assert_eq!(sim.drones.aim_offset, [0.1, 0.05], "one 0.5 ease toward the cursor, screen units");
    sim.step_bond(aim, false, false, false);
    // The second tick: half the remaining gap again, on the EE's chopped add.s/sub.s/mul.s.
    let ease = |o: f32, c: f32| nf_game::trig::ee_add(o, nf_formats::ps2float::mul(nf_game::trig::ee_sub(c, o), 0.5));
    assert_eq!(sim.drones.aim_offset, [ease(0.1, 0.2), ease(0.05, 0.1)]);
    assert!((sim.drones.aim_offset[0] - 0.15).abs() < 1e-6 && (sim.drones.aim_offset[1] - 0.075).abs() < 1e-6);
}

#[test]
fn the_option_off_keeps_the_lock_pointer() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load05(&iso);
    let sim = scene.sim.as_mut().unwrap();
    sim.auto_aim = false;
    sim.drones.aim_target = Some(usize::MAX);
    sim.drones.aim_offset = [0.5, -0.25];
    sim.step_bond(Default::default(), false, false, false);
    assert_eq!(sim.drones.aim_target, Some(usize::MAX), "the off arm only halves the offsets");
    assert_eq!(sim.drones.aim_offset, [0.25, -0.125]);
}

/// A grapple point 27-45 units away, Bond facing it (the progress-remainder probe's search).
fn claw_spot(sim: &mut nf_view::scene::Sim) -> (usize, nf_game::player::Bond) {
    let dist = |a: [f32; 3], b: [f32; 3]| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
    for (gi, gp) in sim.drones.grapple_points.iter().enumerate() {
        let g = gp.pos;
        for r in [30.0f32, 34.0, 38.0] {
            for a in 0..16 {
                let ang = a as f32 * std::f32::consts::FRAC_PI_8;
                let b = nf_game::player::Bond::stand_at(&sim.world, [g[0] + r * ang.sin(), g[1], g[2] + r * ang.cos()], 0.0);
                let head = [b.pos[0], b.pos[1] + 1.0, b.pos[2]];
                if !(27.0..45.0).contains(&dist(head, g)) { continue; }
                if sim.world.line_of_sight_masked(head, g, nf_game::world::LOS_SENSOR) { return (gi, b); }
            }
        }
    }
    panic!("no open 27-45-unit spot around a grapple point");
}

fn face(sim: &mut nf_view::scene::Sim, g: [f32; 3]) {
    let head = sim.bond.head();
    let (dx, dy, dz) = (g[0] - head[0], g[1] - head[1], g[2] - head[2]);
    sim.bond.yaw = dx.atan2(dz);
    let l = (dx * dx + dy * dy + dz * dz).sqrt();
    sim.bond.pitch = ((dy / l).clamp(-1.0, 1.0).asin() / std::f32::consts::FRAC_PI_2).clamp(-1.0, 1.0);
}

#[test]
fn the_q_claw_scans_with_the_option_off_and_never_releases() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    let sim = scene.sim.as_mut().unwrap();
    tick_idle(sim, 3000);
    const GRAPPLE_AIM: usize = 1 << 20; // drone.rs: the grapple-point index in `aim_target`
    let (gi, b) = claw_spot(sim);
    let g = sim.drones.grapple_points[gi].pos;
    sim.bond = b;
    sim.gun.weapon = 0x50;
    sim.auto_aim = false;
    let mut snapped = false;
    for _ in 0..60 {
        face(sim, g);
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.aim_target == Some(GRAPPLE_AIM + gi) { snapped = true; break; }
    }
    assert!(snapped, "the claw scans with the option off");
    // Face away: nothing in the window, but the claw's cap (50) never equals 25 — the lock stays.
    sim.bond.yaw += std::f32::consts::PI;
    for _ in 0..5 { sim.step_bond(Input::default(), false, false, false); }
    assert_eq!(sim.drones.aim_target, Some(GRAPPLE_AIM + gi), "the claw's empty scan keeps the lock");
}
