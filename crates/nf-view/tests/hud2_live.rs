//! hud-2: three HUD follow-ups, each pinned through the real loader (NF_ISO) with a sister.
//!
//! 1. The status-pane fade (`HUD_UpdateStatusPane` 0x19c310:262-270, `HUD_MPUpdateStatusPane`
//!    0x19ca88:77-79) rewrites only the text sprite's colour +0x10; the shadow colour +0x14 stays
//!    0xff, and `View_DrawSprites` 0x1e5290:166-182 / `Font_DrawText` 0x1eee48:96-98 draw the
//!    (+1, +1) copy in it with no alpha test — the letters fade, the shadow stays dark.
//! 2. The keycode message: `PCQWorm_Update` 0x1aee20 state 1 with param 0 != 0 posts
//!    `sprintf(Txt_BindLabel(0x2000014), KeyCodes[p0][0..4])` ("Obtained Lock Keycode %c%c%c%c")
//!    as a type-1 INFO message for 0xb4 ticks and sets `KeyCodes[p0*5+4] = 1` [M :24-31];
//!    `Lock_Update` 0x1ae3d8 case 3 posts the same message again when that lock's code is
//!    known [M :246-257]. On disc the param-0 PCs are 0700000a's two E-usable office computers
//!    (#683 p0 1, #684 p0 2), 07000011's and 07000012's "Computer to hack" (p0 1 / 5); the
//!    Q-Worm gadget PCs (0a #688, 0b #297) carry p0 = 0 and post nothing [D
//!    nf-formats/tests/view_object_census.rs].
//! 3. The security-camera view (pane 16): `Monitor_Activate` 0x1b0058 → the viewer shows the
//!    camera's matrix (`Monitor_Update` 0x1afcc8:155), the player is disabled
//!    (`Player_Disable(player, 0)`), panes 0 and 1 go off and pane 16 on [M 0x1b0058:43-49]; the
//!    crosshair hides for pane 16 [M HUD_UpdateCrossHair 0x19e6d0:24]; E walks FORWARD through
//!    the group's cameras and ends the view after the last [M 0x1afcc8:105-147]; the cooldown
//!    runs 9 → 4 → 0 one state per tick [M :68-83].

use nf_view::scene::{Scene, Sim};

type V3 = [f32; 3];

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load(level: &str) -> Option<Scene> {
    let iso = iso()?;
    Some(nf_view::load_with_carry_weapons_difficulty(&iso, level, false, None, &[], 3).unwrap_or_else(|e| panic!("load {level}: {e}")))
}

/// The overlay's quads drawn with texture `hash`: (screen box in the 512×448 frame, colour).
fn quads(scene: &Scene, hash: u32) -> Vec<([f32; 4], [f32; 4])> {
    let f = nf_view::hud::overlay(scene, 4.0 / 3.0);
    let Some(ti) = scene.hud_textures.iter().position(|(h, _)| *h == hash) else { return Vec::new() };
    let mut out = Vec::new();
    for r in f.runs.iter().filter(|r| r.0 == ti + 1) {
        for q in f.v[r.2 as usize..(r.2 + r.3) as usize].chunks(6) {
            let (a, b) = (q[0].p, q[2].p);
            out.push(([(a[0] + 1.0) * 256.0, (1.0 - a[1]) * 224.0, (b[0] + 1.0) * 256.0, (1.0 - b[1]) * 224.0], q[0].c));
        }
    }
    out
}

const FONT: u32 = 0x0300_0003;

/// Place Bond once beside the target (the documented place-once stand-in of the walkreal
/// tests) and let him ground; the candidate landing nearest the target wins.
fn place_near(sim: &mut Sim, pos: V3) -> V3 {
    let offs: [V3; 5] = [[0.0, 1.2, 1.4], [0.0, -0.5, 1.0], [0.0, -0.5, -1.0], [1.0, -0.5, 0.0], [-1.0, -0.5, 0.0]];
    let mut best = sim.bond.pos;
    let mut best_score = f32::MAX;
    for o in offs {
        sim.bond.pos = [pos[0] + o[0], pos[1] + o[1], pos[2] + o[2]];
        for _ in 0..100 { sim.step_bond(Default::default(), false, false, false); }
        let score = ((sim.bond.pos[0] - pos[0]).powi(2) + (sim.bond.pos[2] - pos[2]).powi(2)).sqrt() + (sim.bond.pos[1] - pos[1]).abs();
        if score < best_score { best_score = score; best = sim.bond.pos; }
    }
    sim.bond.pos = best;
    for _ in 0..60 { sim.step_bond(Default::default(), false, false, false); }
    sim.bond.pos
}

/// Aim at the target and pulse E one tick in six until `done` (the real use input).
fn aim_use(sim: &mut Sim, target: V3, done: &dyn Fn(&Sim) -> bool) -> bool {
    for k in 0..360 {
        let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
        let (dx, dz, dy) = (target[0] - eye[0], target[2] - eye[2], target[1] - eye[1]);
        let err = dx.atan2(dz) - sim.bond.yaw;
        let want_pitch = (dy / (dx * dx + dz * dz).sqrt().max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let input = nf_game::player::Input { turn: err.clamp(-0.08, 0.08), look: (want_pitch - sim.bond.pitch).clamp(-0.08, 0.08), ..Default::default() };
        sim.step_bond(input, false, false, k % 6 == 0);
        if done(sim) { return true; }
    }
    false
}

/// One E press (a fresh edge) followed by idle ticks.
fn press(sim: &mut Sim) {
    sim.step_bond(Default::default(), false, false, false);
    sim.step_bond(Default::default(), false, false, true);
    sim.step_bond(Default::default(), false, false, false);
}

// ---------------------------------------------------------------------------------------------
// 1. The fade keeps the shadow dark.
// ---------------------------------------------------------------------------------------------

#[test]
fn status_message_fade_keeps_the_shadow_opaque() {
    let Some(mut sc) = load("07000005.bin") else { eprintln!("NF_ISO unset — skipping"); return };
    let info = |sc: &Scene| -> (Vec<f32>, Vec<f32>) {
        // The INFO text row (y 129, centred): shadow glyphs are black, the letters 0x7d6d59.
        let q: Vec<_> = quads(sc, FONT).into_iter().filter(|(b, _)| b[1] > 115.0 && b[3] < 150.0).collect();
        let sh = q.iter().filter(|(_, c)| c[0] == 0.0 && c[1] == 0.0 && c[2] == 0.0).map(|(_, c)| c[3]).collect();
        let tx = q.iter().filter(|(_, c)| c[0] > 0.5).map(|(_, c)| c[3]).collect();
        (sh, tx)
    };
    let opaque = (0xffu32 >> 1) as f32 / 128.0;
    {
        let s = sc.sim.as_mut().unwrap();
        let text = s.lang.as_ref().unwrap().bind(0x200_0021).unwrap().trim().to_string();
        s.hud.info_msgs.push_back(nf_view::scene::Msg::new(text, 120.0));
    }
    // Sister: steady (time 120 ≥ 15) — both at full alpha.
    let (sh, tx) = info(&sc);
    assert!(!sh.is_empty() && sh.len() == tx.len(), "shadow/text pairs: {} / {}", sh.len(), tx.len());
    assert!(sh.iter().chain(&tx).all(|&a| (a - opaque).abs() < 1e-4), "steady row is opaque: {sh:?} {tx:?}");
    // The fade: time 5 → the text alpha byte = 5·17 = 85 [M 0x19c310:264-265], the shadow 0xff.
    sc.sim.as_mut().unwrap().hud.info_msgs.front_mut().unwrap().time = 5.0;
    let (sh, tx) = info(&sc);
    let faded = (85u32 >> 1) as f32 / 128.0;
    assert!(tx.iter().all(|&a| (a - faded).abs() < 1e-4), "the letters fade to 85: {tx:?}");
    assert!(!sh.is_empty() && sh.iter().all(|&a| (a - opaque).abs() < 1e-4),
        "the shadow keeps +0x14 = 0xff while the text fades [M 0x19c310:262-270]: {sh:?}");
    // Sister: the old rule (shadow capped at the text alpha) would read 85 here.
    assert!(sh.iter().all(|&a| (a - faded).abs() > 0.1));
    // Alpha 0 (time < 1/17): the letters are gone, the shadow copy still draws [M 0x1e5290:166-182
    // has no alpha test on the text path].
    sc.sim.as_mut().unwrap().hud.info_msgs.front_mut().unwrap().time = 0.01;
    let (sh, tx) = info(&sc);
    assert!(tx.is_empty(), "no letters at alpha 0: {tx:?}");
    assert!(!sh.is_empty(), "the shadow still draws at text alpha 0");
}

// ---------------------------------------------------------------------------------------------
// 2. The keycode message.
// ---------------------------------------------------------------------------------------------

#[test]
fn office_pc_install_posts_the_keycode_and_the_keypad_reposts_it() {
    let Some(mut sc) = load("0700000a.bin") else { eprintln!("NF_ISO unset — skipping"); return };
    let sim = sc.sim.as_mut().unwrap();
    assert_eq!(sim.lang.as_ref().unwrap().bind(0x200_0014).map(str::trim), Some("Obtained Lock Keycode %c%c%c%c"),
        "label 0x2000014 [D USATxt]");
    // The Q-Worm gadget PC carries no keycode (p0 = 0) — its install posts nothing [D #688].
    let qpc = sim.drones.useobjs.pcs.iter().find(|p| p.ch == 0x48).expect("Q-WORM_PC_1 [D #688]");
    assert_eq!(qpc.keys, 0, "the gadget PC reveals no code [D #688 param 0 = 0]");
    let pi = sim.drones.useobjs.pcs.iter().position(|p| p.keys == 1 && p.e_usable != 0).expect("office PC #683 (p0 1, E) [D]");
    let target = sim.drones.useobjs.pcs[pi].sph_centre;
    let code = sim.drones.key_codes.code(1);
    let want = format!("Obtained Lock Keycode {}", code.iter().map(|&c| c as char).collect::<String>());
    // Sister: nothing revealed, nothing posted before the use.
    assert!(!sim.drones.key_codes.shown(1));
    assert!(!sim.hud.info_msgs.iter().any(|m| m.label == 0x200_0014));
    place_near(sim, target);
    assert!(aim_use(sim, target, &|s| s.drones.useobjs.pcs[pi].state >= 2),
        "E on the office PC runs its install [M PCQWorm_Activate 0x1b0840 → 0x1aee20 state 1]");
    for _ in 0..2 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.drones.key_codes.shown(1), "KeyCodes[1*5+4] = 1 [M 0x1aee20:31]");
    let posted: Vec<_> = sim.hud.info_msgs.iter().filter(|m| m.label == 0x200_0014).collect();
    assert_eq!(posted.len(), 1, "one keycode message");
    assert_eq!(posted[0].text, want, "the sprintf'd code chars [M 0x1aee20:24-29]");
    assert!(posted[0].life == 180.0, "0xb4 ticks [M Text_AddMsg(…, 0xb4)]");
    assert!(!sim.drones.key_codes.shown(2), "sister: the other office PC's code stays unknown");

    // The keypad lock with code index 1: arming it posts the known code again [M 0x1ae3d8:246-257].
    sim.hud.info_msgs.clear();
    let li = sim.drones.locks.iter().position(|l| l.kind == 0 && l.code == 1).expect("a keypad lock on code 1 [D]");
    let lpos = sim.drones.locks[li].pos;
    place_near(sim, lpos);
    assert!(aim_use(sim, lpos, &|s| s.drones.keypad.is_some()), "E arms the keypad [M Lock_Activate 0x1aed00]");
    sim.step_bond(Default::default(), false, false, false);
    assert!(sim.drones.keypad.as_ref().unwrap().1.code_shown);
    assert!(sim.hud.info_msgs.iter().any(|m| m.label == 0x200_0014 && m.text == want),
        "the known code is posted again on the keypad [M 0x1ae3d8:246-257]: {:?}", sim.hud.info_msgs.iter().map(|m| &m.text).collect::<Vec<_>>());
}

// ---------------------------------------------------------------------------------------------
// 3. The security-camera view.
// ---------------------------------------------------------------------------------------------

#[test]
fn sec_cam_lamp_blinks_on_a_32_frame_cycle() {
    assert_eq!(nf_view::hud::sec_cam_lamp(0), 0x7f7f_7f78);
    assert_eq!(nf_view::hud::sec_cam_lamp(15), 0x7f7f_7f78);
    assert_eq!(nf_view::hud::sec_cam_lamp(16), 0x7f7f_7f00, "alpha 0 for frames 16..31 [M 0x1a1538:11-19]");
    assert_eq!(nf_view::hud::sec_cam_lamp(32), 0x7f7f_7f78);
}

#[test]
fn monitor_console_shows_the_camera_view_and_e_walks_forward_then_exits() {
    let Some(mut sc) = load("0700000a.bin") else { eprintln!("NF_ISO unset — skipping"); return };
    let (mi, mpos) = {
        let sim = sc.sim.as_ref().unwrap();
        let mi = sim.drones.monitors.list.iter().position(|m| m.group == 10).expect("0700000a's console #783 (group 10) [D]");
        (mi, sim.drones.monitors.list[mi].pos)
    };
    let group: Vec<usize> = {
        let sim = sc.sim.as_ref().unwrap();
        sim.drones.sensors.list.iter().enumerate().filter(|(_, s)| s.listed && s.monitor_id == 10).map(|(i, _)| i).collect()
    };
    assert_eq!(group.len(), 2, "two group-10 cameras [D]");
    // Sister: before the use — Bond's view, the HUD's ammo text and crosshair, no pane 16.
    {
        let sim = sc.sim.as_mut().unwrap();
        place_near(sim, mpos);
        assert!(sim.sec_cam().is_none() && sim.view_override().is_none());
    }
    assert!(quads(&sc, 0x0300_00aa).is_empty(), "no pane-16 brackets before the view");
    assert!(!quads(&sc, 0x0300_0026).is_empty(), "the crosshair shows before the view");
    let ammo_text = |sc: &Scene| quads(sc, FONT).into_iter().filter(|(b, _)| b[0] > 380.0 && b[1] > 390.0 && b[3] < 425.0).count();
    assert!(ammo_text(&sc) > 0, "the ammo count shows before the view");

    let sim = sc.sim.as_mut().unwrap();
    assert!(aim_use(sim, mpos, &|s| s.drones.monitors.list[mi].state == 2), "E enters the view [M Monitor_Activate 0x1b0058]");
    let w1 = sim.sec_cam().expect("viewing");
    assert_eq!(w1, group[0], "the first group camera in CameraList order [M 0x1b0058:19-31; LList_Add 0x1cca28 appends]");
    sim.step_bond(Default::default(), false, false, false);
    // The view is the camera's pose [M 0x1afcc8:155].
    let (p, d) = sim.view_override().expect("the view override");
    let s = &sim.drones.sensors.list[w1];
    assert_eq!(p, s.pos);
    assert_eq!(d, [s.matrix[2][0], s.matrix[2][1], s.matrix[2][2]]);
    // The disabled player: forward + fire move nothing, shoot nothing [M Player_Disable 0x195f70].
    let (pos0, shots0) = (sim.bond.pos, sim.drones.player_bullets.len());
    let walk = nf_game::player::Input { forward: 1.0, turn: 0.05, ..Default::default() };
    for _ in 0..30 { sim.step_bond(walk, true, false, false); }
    let moved = ((sim.bond.pos[0] - pos0[0]).powi(2) + (sim.bond.pos[2] - pos0[2]).powi(2)).sqrt();
    assert!(moved < 1e-3, "Bond does not walk while viewing: {moved}");
    assert_eq!(sim.drones.player_bullets.len(), shots0, "no shot while viewing");
    assert_eq!(sim.sec_cam(), Some(w1), "held input does not cycle");
    // The HUD rules.
    assert_eq!(quads(&sc, 0x0300_00aa).len(), 4, "pane 16's four corner brackets [L SecCamSprInfo 1..4]");
    assert_eq!(quads(&sc, 0x0300_00ad).len(), 1, "pane 16's full-screen wash [L SecCamSprInfo 0]");
    assert!(quads(&sc, 0x0300_0026).is_empty(), "the crosshair hides for pane 16 [M 0x19e6d0:24]");
    assert_eq!(ammo_text(&sc), 0, "the ammo pane is off [M 0x1b0058:48]");
    // E → the next camera, forward [M 0x1afcc8:105-137].
    let sim = sc.sim.as_mut().unwrap();
    press(sim);
    assert_eq!(sim.sec_cam(), Some(group[1]), "the next E shows the group's next camera");
    // E on the last camera ends the view: state 9, then one state per tick to 4, then 0.
    press(sim);
    assert!(sim.sec_cam().is_none() && sim.view_override().is_none(), "the last camera's E ends the view [M 0x1afcc8:138-147]");
    let st = sim.drones.monitors.list[mi].state;
    assert!((4..=8).contains(&st), "the cooldown runs down from 9 [M 0x1afcc8:68-83]: {st}");
    for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
    assert_eq!(sim.drones.monitors.list[mi].state, 0, "back to idle");
    assert!(!quads(&sc, 0x0300_0026).is_empty() && ammo_text(&sc) > 0, "crosshair and ammo pane return");
    assert!(quads(&sc, 0x0300_00aa).is_empty(), "pane 16 off after the exit");
    // Sister: the player walks again.
    let sim = sc.sim.as_mut().unwrap();
    let pos1 = sim.bond.pos;
    for _ in 0..30 { sim.step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false); }
    let moved = ((sim.bond.pos[0] - pos1[0]).powi(2) + (sim.bond.pos[2] - pos1[2]).powi(2)).sqrt();
    assert!(moved > 0.05, "Bond walks after the view: {moved}");
}

#[test]
fn cooldown_counts_one_state_per_tick() {
    use nf_game::monitor::{Monitor, Monitors};
    let mut m = Monitors { list: vec![Monitor { placement: 0, pos: [0.0; 3], group: 1, health: 0.0, state: 9, watched: None }] };
    let mut sensors = nf_game::sensor::Sensors { list: Vec::new(), heard: Default::default() };
    let mut ch = nf_game::triggers::Channels::default();
    let mut seen = Vec::new();
    for f in 0..8 { m.tick(&mut sensors, &mut ch, f); seen.push(m.list[0].state); }
    assert_eq!(seen, vec![8, 7, 6, 5, 4, 0, 0, 0], "states 5..9 decrement, 4 resets [M 0x1afcc8:68-83]");
}
