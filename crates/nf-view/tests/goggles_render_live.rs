//! goggles-render: what the two goggle modes draw, through the real input path (the goggles key
//! = `Input::goggles_action`, the game's `Input_Action(p, 0xd, 4)` [M HUD_MonitorNightSight
//! 0x19e9d8:74]) and the real draw state (`Scene::frame_state`, `Scene::glow_targets`, the HUD
//! overlay). Evidence map: docs/systems/goggles.md.
//!
//! - night sight (mode 1): pane 7's fields + the one-tick white lamp at the viewer
//!   (`Light_Create(32.0, 2.5, …, kind 1)`) [M HUD_UpdateNightSightPane 0x19d218]; the laser
//!   sight's beam gfx swaps to 0x2000131 [M Player_LaserPointer 0x1a65d8:151-155];
//! - x-ray (mode 2): pane 15's fields [M HUD_UpdateXRayPane 0x1a1410]; the guards' body glow
//!   within 10.0 of the viewer [M BodyGlow_Update 0x1ac2c8]; the glists flagged 0x400 draw only
//!   in this mode [M View_AddObjects 0x1e62d8:38-47].
//!
//! Opt-in via NF_ISO. The disc grants the goggles item nowhere [D docs/systems/goggles.md]; the
//! key's request path carries no ownership gate in the decompile [M 0x19e9d8:82-84] and the
//! port's switch machine applies it as it stands — these tests press the key exactly as play does.

use nf_game::player::Input;
use nf_view::scene::{Scene, GLOW_GUARDS, GLOW_SLOT_BASE, NIGHT_BEAM_SLOT, SIGHT_BEAM_SLOT, XRAY_SLOT_BASE};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

const FAR_Y: f32 = -9999.0;

fn step(scene: &mut Scene, n: u32) {
    for _ in 0..n { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
}

/// Press the goggles key (every other tick) until the mode reads `want` — the first press
/// requests the item (latch clear), its draw arms night sight, later presses cycle 1 → 2 → 0.
fn press_to(scene: &mut Scene, want: u16) {
    for t in 0..900 {
        if scene.sim.as_ref().unwrap().goggle_mode() == want && t > 0 { break; }
        let mut i = Input::default();
        i.goggles_action = t % 2 == 0 && scene.sim.as_ref().unwrap().goggle_mode() != want;
        scene.sim.as_mut().unwrap().step_bond(i, false, false, false);
    }
    assert_eq!(scene.sim.as_ref().unwrap().goggle_mode(), want, "the key reached mode {want}");
}

/// A level loaded, the start cut-scene waited out and the battery charged (300 ticks · 3).
fn loaded(iso: &str, level: &str) -> Scene {
    let mut scene = nf_view::load(iso, level, false).expect("load");
    for _ in 0..3000 {
        step(&mut scene, 1);
        if !scene.sim.as_ref().unwrap().disabled { break; }
    }
    step(&mut scene, 300);
    scene
}

fn opts() -> nf_view::scene::ViewOpts {
    nf_view::scene::ViewOpts { reinforcements: false, firing: false, right: [1.0, 0.0, 0.0], up: [0.0, 1.0, 0.0] }
}

fn slot_index(scene: &Scene, slot: usize) -> Option<usize> {
    scene.attachments.iter().position(|a| a.tracer == Some(slot))
}

/// Night sight: the pane comes up with the game's fields, the lamp sits at the viewer every
/// tick, and the panes/lamp vanish with the mode.
#[test]
fn night_sight_raises_pane_7_and_the_viewer_lamp() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let mut scene = loaded(&iso, "07000005.bin");
    press_to(&mut scene, 1);
    step(&mut scene, 1);
    let s = scene.sim.as_ref().unwrap();
    let (night, xray, p) = s.goggle_panes();
    assert!(night && !xray, "pane 7 up, pane 15 down");
    assert_eq!(p.night_colour >> 8, 0x00ff00, "sprite 0 = (Rand(0x18)+7) | 0xff0000 [M 0x19d218:39-40]");
    assert!((7..31).contains(&(p.night_colour & 0xff)));
    let bar = p.night_bar.expect("the battery bar was written");
    let want = ((s.goggle_battery() + 1.0) * 0.000_555_555_57 * 200.0 + 28.0) as u16;
    assert!(bar.abs_diff(want) <= 1, "bar {bar} ≈ 28 + battery·200/1800 ({want})");
    let head = s.bond.head();
    let lamp = s.drones.lights.lights.iter().find(|l| l.radius == 32.0 && l.intensity == 2.5);
    let lamp = lamp.expect("Light_Create(32.0, 2.5, viewer, white, 0, 0, 1, 0) this tick [M 0x19d218:48-51]");
    assert_eq!((lamp.rgb, lamp.kind, lamp.channel), ([0xff; 3], 1, 0));
    let d = ((lamp.pos[0] - head[0]).powi(2) + (lamp.pos[1] - head[1]).powi(2) + (lamp.pos[2] - head[2]).powi(2)).sqrt();
    assert!(d < 0.5, "the lamp sits at the viewer (off by {d})");
    // The HUD draws more with the pane up than without (6 sprites, ≥ 4 quads after the zero-alpha
    // and missing-texture skips).
    let with = nf_view::hud::overlay(&scene, 4.0 / 3.0).v.len();
    // Sister (can fail): cycle 1 → 2 → 0 — pane 7 parks, no lamp is made after the drop.
    press_to(&mut scene, 2);
    press_to(&mut scene, 0);
    step(&mut scene, 2);
    let s = scene.sim.as_ref().unwrap();
    assert!(!s.goggle_panes().0 && !s.goggle_panes().1, "both panes park with the mode off");
    assert!(!s.drones.lights.lights.iter().any(|l| l.radius == 32.0 && l.intensity == 2.5), "no lamp with the mode off");
    let without = nf_view::hud::overlay(&scene, 4.0 / 3.0).v.len();
    assert!(with >= without + 24, "the night pane adds its quads ({with} vs {without})");
}

/// X-ray: the glow pool fills with the nearest guard inside 10.0, at the game's alpha, its parts
/// posed on the guard's bones; outside 10.0 or in night sight the pool stays empty.
#[test]
fn xray_glows_the_guards_inside_ten_units() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let mut scene = loaded(&iso, "07000005.bin");
    press_to(&mut scene, 2);
    assert!(scene.sim.as_ref().unwrap().goggle_panes().1, "pane 15 up");
    assert_eq!(scene.attachments.iter().filter(|a| a.tracer.is_some_and(|t| (GLOW_SLOT_BASE..GLOW_SLOT_BASE + GLOW_GUARDS * 16).contains(&t))).count(), GLOW_GUARDS * 16, "the glow pool is built");
    // A present guard with a drawn character; Bond stands 3.0 from it (eye height above).
    let (ci, di) = {
        let s = scene.sim.as_ref().unwrap();
        s.char_drone.iter().enumerate().filter_map(|(ci, d)| d.map(|d| (ci, d)))
            .find(|&(_, d)| s.drones.list[d].present && s.drones.list[d].enabled && s.drones.list[d].body_glow.is_some())
            .expect("a present guard")
    };
    let gpos = scene.sim.as_ref().unwrap().drones.list[di].pos;
    scene.sim.as_mut().unwrap().bond.pos = [gpos[0] + 3.0, gpos[1], gpos[2]];
    let fs = scene.frame_state(None, 0.0, opts());
    let glow = scene.glow_targets(|c| fs.posed.get(c).is_some_and(|p| p.is_some()) && !fs.hidden[c]);
    let t = glow.iter().find(|t| scene.sim.as_ref().unwrap().char_drone[t.ci] == Some(di)).expect("the near guard glows");
    let head = scene.sim.as_ref().unwrap().bond.head();
    let dist = ((gpos[0] - head[0]).powi(2) + (gpos[1] - head[1]).powi(2) + (gpos[2] - head[2]).powi(2)).sqrt();
    let count = scene.sim.as_ref().unwrap().drones.list[di].body_glow.as_ref().unwrap().count;
    assert_eq!(t.alpha, ((1.0 - dist * 0.1) * f32::from(count as i16) * 128.0 * 0.003_906_25) as i32 as u8, "alpha (1 − d/10)·count/2 [M 0x1ac2c8:224-226]");
    assert!(t.xray);
    let _ = ci;
    let g = glow.iter().position(|x| x == t).unwrap();
    for part in 0..16 {
        let k = slot_index(&scene, GLOW_SLOT_BASE + g * 16 + part).unwrap();
        assert!(fs.attach[k][3][1] > FAR_Y, "glow part {part} is posed");
        let p = fs.attach[k][3];
        let dg = ((p[0] - gpos[0]).powi(2) + (p[2] - gpos[2]).powi(2)).sqrt();
        assert!(dg < 1.5, "glow part {part} sits on the guard (xz off {dg})");
    }
    // Sister (can fail): 12.0 away (> 10.0) the guard drops out of the pool and its slots park.
    scene.sim.as_mut().unwrap().bond.pos = [gpos[0] + 12.0, gpos[1], gpos[2]];
    let glow = scene.glow_targets(|_| true);
    assert!(!glow.iter().any(|t| scene.sim.as_ref().unwrap().char_drone[t.ci] == Some(di)), "> 10.0 hides [M 0x1ac2c8:197-208]");
    // Sister: night sight glows nobody.
    scene.sim.as_mut().unwrap().bond.pos = [gpos[0] + 3.0, gpos[1], gpos[2]];
    press_to(&mut scene, 0);
    press_to(&mut scene, 1);
    scene.sim.as_mut().unwrap().bond.pos = [gpos[0] + 3.0, gpos[1], gpos[2]];
    assert!(scene.glow_targets(|_| true).is_empty(), "mode 1: no viewer mask bit, no glow");
    let fs = scene.frame_state(None, 0.0, opts());
    let k = slot_index(&scene, GLOW_SLOT_BASE + 1).unwrap();
    assert!(fs.attach[k][3][1] <= FAR_Y, "the pool parks outside x-ray");
}

/// A dead guard's glow fades: the count steps down once per tick from 0x100 [M 0x1ac2c8:96-136].
#[test]
fn a_dead_guards_glow_counts_down() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let mut scene = loaded(&iso, "07000005.bin");
    let di = {
        let s = scene.sim.as_ref().unwrap();
        (0..s.drones.list.len()).find(|&d| s.drones.list[d].present && s.drones.list[d].body_glow.is_some()).expect("a present guard")
    };
    assert_eq!(scene.sim.as_ref().unwrap().drones.list[di].body_glow.as_ref().unwrap().count, 0x100, "a live guard holds 0x100");
    scene.sim.as_mut().unwrap().drones.list[di].health = 0.0;
    step(&mut scene, 10);
    let s = scene.sim.as_ref().unwrap();
    if let Some(g) = s.drones.list[di].body_glow.as_ref() {
        // The AI may revive/remove the record on its own path; while it exists the count fell.
        assert!(g.count < 0x100 && g.count >= 0x100 - 10, "the dead owner's count fell by ≤ 10 ticks: {:#x}", g.count);
    }
    // Sister (can fail): a live guard elsewhere keeps 0x100.
    let other = (0..s.drones.list.len()).find(|&d| d != di && s.drones.list[d].present && s.drones.list[d].health > 0.0 && s.drones.list[d].body_glow.is_some());
    if let Some(o) = other { assert_eq!(s.drones.list[o].body_glow.as_ref().unwrap().count, 0x100); }
}

/// The 0x400-flagged placed glists draw only in x-ray: 0700000a's ten "laserbeam" triggers [D].
#[test]
fn xray_only_laser_beams_draw_only_in_xray() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let mut scene = loaded(&iso, "0700000a.bin");
    let slots: Vec<usize> = scene.attachments.iter().enumerate().filter(|(_, a)| a.tracer.is_some_and(|t| t >= XRAY_SLOT_BASE)).map(|(k, _)| k).collect();
    assert_eq!(slots.len(), 10, "0700000a: ten x-ray-only placements (the laserbeam triggers, kind 0xeb) [D]");
    let fs = scene.frame_state(None, 0.0, opts());
    assert!(slots.iter().all(|&k| fs.attach[k][3][1] <= FAR_Y), "mode 0: none draws");
    press_to(&mut scene, 2);
    let fs = scene.frame_state(None, 0.0, opts());
    let drawn: Vec<usize> = slots.iter().copied().filter(|&k| fs.attach[k][3][1] > FAR_Y).collect();
    assert!(!drawn.is_empty(), "mode 2: the live beams draw");
    for &k in &drawn { assert_eq!(fs.attach[k], scene.attachments[k].local, "at the placement matrix"); }
    // Sister (can fail): night sight hides them again.
    press_to(&mut scene, 0);
    press_to(&mut scene, 1);
    let fs = scene.frame_state(None, 0.0, opts());
    assert!(slots.iter().all(|&k| fs.attach[k][3][1] <= FAR_Y), "mode 1: none draws");
}

/// 07000003's "HOTOBJECT" breakable is the only x-ray-only placement there, and 07000005 has none [D].
#[test]
fn the_x_ray_only_census_per_level() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let count = |level: &str| {
        let scene = nf_view::load(&iso, level, false).expect("load");
        scene.attachments.iter().filter(|a| a.tracer.is_some_and(|t| t >= XRAY_SLOT_BASE)).count()
    };
    assert_eq!(count("07000003.bin"), 1, "07000003: HOTOBJECT (kind 0x20)");
    assert_eq!(count("07000014.bin"), 8, "07000014: eight laserbeam triggers (kind 0xea)");
    assert_eq!(count("07000005.bin"), 0, "sister: 07000005 has no 0x400 glist placement");
}

/// Night sight swaps the laser sight's beam from 0x2000133 to 0x2000131 — same matrix
/// [M Player_LaserPointer 0x1a65d8:151-155].
#[test]
fn night_sight_swaps_the_laser_sight_beam_sprite() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let mut scene = loaded(&iso, "07000005.bin");
    // The SG5 (0x16) given and drawn, then the ○ twin swap to the semi 0x17 — the record that
    // carries the sight bit (+0x68 & 0x8) [D; the fix_sg5_laser_live path].
    let (status, switch_to) = nf_view::panel::apply(&mut scene, &[("do".to_string(), "weapon".to_string()), ("id".to_string(), "0x16".to_string())]);
    assert!(status.contains("gave"), "the give equips: {status}");
    {
        let sim = scene.sim.as_mut().unwrap();
        let mut input = Input::default();
        input.switch_to = Some(switch_to.expect("the give returns the equip switch"));
        sim.step_bond(input, false, false, false);
    }
    step(&mut scene, 200);
    {
        let mut input = Input::default();
        input.mode_switch = true;
        scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
    }
    step(&mut scene, 200);
    scene.sim.as_mut().unwrap().bond.pitch = -0.5;
    step(&mut scene, 4);
    assert_eq!(scene.sim.as_ref().unwrap().gun.weapon, 0x17);
    assert!(scene.sim.as_ref().unwrap().laser_sight().is_some(), "the SG5 semi's sight is on");
    let (day, night) = (slot_index(&scene, SIGHT_BEAM_SLOT).expect("day beam slot"), slot_index(&scene, NIGHT_BEAM_SLOT).expect("night beam slot"));
    let fs = scene.frame_state(None, 0.0, opts());
    assert!(fs.attach[day][3][1] > FAR_Y && fs.attach[night][3][1] <= FAR_Y, "mode 0: 0x2000133 draws");
    press_to(&mut scene, 1);
    step(&mut scene, 200);
    assert_eq!(scene.sim.as_ref().unwrap().gun.weapon, 0x17, "the goggles item stowed back to the SG5");
    assert!(scene.sim.as_ref().unwrap().laser_sight().is_some());
    let fs = scene.frame_state(None, 0.0, opts());
    assert!(fs.attach[night][3][1] > FAR_Y, "mode 1: 0x2000131 draws");
    // Sister (can fail): the ordinary beam parks in night sight.
    assert!(fs.attach[day][3][1] <= FAR_Y, "mode 1: 0x2000133 parks");
}
