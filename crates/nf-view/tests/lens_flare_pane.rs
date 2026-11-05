//! The lens-flare pane's pure state and geometry (`HUD_UpdateLensFlarePane` 0x19d770, the port's
//! `nf_view::hud::{flare_alpha_step, flare_on_screen, flare_quads, health_pip_alpha}`), plus the
//! health pips' fade-scaled blink.

use nf_view::hud::{flare_alpha_step, flare_on_screen, flare_quads, health_pip_alpha};
use nf_formats::hud::SpriteInfo;

fn sp(tex: u32) -> SpriteInfo {
    SpriteInfo { colour: 0xffff_ff00, colour2: 0xffff_ffff, layer: -1, flags: 0x0900, x: 0, y: 0, w: 0, h: 0,
        uv: [0, 0, 31, 31], label: -1, texture: tex }
}

fn table() -> Vec<SpriteInfo> {
    (0..7).map(|k| sp(0x0300_004f + k)).collect()
}

/// The sticky alpha: +0x20 while the sun is visible and on-screen, −0x10 otherwise,
/// clamped 0..255 [M 0x19d770:73-111].
#[test]
fn flare_alpha_ramp_and_clamps() {
    let mut a = 0.0f32;
    for _ in 0..8 { a = flare_alpha_step(a, true); } // 0x20·8 = 256 → the 0xff clamp
    assert_eq!(a, 255.0);
    a = flare_alpha_step(250.0, true);
    assert_eq!(a, 255.0);
    a = flare_alpha_step(10.0, false);
    assert_eq!(a, 0.0);
    a = flare_alpha_step(255.0, false);
    assert_eq!(a, 239.0); // one decay tick off the ceiling
}

/// The gate is the strict (−5, 517)×(−5, 453) box on the RESCALED position [M :103-111].
#[test]
fn flare_on_screen_gate_bounds() {
    assert!(!flare_on_screen(-5.0, 224.0));
    assert!(flare_on_screen(-4.9, 224.0));
    assert!(!flare_on_screen(517.0, 224.0));
    assert!(flare_on_screen(516.9, 224.0));
    assert!(!flare_on_screen(256.0, -5.0));
    assert!(flare_on_screen(256.0, -4.9));
    assert!(!flare_on_screen(256.0, 453.0));
    assert!(flare_on_screen(256.0, 452.9));
}

/// The 7 quads: size = texture dims × [3, 4, 1, 5, 2, 0.5], position = the sun mirrored about
/// the screen centre by [0.7, 0.5, 0.3, −0.1, −0.2, −0.4], sprite 6 on the sun with the
/// distance-shrunk centre glow [M :135-183]. 64×64 textures.
#[test]
fn flare_quads_geometry_and_truncations() {
    let t = table();
    // Sun at (356, 124): dx = 100, dy = −100 from the 512×448 centre.
    let quads = flare_quads(&t, 255, 356.0, 124.0, |_| (64.0, 64.0));
    assert_eq!(quads.len(), 7);
    let size = [3.0, 4.0, 1.0, 5.0, 2.0, 0.5];
    let pos = [0.7, 0.5, 0.3, -0.1, -0.2, -0.4];
    for k in 0..6 {
        let (sp, x, y, w, h) = quads[k];
        assert_eq!(w, 64.0 * size[k]);
        assert_eq!(h, 64.0 * size[k]);
        assert_eq!(x, ((356.0 - 256.0) * pos[k] + 256.0) as i32 as f32);
        assert_eq!(y, ((124.0 - 224.0) * pos[k] + 224.0) as i32 as f32);
        assert_eq!(sp.texture, 0x0300_004f + k as u32);
    }
    // Sprite 6: dist = sqrt(100² + 100²) ≈ 141.42 → (int)(256 − 141.42) = 114 → ×1/64 = 1.78125
    // → w = (int)(64·1.78125) = 114; on the sun exactly.
    let (_, x, y, w, _) = quads[6];
    assert_eq!((x, y), (356.0, 124.0));
    assert_eq!(w, 114.0);
    // The sun dead-centre: k = 256·0.015625 = 4.0 → w = 256 (the 64-dim texture).
    let quads = flare_quads(&t, 255, 256.0, 224.0, |_| (64.0, 64.0));
    assert_eq!(quads[6].3, 256.0);
    // Off screen far: the centre glow clamps to nothing.
    let quads = flare_quads(&t, 255, 900.0, 224.0, |_| (64.0, 64.0));
    assert_eq!(quads[6].3, 0.0);
    // No alpha or a short table: nothing draws.
    assert!(flare_quads(&t, 0, 256.0, 224.0, |_| (64.0, 64.0)).is_empty());
    assert!(flare_quads(&t[..3], 255, 256.0, 224.0, |_| (64.0, 64.0)).is_empty());
}

/// The health pips' alpha: filled 0.55 / empty 0.15, × the pane fade; pip 0's low-health blink
/// REPLACES the factor before the fade multiply [M 0x19b9c8:185-200] — the sister asserts the
/// pre-fix shape (no fade in the blink arm) fails.
#[test]
fn health_pip_alpha_blink_scales_with_fade() {
    // Steady pips: the fade always multiplies.
    assert_eq!(health_pip_alpha(2, 4.0, 0, 1.0), 0.55);
    assert_eq!(health_pip_alpha(2, 4.0, 0, 0.5), 0.275);
    assert_eq!(health_pip_alpha(5, 4.0, 0, 0.5), 0.075);
    // Pip 0 blinking at health < 7.14: the blink factor, still × fade.
    assert_eq!(health_pip_alpha(0, 0.3, 0, 1.0), 0.55); // frame 0: 0 % 30 < 15 → 0.55
    assert_eq!(health_pip_alpha(0, 0.3, 0, 0.5), 0.275);
    assert_eq!(health_pip_alpha(0, 0.3, 15, 0.5), 0.075); // 15 % 30 = 15 → the 0.15 arm
    // Above the blink threshold pip 0 is a normal filled pip.
    assert_eq!(health_pip_alpha(0, 4.0, 15, 0.5), 0.275);
}

/// A sprite with no loaded texture draws at size 0, so the whole flare vanished with its ramp green.
#[test]
fn every_flare_sprite_has_its_texture_on_the_desert_level() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let scene = nf_view::load(&iso, "0700000c.bin", false).unwrap();
    let flare = &scene.hud.as_ref().unwrap().flare;
    assert_eq!(flare.len(), 7);
    for sp in flare {
        assert!(scene.hud_textures.iter().any(|(h, _)| *h == sp.texture), "{:#x}", sp.texture);
    }
}
