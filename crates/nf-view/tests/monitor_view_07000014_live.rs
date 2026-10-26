//! level-14-camera-view: "Press Action for Camera View" works on 07000014 — Action at a security
//! console enters the game's camera view, the feed is the game's camera (the group's FIRST
//! CameraList sensor [M Monitor_Activate 0x1b0058:17-36]), the player freezes while viewing
//! [M Player_Disable 0x195f70 → Player_Update 0x193600:80-87 early-return], the edge cycles
//! forward and past the last camera restores the player view [M Monitor_Update 0x1afcc8:118-168],
//! and the console re-arms after the cooldown drains [M 0x1afcc8:55-65].
//!
//! The console: the east-wing monitor group 2 [D placements — `SWITCH_Consoles_Evilbase` #1166
//! (32.50, 1.54, −17.80), param 0 = 2; the "Press Action for Camera View" hint plate #1237
//! (32.38, 0.61, −17.75) label 0x2000057] and its four group-2 sensors `camera(a)-lens`
//! #1035–#1038 [D: (32.55, 4.36, 15.23), (32.60, 4.36, 10.15), (32.55, 4.36, −6.24),
//! (32.60, 4.36, −11.32)]. The task's observation spot (33.71, 0.65, −18.11) faces the console.
//! Everything drives the real input path: mouse-look aim + E pulses through `Walker::tick`.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

use nf_game::world::V3;
use walker::{dxz as dxz_pub, wrap_pi as wrap_pi_pub};

fn iso() -> String {
    std::env::var("NF_ISO").unwrap_or_else(|_| {
        eprintln!("NF_ISO not set — skipping");
        std::process::exit(0);
    })
}

/// Load, wait for the intro to release the controls, settle.
fn ready(iso: &str) -> Walker {
    let mut w = Walker::new(nf_view::load(iso, "07000014.bin", false).expect("load 07000014"));
    w.wait_enabled(3000, "the intro");
    w.wait(30);
    w
}

/// The east-wing group-2 console: its sim index and position.
fn console(w: &Walker) -> (usize, V3) {
    let sim = w.scene.sim.as_ref().unwrap();
    // The group-2 monitor nearest the task's observation spot (33.71, 0.65, −18.11) [D #1166].
    let (i, d2) = sim.drones.monitors.list.iter().enumerate()
        .filter(|(_, m)| m.group == 2)
        .map(|(i, m)| (i, dxz_pub(m.pos, [33.71, 0.65, -18.11]) + (m.pos[1] - 0.65).powi(2)))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .expect("the east-wing group-2 monitor [D placements #1166]");
    (i, sim.drones.monitors.list[i].pos)
}

/// The group-2 listed sensors, in the port's list (creation) order — the game's CameraList order
/// (tail-append at creation [M LList_Add 0x1cca28], walked from the head [M 0x1b0058:17-36]).
fn group2_cameras(w: &Walker) -> Vec<usize> {
    let sim = w.scene.sim.as_ref().unwrap();
    sim.drones.sensors.list.iter().enumerate()
        .filter(|(_, s)| s.listed && s.monitor_id == 2)
        .map(|(i, _)| i)
        .collect()
}

/// Aim Bond at `target` through the real mouse-look path, pulsing E every 6th tick, until `done`.
fn aim_and_use(w: &mut Walker, target: V3, ticks: u32, done: &dyn Fn(&Walker) -> bool) -> bool {
    for t in 0..ticks {
        if done(w) { return true; }
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        let dx = target[0] - bp[0];
        let dz = target[2] - bp[2];
        let dy = target[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
        let desired = dx.atan2(dz);
        let err = wrap_pi_pub(desired - sim.bond.yaw);
        let want_pitch = (dy / dxz_pub(bp, target).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let perr = want_pitch - sim.bond.pitch;
        let input = nf_game::player::Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() };
        w.tick(input, t % 6 == 0);
    }
    done(w)
}

/// The east-wing console enters the game's camera view on Action: state 2 watching the group's
/// FIRST CameraList camera — sensor #1035 (32.55, 4.36, 15.23) [D] — and the exposed active view
/// IS that sensor's swept pose [M Monitor_Update 0x1afcc8:175 Mat_Copy into viewer+0x120]. While
/// the view stands the player is frozen (movement input does nothing [M Player_Disable 0x195f70:16
/// → Player_Update 0x193600:80-87]).
#[test]
fn action_at_the_console_enters_the_camera_view_on_the_games_first_camera_and_freezes_the_player() {
    let iso = iso();
    let mut w = ready(&iso);
    let (mi, mpos) = console(&w);
    let cams = group2_cameras(&w);
    assert_eq!(cams.len(), 4, "the group-2 CameraList: camera(a)-lens #1035–#1038 [D placements]");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.monitors.list[mi].state, 0, "idle before Action [M 0x1b0058's state gate]");
    // The sister within the same load: before the press nothing is watched and no view stands.
    assert!(w.scene.sim.as_ref().unwrap().monitor_camera().is_none());

    // The task's observation spot, facing the console through the real look path.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [33.71, 0.65, -18.11];
        sim.bond.vel = [0.0; 3];
    }
    for _ in 0..200 { if w.scene.sim.as_ref().unwrap().bond.grounded { break; } w.tick(nf_game::player::Input::default(), false); }
    let entered = aim_and_use(&mut w, mpos, 240, &|w| {
        w.scene.sim.as_ref().unwrap().drones.monitors.list[console(w).0].state == 2
    });
    assert!(entered, "Action at the console enters the camera view [M Monitor_Activate 0x1b0058]");

    let sim = w.scene.sim.as_ref().unwrap();
    let watched = sim.drones.monitors.list[mi].watched.expect("a watched camera [M 0x1b0058:38]");
    assert_eq!(Some(watched), cams.first().copied(),
        "the enter picks the CameraList HEAD [M 0x1b0058:17-36]");
    let s = &sim.drones.sensors.list[watched];
    assert!(dxz_pub(s.pos, [32.55, 4.36, 15.23]) < 0.3 && (s.pos[1] - 4.36).abs() < 0.3,
        "the game's camera id is disc camera(a)-lens #1035 [D placement]");
    assert_eq!(s.display_mask, 0, "the watched camera's display mask clears (the body hides) [M 0x1b0058:26]");
    // The active view camera: the watched sensor's own matrix — position = its translation,
    // yaw/pitch = Mat_GetDir's axis (row 2) [M 0x1afcc8:175; the row-2 read is the game's own
    // Mat_GetDir use at 0x1a5170:101].
    let (p, yaw, pitch) = sim.monitor_camera().expect("the view stands [M 0x1afcc8:175]");
    let m = &sim.drones.sensors.list[watched].matrix;
    let dir = [m[2][0], m[2][1], m[2][2]];
    assert!((p[0] - m[3][0]).abs() < 1e-4 && (p[1] - m[3][1]).abs() < 1e-4 && (p[2] - m[3][2]).abs() < 1e-4);
    assert!((yaw - dir[0].atan2(dir[2])).abs() < 1e-4 && (pitch - dir[1].clamp(-1.0, 1.0).asin()).abs() < 1e-4);
    assert!(sim.monitor_viewing(), "the view state stands [M 0x1b0058:40 state 2]");

    // The freeze: movement + look input does nothing while the feed stands [M Player_Disable
    // 0x195f70:16 (BL+0x94A = 0) → Player_Update 0x193600:80-87 returns early].
    let (pos0, yaw0, pitch0) = { let s = w.scene.sim.as_ref().unwrap(); (s.bond.pos, s.bond.yaw, s.bond.pitch) };
    for _ in 0..12 {
        w.tick(nf_game::player::Input { turn: 0.06, look: 0.05, ..Default::default() }, false);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert_eq!(sim.bond.pos, pos0, "no movement while viewing [M Player_Update 0x193600:80-87]");
    assert_eq!(sim.bond.yaw, yaw0, "no aim while viewing [M]");
    assert_eq!(sim.bond.pitch, pitch0, "no aim while viewing [M]");
}

/// The Action edge cycles FORWARD through the group's cameras [M Monitor_Update 0x1afcc8:124-153
/// — the walk starts at the watched camera's node's next, NO wrap] and past the last camera the
/// view exits: state 9, the player view restored and the controls back [M :159-168 Player_Enable
/// 0x195498], the console re-arming after the cooldown drains [M :55-65].
#[test]
fn the_edge_cycles_forward_and_past_the_last_camera_restores_the_player_view() {
    let iso = iso();
    let mut w = ready(&iso);
    let (mi, mpos) = console(&w);
    let cams = group2_cameras(&w);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [33.71, 0.65, -18.11];
        sim.bond.vel = [0.0; 3];
    }
    for _ in 0..200 { if w.scene.sim.as_ref().unwrap().bond.grounded { break; } w.tick(nf_game::player::Input::default(), false); }
    let entered = aim_and_use(&mut w, mpos, 240, &|w| {
        w.scene.sim.as_ref().unwrap().drones.monitors.list[console(w).0].state == 2
    });
    assert!(entered, "the enter [M 0x1b0058]");
    // Each further press steps to the NEXT camera in list order — never back.
    for (k, want) in cams.iter().skip(1).enumerate() {
        let stepped = aim_and_use(&mut w, mpos, 30, &|w| {
            w.scene.sim.as_ref().unwrap().drones.monitors.list[console(w).0].watched == Some(*want)
        });
        assert!(stepped, "press {} steps forward to camera {} [M 0x1afcc8:124-153]", k + 2, want);
    }
    // Past the last camera: the exit.
    let exited = aim_and_use(&mut w, mpos, 30, &|w| {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.monitors.list[console(w).0].state != 2
    });
    assert!(exited, "the press after the last camera exits [M 0x1afcc8:159-168]");
    let sim = w.scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.monitors.list[mi].state, 9, "the exit cooldown state [M 0x1afcc8:164]");
    assert_eq!(sim.drones.monitors.list[mi].watched, None);
    assert!(sim.monitor_camera().is_none(), "the player view is back [M Camera_SetUpdator 0x1bc370 re-attached]");
    // The last camera's mask restored before the walk [M 0x1afcc8:125].
    let last = *cams.last().unwrap();
    assert_eq!(sim.drones.sensors.list[last].display_mask, 1);
    // The controls answer again: a turn input moves the view.
    let yaw0 = sim.bond.yaw;
    for _ in 0..8 {
        w.tick(nf_game::player::Input { turn: 0.06, ..Default::default() }, false);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!((sim.bond.yaw - yaw0).abs() > 0.01, "the player controls are restored [M Player_Enable 0x195498]");
    // The cooldown drains to re-arm [M 0x1afcc8:55-65: 9→8→…→5 per tick, 4 → 0].
    for _ in 0..8 { w.tick(nf_game::player::Input::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.monitors.list[mi].state, 0, "the cooldown drains [M 0x1afcc8:55-65]");
    // And the console re-arms on the next press.
    let re = aim_and_use(&mut w, mpos, 60, &|w| {
        w.scene.sim.as_ref().unwrap().drones.monitors.list[console(w).0].state == 2
    });
    assert!(re, "the cooled-down console re-arms [M 0x1afcc8:65 → the state-0 gate]");
    let sim = w.scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.monitors.list[mi].watched, cams.first().copied(), "the re-arm restarts at the head [M 0x1b0058:17-36]");
}

/// Sister: aiming at the console without pressing Action leaves it idle — no state, no watched
/// camera, no view [M 0x1b0058's gate].
#[test]
fn without_action_the_console_stays_idle_and_no_view_stands() {
    let iso = iso();
    let mut w = ready(&iso);
    let (mi, mpos) = console(&w);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [33.71, 0.65, -18.11];
        sim.bond.vel = [0.0; 3];
    }
    for _ in 0..200 { if w.scene.sim.as_ref().unwrap().bond.grounded { break; } w.tick(nf_game::player::Input::default(), false); }
    // The same aim, E never pressed.
    for t in 0..60 {
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        let desired = (mpos[0] - bp[0]).atan2(mpos[2] - bp[2]);
        let err = wrap_pi_pub(desired - sim.bond.yaw);
        w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, false);
        let _ = t;
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.monitors.list[mi].state, 0, "idle without Action [M 0x1b0058]");
    assert_eq!(sim.drones.monitors.list[mi].watched, None);
    assert!(sim.monitor_camera().is_none());
    assert!(!sim.monitor_viewing());
}
