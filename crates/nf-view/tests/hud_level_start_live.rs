//! The level-start HUD and the pause HUD against the decompile (hud-start):
//! - the weapon-name label at the start: `Player_InitWeapon` holds 0x47 [M 0x1a7ef8:137] and the
//!   first `Player_Update` runs `HUD_Update` before `Player_Weapon` [M 0x193600:79 vs :412], so the
//!   ammo pane arms `BL+0x95E = FRAME_RATE` once [M 0x19a7f0:212-223] — the name shows ~60 ticks;
//! - the entry fade's cover: the fade byte goes raw into the GS alpha (0x80 = 1.0) [M 0x1bada0,
//!   0x1181b0], so the screen is black while the byte is ≥ 128; the fade draws UNDER the HUD
//!   [L PCSX2 starts: iris + label bright over black];
//! - the objective banner's 60-tick opening flash [M 0x19c310:201-213];
//! - the pause: the HUD stays under the page and the health pane's fade factor is forced to 1
//!   [M 0x19b9c8:116-123, 0x193600:73-79].
//! The ISO tests are opt-in via `NF_ISO`.

use nf_view::hud::{fade_cover, objective_rows};

/// (in the flash window, status label drawn) from `objective_rows`: the flash drops the label's
/// shadow and blinks its colour.
fn objective_flash(complete: bool, time: f32) -> (bool, bool) {
    let (st, shadow, _) = objective_rows(complete, time);
    (!shadow, st & 0xff != 0)
}

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Font glyph quads (font 2 sheet 0x03000003) whose centre lies in the 512×448 box.
fn glyphs_in(scene: &nf_view::scene::Scene, f: &nf_view::gpu::HudFrame, x0: f32, y0: f32, x1: f32, y1: f32) -> usize {
    let Some(ti) = scene.hud_textures.iter().position(|(h, _)| *h == 0x0300_0003) else { return 0 };
    let mut n = 0;
    for r in f.runs.iter().filter(|r| r.0 == ti + 1) {
        for q in f.v[r.2 as usize..(r.2 + r.3) as usize].chunks(6) {
            let (a, b) = (q[0], q[2]);
            let cx = ((a.p[0] + 1.0) * 256.0 + (b.p[0] + 1.0) * 256.0) / 2.0;
            let cy = ((1.0 - a.p[1]) * 224.0 + (1.0 - b.p[1]) * 224.0) / 2.0;
            if cx > x0 && cx < x1 && cy > y0 && cy < y1 { n += 1; }
        }
    }
    n
}

fn name_glyphs(scene: &nf_view::scene::Scene, f: &nf_view::gpu::HudFrame) -> usize { glyphs_in(scene, f, 300.0, 422.0, 512.0, 446.0) }
fn ammo_glyphs(scene: &nf_view::scene::Scene, f: &nf_view::gpu::HudFrame) -> usize { glyphs_in(scene, f, 380.0, 395.0, 512.0, 420.0) }

#[test]
fn the_entry_fade_cover_is_the_gs_alpha_byte_over_128() {
    assert_eq!(fade_cover(1.0), 1.0);
    assert_eq!(fade_cover(0.0), 0.0);
    // Byte 128 and up cover the screen completely.
    assert_eq!(fade_cover(128.0 / 255.0 + 1e-4), 1.0);
    assert_eq!(fade_cover(0.75), 1.0, "byte 191 is still full black");
    // Below it the cover is byte/128.
    assert!((fade_cover(0.25) - 63.0 / 128.0).abs() < 1e-6, "byte 63 → 63/128");
    // The 45-tick entry ramp (alpha (45-n)/45): black through tick 22, lighter from tick 23.
    let cover = |n: u32| fade_cover((45 - n) as f32 / 45.0);
    for n in 0..=22 { assert_eq!(cover(n), 1.0, "tick {n} is black"); }
    assert!(cover(23) < 1.0 && cover(23) > 0.95, "tick 23 starts lightening ({})", cover(23));
    assert!((cover(36) - 51.0 / 128.0).abs() < 1e-6, "tick 36: byte 51");
    // Sister: the old reading (cover = alpha) differs over the whole black half.
    assert!(cover(20) - (25.0 / 45.0) > 0.4, "the old cover was 0.56 at tick 20");
}

#[test]
fn a_new_objective_blinks_its_label_alone_for_60_ticks() {
    // Flash window: time + 60 > 300 → time in (240, 300].
    for t in [300.0f32, 280.0, 260.0, 241.0] { assert!(objective_flash(false, t).0, "t {t} flashes"); }
    for t in [240.0f32, 200.0, 10.0] { assert_eq!(objective_flash(false, t), (false, true), "t {t} steady with the label on"); }
    // The blink: on while fmod(t, 40) < 20.
    assert_eq!(objective_flash(false, 299.0), (true, true), "fmod 19 → on");
    assert_eq!(objective_flash(false, 300.0), (true, false), "fmod 20 → off");
    assert_eq!(objective_flash(false, 270.0), (true, false), "fmod 30 → off");
    assert_eq!(objective_flash(false, 250.0), (true, true), "fmod 10 → on");
    // OBJECTIVE COMPLETE uses its own literal 180.
    assert!(objective_flash(true, 150.0).0 && !objective_flash(true, 120.0).0);
    // Sister: a new objective at 150 is long past its window.
    assert!(!objective_flash(false, 150.0).0);
}

#[test]
fn the_level_start_shows_the_weapon_name_for_60_ticks() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    for level in ["07000005", "07000011", "0700000c"] {
        let mut scene = nf_view::load(&iso, &format!("{level}.bin"), false).expect("load");
        let (req, name_len) = {
            let s = scene.sim.as_ref().unwrap();
            let lang = s.lang.as_ref().expect("USATxt.dat");
            let req = s.gun.switch_request;
            let n = lang.bind(s.weapons.name_label(req as usize)).unwrap_or("").chars().filter(|c| !c.is_whitespace()).count();
            assert_eq!(s.gun.weapon, 0x47, "{level}: nothing held at the start [M 0x1a7ef8:137]");
            assert_eq!(s.hud.weapon_label_timer, 60, "{level}: the first HUD_Update's arm [M 0x19a7f0:216]");
            (req, n)
        };
        assert!(name_len > 0, "{level}: the requested weapon {req:#x} has a name");
        for tick in 0..62 {
            scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
            let f = nf_view::hud::overlay(&scene, 4.0 / 3.0);
            let (s, n, a) = (scene.sim.as_ref().unwrap(), name_glyphs(&scene, &f), ammo_glyphs(&scene, &f));
            if tick == 0 { assert_eq!(s.gun.weapon, req, "{level}: the no-holster draw swaps on the first tick"); }
            if tick < 59 {
                assert_eq!(s.hud.weapon_label_timer as i32, 59 - tick, "{level} tick {tick}: one arm, then the decay");
                assert_eq!(n, 2 * name_len, "{level} tick {tick}: the name (shadow + glyph per character)");
                assert_eq!(a, 0, "{level} tick {tick}: no ammo count while the name shows");
            } else {
                // Sister: the label is gone and the ammo count is back.
                assert_eq!(s.hud.weapon_label_timer, 0, "{level} tick {tick}");
                assert!(a >= 3, "{level} tick {tick}: the ammo count at rest ({a} glyphs)");
            }
        }
    }
}

#[test]
fn the_first_label_shows_the_rounds_strip_and_it_remains_after() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000011.bin", false).expect("load");
    let strips = |scene: &nf_view::scene::Scene, f: &nf_view::gpu::HudFrame| -> Vec<[f32; 4]> {
        let Some(ti) = scene.hud_textures.iter().position(|(h, _)| *h == 0x0300_0128) else { return Vec::new() };
        let mut out = Vec::new();
        for r in f.runs.iter().filter(|r| r.0 == ti + 1) {
            for q in f.v[r.2 as usize..(r.2 + r.3) as usize].chunks(6) {
                out.push([(q[0].p[0] + 1.0) * 256.0, (1.0 - q[0].p[1]) * 224.0, (q[2].p[0] + 1.0) * 256.0, (1.0 - q[2].p[1]) * 224.0]);
            }
        }
        out
    };
    // The 07000011 f42 capture: during the first 60-tick weapon-name label the ammo bar draws with
    // it — the strip sprites are still in their created state (template layers 0x1c/0x1d, the
    // template rect pane (496,394) + (−25,−64) = (471,330)..(496,394), 25×64) [D AmmoSprInfo[5]/[6];
    // M HUD_CreateDefault 0x199b70 copies the template layer and adds the pane origin].
    for tick in 0..59 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let f = nf_view::hud::overlay(&scene, 4.0 / 3.0);
        let bar = strips(&scene, &f);
        assert!(bar.len() >= 2, "tick {tick}: the strips draw during the first label ({bar:?})");
        for b in &bar {
            assert!((b[0] - 471.0).abs() < 0.5 && (b[1] - 330.0).abs() < 0.5 && (b[2] - 496.0).abs() < 0.5 && (b[3] - 394.0).abs() < 0.5, "tick {tick}: the created rect {b:?}");
        }
    }
    // Sister: after the timer the label is gone and the bar REMAINS — the pane fn's timer==0 path
    // has now positioned it [M 0x19a7f0:248-269] (a rest draw, so at least the background column).
    for tick in 59..63 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let s = scene.sim.as_ref().unwrap();
        let f = nf_view::hud::overlay(&scene, 4.0 / 3.0);
        assert_eq!(s.hud.weapon_label_timer, 0, "tick {tick}: the label ran out");
        assert!(!strips(&scene, &f).is_empty(), "tick {tick}: the bar remains after the label");
    }
    // And once the pane has rested, a switch takes the strips off again (layer 0xff [M 0x19ad0c /
    // 0x19ae48]; the 2026-10-12 live pass: a mid-game switch shows the name alone).
    scene.sim.as_mut().unwrap().hud.weapon_label_timer = 30;
    let f = nf_view::hud::overlay(&scene, 4.0 / 3.0);
    assert!(strips(&scene, &f).is_empty(), "a later switch hides the strips again");
}

#[test]
fn the_entry_fade_draws_under_the_hud() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load");
    for _ in 0..10 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let entry = scene.entry_fade_hud(4.0 / 3.0);
    assert_eq!(entry.v.len(), 6, "one full-screen quad");
    assert!(entry.v.iter().all(|v| v.c[3] == 1.0 && v.c[0] == 0.0), "tick 10 is solid black");
    // The script overlay no longer carries the entry fade (it would cover the HUD).
    assert!(scene.script_overlay_hud(4.0 / 3.0).v.is_empty(), "no letterbox / script fade on 05 at tick 10");
    let f = nf_view::shot_hud(&scene, 4.0 / 3.0, None, false);
    let hud_len = nf_view::hud::overlay(&scene, 4.0 / 3.0).v.len();
    assert!(hud_len > 0);
    assert_eq!(f.v.len(), hud_len + 6, "HUD + the fade quad");
    // The fade quad comes FIRST (frame order = draw order) — the HUD draws over it.
    assert!(f.v[..6].iter().all(|v| v.c[3] == 1.0 && v.c[0] == 0.0 && v.c[1] == 0.0), "the first quad is the black fade");
    // Sister: the label glyphs sit in the frame after it, still drawn at tick 10.
    assert!(name_glyphs(&scene, &f) > 0, "the weapon name is in the frame over the fade");
}

#[test]
fn the_opening_objective_shows_its_label_before_its_text() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load");
    let mut seen_flash = false;
    let mut seen_body = false;
    for tick in 0..200 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let Some((time, complete)) = scene.sim.as_ref().unwrap().hud.objective_msgs.front().map(|m| (m.time, m.status == 0x0200_0053)) else { continue };
        let f = nf_view::hud::overlay(&scene, 4.0 / 3.0);
        let (label, body) = (glyphs_in(&scene, &f, 16.0, 10.0, 496.0, 30.0), glyphs_in(&scene, &f, 16.0, 32.0, 496.0, 56.0));
        let (flashing, on) = objective_flash(complete, time);
        if flashing {
            seen_flash = true;
            assert_eq!(body, 0, "tick {tick} (time {time}): no body text in the flash window");
            assert_eq!(label > 0, on, "tick {tick} (time {time}): the label blinks");
        } else if !complete {
            seen_body = true;
            assert!(body > 0 && label > 0, "tick {tick} (time {time}): label + text after the window");
        }
    }
    assert!(seen_flash && seen_body, "05's opening objective passed through both phases");
}

#[test]
fn the_pause_keeps_the_health_ring_at_full_strength() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load");
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let ring = |scene: &nf_view::scene::Scene| -> f32 {
        let f = nf_view::hud::overlay(scene, 4.0 / 3.0);
        let ti = scene.hud_textures.iter().position(|(h, _)| *h == 0x0300_007a).unwrap() + 1;
        // The health pane's quads: the 0x0300007A sheet in the bottom-left quarter.
        f.runs.iter().filter(|r| r.0 == ti).flat_map(|r| f.v[r.2 as usize..(r.2 + r.3) as usize].iter())
            .filter(|v| v.p[0] < -0.4 && v.p[1] < -0.4).map(|v| v.c[3]).fold(0.0, f32::max)
    };
    // A faded-out pane (the pane-fade option decayed it).
    scene.sim.as_mut().unwrap().drones.pain_alpha = 0.0;
    assert_eq!(ring(&scene), 0.0, "sister: a faded pane draws at alpha 0");
    nf_view::pause_hud_refresh(scene.sim.as_mut().unwrap());
    assert_eq!(scene.sim.as_ref().unwrap().drones.pain_alpha, 1.0, "BL+0x8BC = 1.0 while paused [M 0x19b9c8:121]");
    assert!(ring(&scene) > 0.4, "the ring draws at full strength in the pause ({})", ring(&scene));
    // The page draws over the HUD: HUD vertices first.
    let hud = nf_view::hud::overlay(&scene, 4.0 / 3.0);
    let menu = nf_view::gpu::HudFrame { v: hud.v[..6].to_vec(), runs: vec![(0, 0, 0, 6)] };
    let both = nf_view::hud_under_menu(hud.clone(), &menu);
    assert_eq!(both.v.len(), hud.v.len() + 6);
    assert_eq!(both.runs.last().unwrap().2, hud.v.len() as u32, "the menu's run starts after the HUD's vertices");
}
