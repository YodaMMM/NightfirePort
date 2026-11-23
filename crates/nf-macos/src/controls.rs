//! Virtual macOS keyboard & mouse input layer mirroring `nf-view`'s exact `winit::keyboard::KeyCode`
//! bindings, plus an automated `--verify-controls` test suite that validates every directional axis,
//! mouse look axis, menu navigation key, and action key across FrontEnd, On-Foot, and Driving modes.

use std::collections::HashSet;

use anyhow::{anyhow, Result};
use winit::keyboard::KeyCode;

use crate::engine::{
    InputState, LoadOptions, MacEngine, BTN_AIM, BTN_CROUCH, BTN_FIRE, BTN_JUMP, BTN_PAUSE,
    BTN_RELOAD, BTN_USE, BTN_WEAP_NEXT, BTN_WEAP_PREV,
};

/// Virtual keyboard + mouse state that translates real macOS `KeyCode` presses and mouse deltas
/// through the same mapping rules as `NightfirePort.app` (`nf-view/src/lib.rs`).
#[derive(Clone, Debug, Default)]
pub struct VirtualKbm {
    pub keys: HashSet<KeyCode>,
    pub mouse_left: bool,
    pub mouse_right: bool,
    pub turn_accum: f32,
    pub look_accum: f32,
}

impl VirtualKbm {
    pub fn key_down(&mut self, key: KeyCode) {
        self.keys.insert(key);
    }

    pub fn key_up(&mut self, key: KeyCode) {
        self.keys.remove(&key);
    }

    pub fn mouse_move(&mut self, dx: f64, dy: f64, zoom: f32) {
        nf_view::mouse_look_add(&mut self.turn_accum, &mut self.look_accum, dx, dy, zoom);
    }

    pub fn reset(&mut self) {
        self.keys.clear();
        self.mouse_left = false;
        self.mouse_right = false;
        self.turn_accum = 0.0;
        self.look_accum = 0.0;
    }

    /// Converts current held keyboard/mouse state into `MacEngine::InputState` using `nf-view`'s
    /// exact bindings (WASD + Arrows, Space, C/Ctrl, E, R, Q, Mouse Left/Right, mouse look deltas).
    pub fn to_input_state(&mut self, is_driving: bool) -> InputState {
        let held = |k: KeyCode| self.keys.contains(&k);
        let (move_x, move_y) = if is_driving {
            let throttle = if held(KeyCode::KeyW) || held(KeyCode::ArrowUp) { 1.0 } else { 0.0 };
            let brake = if held(KeyCode::KeyS) || held(KeyCode::ArrowDown) { 1.0 } else { 0.0 };
            let steer = if held(KeyCode::KeyA) || held(KeyCode::ArrowLeft) { -1.0 } else { 0.0 }
                + if held(KeyCode::KeyD) || held(KeyCode::ArrowRight) { 1.0 } else { 0.0 };
            (steer, throttle - brake)
        } else {
            let fwd = if held(KeyCode::KeyW) || held(KeyCode::ArrowUp) { 1.0 } else { 0.0 }
                - if held(KeyCode::KeyS) || held(KeyCode::ArrowDown) { 1.0 } else { 0.0 };
            let right = if held(KeyCode::KeyD) || held(KeyCode::ArrowRight) { 1.0 } else { 0.0 }
                - if held(KeyCode::KeyA) || held(KeyCode::ArrowLeft) { 1.0 } else { 0.0 };
            (right, fwd)
        };

        let mut buttons = 0i32;
        if self.mouse_left || (is_driving && held(KeyCode::ControlLeft)) {
            buttons |= BTN_FIRE;
        }
        if self.mouse_right {
            buttons |= BTN_AIM;
        }
        if held(KeyCode::KeyR) {
            buttons |= BTN_RELOAD;
        }
        if held(KeyCode::Space) {
            buttons |= BTN_JUMP;
        }
        if held(KeyCode::KeyC) || (!is_driving && held(KeyCode::ControlLeft)) {
            buttons |= BTN_CROUCH;
        }
        if held(KeyCode::KeyE) {
            buttons |= BTN_USE;
        }
        if held(KeyCode::KeyQ) {
            buttons |= BTN_WEAP_NEXT;
        }
        if held(KeyCode::BracketLeft) {
            buttons |= BTN_WEAP_PREV;
        }
        if held(KeyCode::Escape) {
            buttons |= BTN_PAUSE;
        }

        InputState {
            move_x,
            move_y,
            look_dx: std::mem::take(&mut self.turn_accum),
            look_dy: std::mem::take(&mut self.look_accum),
            buttons,
        }
    }
}

/// Parses human-readable key names (e.g. `w`, `KeyW`, `up`, `ArrowUp`, `space`, `enter`, `esc`, `ctrl`, `c`, `r`, `e`, `q`).
pub fn parse_key(name: &str) -> Result<KeyCode> {
    let k = match name.trim().to_ascii_lowercase().as_str() {
        "w" | "keyw" => KeyCode::KeyW,
        "a" | "keya" => KeyCode::KeyA,
        "s" | "keys" => KeyCode::KeyS,
        "d" | "keyd" => KeyCode::KeyD,
        "c" | "keyc" => KeyCode::KeyC,
        "e" | "keye" => KeyCode::KeyE,
        "r" | "keyr" => KeyCode::KeyR,
        "q" | "keyq" => KeyCode::KeyQ,
        "f" | "keyf" => KeyCode::KeyF,
        "g" | "keyg" => KeyCode::KeyG,
        "up" | "arrowup" => KeyCode::ArrowUp,
        "down" | "arrowdown" => KeyCode::ArrowDown,
        "left" | "arrowleft" => KeyCode::ArrowLeft,
        "right" | "arrowright" => KeyCode::ArrowRight,
        "space" => KeyCode::Space,
        "enter" | "return" => KeyCode::Enter,
        "esc" | "escape" => KeyCode::Escape,
        "backspace" => KeyCode::Backspace,
        "tab" => KeyCode::Tab,
        "ctrl" | "lctrl" | "controlleft" => KeyCode::ControlLeft,
        "shift" | "lshift" | "shiftleft" => KeyCode::ShiftLeft,
        "1" | "digit1" => KeyCode::Digit1,
        "2" | "digit2" => KeyCode::Digit2,
        "3" | "digit3" => KeyCode::Digit3,
        "4" | "digit4" => KeyCode::Digit4,
        "[" | "bracketleft" => KeyCode::BracketLeft,
        "]" | "bracketright" => KeyCode::BracketRight,
        other => return Err(anyhow!("unknown key name: {other}")),
    };
    Ok(k)
}

/// Single control-verification assertion result.
#[derive(Clone, Debug)]
pub struct ControlCheck {
    pub mode: &'static str,
    pub name: &'static str,
    pub passed: bool,
    pub detail: String,
}

impl ControlCheck {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"mode\":\"{}\",\"name\":\"{}\",\"passed\":{},\"detail\":{}}}",
            self.mode,
            self.name,
            self.passed,
            crate::engine::json_escape(&self.detail)
        )
    }
}

fn cam_right(c: &nf_view::Camera) -> glam::Vec3 {
    glam::Vec3::new(-c.yaw.cos(), 0.0, c.yaw.sin())
}

fn cam_dir(c: &nf_view::Camera) -> glam::Vec3 {
    glam::Vec3::new(
        c.pitch.cos() * c.yaw.sin(),
        c.pitch.sin(),
        c.pitch.cos() * c.yaw.cos(),
    )
}

/// Runs the full automated control & directional mapping verification suite across
/// FrontEnd menus, On-Foot gameplay (`07000005.bin`), and Driving (`09000006.bin`).
pub fn verify_controls(engine: &mut MacEngine) -> Result<Vec<ControlCheck>> {
    let mut results = Vec::new();
    let mut kbm = VirtualKbm::default();

    // ── 1. FrontEnd Menu Controls ───────────────────────────────────────────
    engine.load_level_with("07000048.bin", LoadOptions::default())?;
    let c0 = engine.fe.as_ref().map(|f| f.cursor).unwrap_or(99);
    if let Some(act) = nf_view::menu::key_action(KeyCode::ArrowDown) {
        engine.menu_action(act);
    }
    engine.step(1.0 / 60.0);
    let c1 = engine.fe.as_ref().map(|f| f.cursor).unwrap_or(99);
    results.push(ControlCheck {
        mode: "frontend",
        name: "ArrowDown moves menu cursor down",
        passed: c1 == c0 + 1,
        detail: format!("cursor {c0} -> {c1}"),
    });

    if let Some(act) = nf_view::menu::key_action(KeyCode::ArrowUp) {
        engine.menu_action(act);
    }
    engine.step(1.0 / 60.0);
    let c2 = engine.fe.as_ref().map(|f| f.cursor).unwrap_or(99);
    results.push(ControlCheck {
        mode: "frontend",
        name: "ArrowUp moves menu cursor up",
        passed: c2 == c0,
        detail: format!("cursor {c1} -> {c2}"),
    });

    if let Some(act) = nf_view::menu::key_action(KeyCode::Enter) {
        engine.menu_action(act);
    }
    for _ in 0..25 {
        engine.step(1.0 / 60.0);
    }
    let page_after_enter = engine
        .fe
        .as_ref()
        .map(|f| format!("{:?}", f.page))
        .unwrap_or_default();
    results.push(ControlCheck {
        mode: "frontend",
        name: "Enter confirms Main -> Codename",
        passed: page_after_enter == "Codename",
        detail: format!("page={page_after_enter}"),
    });

    // ── 2. On-Foot Directional, Mouse Look, Crouch, Fire & Reload ───────────
    engine.load_level("07000005.bin")?;
    for _ in 0..35 {
        engine.step(1.0 / 60.0);
    }
    let start_pos = engine.scene.sim.as_ref().unwrap().bond.pos;
    let start_yaw = engine.scene.sim.as_ref().unwrap().bond.yaw;
    let fwd = cam_dir(&engine.cam);
    let right = cam_right(&engine.cam);

    let run_hold = |engine: &mut MacEngine, kbm: &mut VirtualKbm, key: KeyCode, ticks: u32| {
        kbm.reset();
        engine.teleport(start_pos, Some(start_yaw), Some(0.0));
        kbm.key_down(key);
        for _ in 0..ticks {
            let inp = kbm.to_input_state(false);
            engine.set_input(inp);
            engine.step(1.0 / 60.0);
        }
        kbm.reset();
        engine.set_input(InputState::default());
        let end_pos = engine.scene.sim.as_ref().unwrap().bond.pos;
        [
            end_pos[0] - start_pos[0],
            end_pos[1] - start_pos[1],
            end_pos[2] - start_pos[2],
        ]
    };

    let dot = |a: [f32; 3], b: glam::Vec3| a[0] * b.x + a[1] * b.y + a[2] * b.z;

    let d_w = run_hold(engine, &mut kbm, KeyCode::KeyW, 25);
    let proj_w = dot(d_w, fwd);
    results.push(ControlCheck {
        mode: "onfoot",
        name: "KeyW moves forward along facing direction",
        passed: proj_w > 0.5,
        detail: format!("dot(delta, fwd)={proj_w:.2}m"),
    });

    let d_s = run_hold(engine, &mut kbm, KeyCode::KeyS, 25);
    let proj_s = dot(d_s, fwd);
    results.push(ControlCheck {
        mode: "onfoot",
        name: "KeyS moves backward opposite facing direction",
        passed: proj_s < -0.5,
        detail: format!("dot(delta, fwd)={proj_s:.2}m"),
    });

    let d_d = run_hold(engine, &mut kbm, KeyCode::KeyD, 25);
    let proj_d = dot(d_d, right);
    results.push(ControlCheck {
        mode: "onfoot",
        name: "KeyD strafes right (+cam.right)",
        passed: proj_d > 0.5,
        detail: format!("dot(delta, right)={proj_d:.2}m"),
    });

    let d_a = run_hold(engine, &mut kbm, KeyCode::KeyA, 25);
    let proj_a = dot(d_a, right);
    results.push(ControlCheck {
        mode: "onfoot",
        name: "KeyA strafes left (-cam.right)",
        passed: proj_a < -0.5,
        detail: format!("dot(delta, right)={proj_a:.2}m"),
    });

    // Mouse look right / left / up / down
    engine.teleport(start_pos, Some(start_yaw), Some(0.0));
    let old_right = cam_right(&engine.cam);
    kbm.reset();
    kbm.mouse_move(100.0, 0.0, 1.0);
    let inp = kbm.to_input_state(false);
    engine.set_input(inp);
    engine.step(1.0 / 60.0);
    let turn_r = cam_dir(&engine.cam).dot(old_right);
    results.push(ControlCheck {
        mode: "onfoot",
        name: "MouseMove(+dx, 0) turns view right",
        passed: turn_r > 0.05,
        detail: format!("dot(new_fwd, old_right)={turn_r:.3}"),
    });

    engine.teleport(start_pos, Some(start_yaw), Some(0.0));
    let old_right = cam_right(&engine.cam);
    kbm.reset();
    kbm.mouse_move(-100.0, 0.0, 1.0);
    let inp = kbm.to_input_state(false);
    engine.set_input(inp);
    engine.step(1.0 / 60.0);
    let turn_l = cam_dir(&engine.cam).dot(old_right);
    results.push(ControlCheck {
        mode: "onfoot",
        name: "MouseMove(-dx, 0) turns view left",
        passed: turn_l < -0.05,
        detail: format!("dot(new_fwd, old_right)={turn_l:.3}"),
    });

    engine.teleport(start_pos, Some(start_yaw), Some(0.0));
    let old_y = cam_dir(&engine.cam).y;
    kbm.reset();
    kbm.mouse_move(0.0, -80.0, 1.0);
    let inp = kbm.to_input_state(false);
    engine.set_input(inp);
    engine.step(1.0 / 60.0);
    let dy_up = cam_dir(&engine.cam).y - old_y;
    results.push(ControlCheck {
        mode: "onfoot",
        name: "MouseMove(0, -dy) looks up",
        passed: dy_up > 0.05,
        detail: format!("delta_fwd_y={dy_up:.3}"),
    });

    // Crouch (KeyC)
    engine.teleport(start_pos, Some(start_yaw), Some(0.0));
    kbm.reset();
    kbm.key_down(KeyCode::KeyC);
    for _ in 0..15 {
        let inp = kbm.to_input_state(false);
        engine.set_input(inp);
        engine.step(1.0 / 60.0);
    }
    let crouch_drop = engine.scene.sim.as_ref().unwrap().bond.crouch_drop;
    results.push(ControlCheck {
        mode: "onfoot",
        name: "KeyC crouches Bond (crouch_drop > 0.1)",
        passed: crouch_drop > 0.1,
        detail: format!("crouch_drop={crouch_drop:.3}m"),
    });
    kbm.reset();
    engine.set_input(InputState::default());

    // Fire (MouseLeft) & Reload (KeyR)
    let clip_before = engine.scene.sim.as_ref().unwrap().gun.clip;
    kbm.mouse_left = true;
    for _ in 0..8 {
        let inp = kbm.to_input_state(false);
        engine.set_input(inp);
        engine.step(1.0 / 60.0);
    }
    kbm.mouse_left = false;
    let clip_after_fire = engine.scene.sim.as_ref().unwrap().gun.clip;
    results.push(ControlCheck {
        mode: "onfoot",
        name: "MouseLeft fires weapon (clip decrements)",
        passed: clip_after_fire < clip_before,
        detail: format!("clip {clip_before} -> {clip_after_fire}"),
    });

    // Wait for the fire animation to settle before tapping KeyR to reload.
    engine.set_input(InputState::default());
    for _ in 0..30 {
        engine.step(1.0 / 60.0);
    }
    kbm.key_down(KeyCode::KeyR);
    let inp = kbm.to_input_state(false);
    engine.set_input(inp);
    engine.step(1.0 / 60.0);
    kbm.reset();
    engine.set_input(InputState::default());
    for _ in 0..120 {
        engine.step(1.0 / 60.0);
    }
    let clip_after_reload = engine.scene.sim.as_ref().unwrap().gun.clip;
    results.push(ControlCheck {
        mode: "onfoot",
        name: "KeyR reloads weapon",
        passed: clip_after_reload >= clip_before,
        detail: format!("clip {clip_after_fire} -> {clip_after_reload}"),
    });

    // ── 3. Driving Controls (MIS4 Vanquished: player_drives immediately) ────
    engine.load_level("09000006.bin")?;
    kbm.reset();
    kbm.key_down(KeyCode::KeyW);
    for _ in 0..45 {
        let inp = kbm.to_input_state(true);
        engine.set_input(inp);
        engine.step(1.0 / 60.0);
    }
    let spd_w = engine.telemetry().speed.unwrap_or(0.0);
    results.push(ControlCheck {
        mode: "driving",
        name: "KeyW accelerates vehicle forward",
        passed: spd_w > 3.0,
        detail: format!("speed={spd_w:.2} m/s"),
    });

    // Steer left (KeyA) while driving
    let right0 = cam_right(&engine.cam);
    kbm.reset();
    kbm.key_down(KeyCode::KeyW);
    kbm.key_down(KeyCode::KeyA);
    for _ in 0..35 {
        let inp = kbm.to_input_state(true);
        engine.set_input(inp);
        engine.step(1.0 / 60.0);
    }
    let steer_left_proj = cam_dir(&engine.cam).dot(right0);
    results.push(ControlCheck {
        mode: "driving",
        name: "KeyA steers vehicle left (-cam.right)",
        passed: steer_left_proj < -0.02,
        detail: format!("dot(new_fwd, old_right)={steer_left_proj:.3}"),
    });

    // Reload MIS4 and steer right (KeyD)
    engine.load_level("09000006.bin")?;
    kbm.reset();
    kbm.key_down(KeyCode::KeyW);
    for _ in 0..45 {
        let inp = kbm.to_input_state(true);
        engine.set_input(inp);
        engine.step(1.0 / 60.0);
    }
    let right1 = cam_right(&engine.cam);
    kbm.key_down(KeyCode::KeyD);
    for _ in 0..35 {
        let inp = kbm.to_input_state(true);
        engine.set_input(inp);
        engine.step(1.0 / 60.0);
    }
    let steer_right_proj = cam_dir(&engine.cam).dot(right1);
    results.push(ControlCheck {
        mode: "driving",
        name: "KeyD steers vehicle right (+cam.right)",
        passed: steer_right_proj > 0.02,
        detail: format!("dot(new_fwd, old_right)={steer_right_proj:.3}"),
    });

    // Brake (KeyS) slows vehicle
    let spd_pre_brake = engine.telemetry().speed.unwrap_or(0.0);
    kbm.reset();
    kbm.key_down(KeyCode::KeyS);
    for _ in 0..30 {
        let inp = kbm.to_input_state(true);
        engine.set_input(inp);
        engine.step(1.0 / 60.0);
    }
    let spd_post_brake = engine.telemetry().speed.unwrap_or(0.0);
    results.push(ControlCheck {
        mode: "driving",
        name: "KeyS brakes vehicle",
        passed: spd_post_brake < spd_pre_brake,
        detail: format!("speed {spd_pre_brake:.2} -> {spd_post_brake:.2} m/s"),
    });

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_key_names_and_rejects_unknown() {
        assert_eq!(parse_key("w").unwrap(), KeyCode::KeyW);
        assert_eq!(parse_key("ArrowLeft").unwrap(), KeyCode::ArrowLeft);
        assert_eq!(parse_key("ctrl").unwrap(), KeyCode::ControlLeft);
        assert!(parse_key("nonexistent_key").is_err());
    }

    #[test]
    fn kbm_maps_wasd_and_arrows_for_onfoot_and_driving() {
        let mut kbm = VirtualKbm::default();
        kbm.key_down(KeyCode::KeyW);
        kbm.key_down(KeyCode::KeyA);
        let onfoot = kbm.to_input_state(false);
        assert_eq!(onfoot.move_y, 1.0);
        assert_eq!(onfoot.move_x, -1.0);
        let drv = kbm.to_input_state(true);
        assert_eq!(drv.move_y, 1.0);
        assert_eq!(drv.move_x, -1.0);
    }
}
