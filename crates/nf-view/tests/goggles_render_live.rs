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

/// The night noise's v jitter rolls under the runtime sprite's +0x2a word — the texture's height
/// (`hashtable_set_sprite` 0x1ca3d8), not the template's dv [M HUD_UpdateNightSightPane
/// 0x19d218:43-47; Sprite_Create2 0x1de3d8]. 0x0300004A is 64 tall on the disc [D], so v ∈ 0..63.
#[test]
fn night_noise_jitter_rolls_under_the_texture_height() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let mut scene = loaded(&iso, "07000005.bin");
    let tex = scene.hud_textures.iter().find(|(h, _)| *h == 0x0300_004a).map(|(_, t)| (t.width, t.height));
    assert_eq!(tex, Some((64, 64)), "the night noise texture 0x0300004A is 64x64 [D]");
    assert_eq!(nf_view::scene::GOGGLE_NIGHT_SPRITE0_H as u32, tex.unwrap().1, "the jitter range word is the texture height");
    press_to(&mut scene, 1);
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..400 {
        step(&mut scene, 1);
        seen.insert(scene.sim.as_ref().unwrap().goggle_panes().2.night_v);
    }
    let max = *seen.iter().max().unwrap();
    assert!(max < 63, "v = Rand(64 − 1) stays below 63 (max {max})");
    // Sister (can fail): the rolls really vary over the range — not stuck at 0, and past the
    // half (the old range 255 would spread them up to 253).
    assert!(seen.len() > 20 && max > 31, "the jitter spreads over 0..63 ({} values, max {max})", seen.len());
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
    // A static beam draws at its placement matrix, a keyed one at its spline pose (#606/#614/#625
    // carry 3-key lists [D]; `Trigger_Update` 0x1b1a78:60-70).
    let s = scene.sim.as_ref().unwrap();
    for &k in &drawn {
        let pi = scene.attachments[k].tracer.unwrap() - XRAY_SLOT_BASE;
        match s.drones.triggers.iter().find(|t| t.placement == pi).and_then(|t| t.motion.as_ref()) {
            Some(m) => assert_eq!(fs.attach[k], m.matrix, "#{pi} at its spline pose"),
            None => assert_eq!(fs.attach[k], scene.attachments[k].local, "#{pi} at the placement matrix"),
        }
    }
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

/// night-alpha: night sight quarters a gas puff's alpha byte. Every `Gas_Init` 0x1aacc8 puff
/// carries obj+0xF0 0x20000 (`|= 0x20020120` at :83), and `View_SetupRenderModes_Tweak`
/// 0x1e5888:12-17 hands `obj+0x106 >> 2` to `psiSetTweakARGB` while viewer+0x236 == 1. The puff's
/// GPU colour slot (the shader's alpha factor) and its CPU vertex alpha both carry it; x-ray and
/// off keep the full byte. The factor multiplies the mesh's own vertex alpha (byte/255, the
/// Tweak's float [M 0x1e5888:46-49]) on both paths, so the CPU alpha is vertex_a × factor.
#[test]
fn night_sight_quarters_the_gas_puff_alpha() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = loaded(&iso, "07000005.bin");
    // A long-lived white puff 2 units ahead of Bond (kind 0: damped, no drift), alpha 0xc8.
    {
        let sim = scene.sim.as_mut().unwrap();
        let yaw = sim.bond.yaw;
        let p = [sim.bond.pos[0] + yaw.sin() * 2.0, sim.bond.head()[1], sim.bond.pos[2] + yaw.cos() * 2.0];
        let mut rng = nf_game::drone::Rng::new(0x21d);
        sim.drones.gas.init(&mut rng, p, 0.0, [0.0; 3], 0.5, 0.5, 0, [0xff, 0xff, 0xff, 0xc8], 6000.0);
    }
    // (the pool's alpha byte, the GPU colour slot alpha, the CPU vertex alpha, the mesh's vertex alpha)
    let read = |scene: &Scene| -> (u8, f32, u8, u8) {
        let sim = scene.sim.as_ref().unwrap();
        let pi = sim.drones.gas.puffs.iter().position(|g| g.life > 3000.0).expect("the test puff lives");
        let g = &sim.drones.gas.puffs[pi];
        let ai = scene.attachments.iter().position(|a| a.gas == Some((pi as u16, g.mesh))).expect("a gas draw slot reads the puff");
        let (layout, _) = scene.skin_layout();
        let (_, cols) = scene.skin_palette(&layout, None, 240.0, opts(), None);
        let gpu = cols[(layout.col_att + ai as u32) as usize][3];
        let posed = scene.posed_vertices(None, 240.0, opts());
        let (cpu, base): (Vec<u8>, Vec<u8>) = scene.char_verts.iter().enumerate()
            .filter(|(_, cv)| cv.attachment == Some(ai as u32))
            .map(|(i, _)| (posed[i].rgba[3], scene.vertices[scene.char_vertex_start + i].rgba[3])).unzip();
        assert!(!cpu.is_empty(), "the puff's slot has vertices");
        assert!(cpu.iter().all(|&a| a == cpu[0]) && base.iter().all(|&a| a == base[0]));
        (g.alpha_byte(), gpu, cpu[0], base[0])
    };
    let cpu_of = |base: u8, f: f32| (base as f32 * f).round() as u8;
    let (b0, gpu0, cpu0, v0) = read(&scene);
    assert!(b0 > 0xc0, "the puff's alpha is still near its 0xc8 seed: {b0:#x}");
    assert!((gpu0 - b0 as f32 / 255.0).abs() < 1.5 / 255.0, "mode 0: full alpha, got {gpu0}");
    assert_eq!(cpu0, cpu_of(v0, gpu0), "mode 0: CPU alpha = vertex alpha × the full factor");

    press_to(&mut scene, 1);
    let (b1, gpu1, cpu1, v1) = read(&scene);
    assert_eq!(gpu1, (b1 >> 2) as f32 / 255.0, "night sight: the GPU alpha factor is obj+0x106 >> 2");
    assert_eq!(cpu1, cpu_of(v1, (b1 >> 2) as f32 / 255.0), "night sight: the CPU vertex alpha carries obj+0x106 >> 2");
    assert!(cpu1 < cpu0 / 2, "night sight: the CPU alpha drops with the quarter ({cpu1} vs {cpu0})");
    // Sister (can fail): the quarter is far from the full byte, so the old code's full alpha fails above.
    assert!(gpu1 < b1 as f32 / 255.0 * 0.3, "night sight must not keep the full alpha ({gpu1} vs {})", b1 as f32 / 255.0);

    press_to(&mut scene, 2);
    let (b2, gpu2, _, _) = read(&scene);
    assert!((gpu2 - b2 as f32 / 255.0).abs() < 1.5 / 255.0, "x-ray keeps the full alpha (the gate is mode 1 only), got {gpu2}");
}
