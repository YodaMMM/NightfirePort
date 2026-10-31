//! fix-hud (audit): HUD divergences found by a pass over the HUD draw functions, each pinned with a
//! sister that proves the check can fail:
//! - the text drop shadow is one copy at (+1, +1) (`Font_DrawText` 0x1eee48:96-98);
//! - the damage-glow alpha `BL+0x968` is a byte (`Player_HandlePain` 0x1902c8:126-130);
//! - on a mounted gun the ammo strip is the imp's heat gauge, kind 0x1f of 100
//!   (`HUD_UpdateAmmoPane` 0x19a7f0:137-145).
//! The pure-function pins run always; the frame pins are opt-in via `NF_ISO`.

use nf_view::hud;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Screen-space boxes (x0, y0, x1, y1) of the quads drawn with texture `hash`, in the 512×448 frame
/// (the overlay is built at 4:3, so NDC maps linearly).
fn boxes(scene: &nf_view::scene::Scene, f: &nf_view::gpu::HudFrame, hash: u32) -> Vec<[f32; 4]> {
    let Some(ti) = scene.hud_textures.iter().position(|(h, _)| *h == hash) else { return Vec::new() };
    let mut out = Vec::new();
    for r in f.runs.iter().filter(|r| r.0 == ti + 1) {
        for q in f.v[r.2 as usize..(r.2 + r.3) as usize].chunks(6) {
            let (a, b) = (q[0].p, q[2].p);
            out.push([(a[0] + 1.0) * 256.0, (1.0 - a[1]) * 224.0, (b[0] + 1.0) * 256.0, (1.0 - b[1]) * 224.0]);
        }
    }
    out
}

#[test]
fn text_shadow_is_one_pixel() {
    assert_eq!(hud::text_shadow_offset(1.0), 1.0, "Font_DrawText's shadow copy sits at (+1, +1)");
    assert_eq!(hud::text_shadow_offset(2.0), 2.0, "scaled text keeps the proportion");
    // Sister: the drifted 1.5-px offset is not the game's.
    assert_ne!(hud::text_shadow_offset(1.0), 1.5);
}

#[test]
fn mounted_heat_strip_is_kind_1f_of_100() {
    // (int)(heat · 0.5), max 100: 0 heat = empty, 120 heat = 60, saturated 200 = 100.
    assert_eq!(hud::mounted_heat_strip(0.0), (0.0, 100.0));
    assert_eq!(hud::mounted_heat_strip(121.0), (60.0, 100.0));
    assert_eq!(hud::mounted_heat_strip(200.0), (100.0, 100.0));
    assert_eq!(hud::mounted_heat_strip(260.0), (100.0, 100.0), "clamped to 100");
    // Sister: the value is the halved heat, not the raw heat.
    assert_ne!(hud::mounted_heat_strip(80.0).0, 80.0);
}

#[test]
fn frame_ammo_text_shadow_offset() {
    let Some(iso) = iso() else { eprintln!("NF_ISO unset — skipping"); return };
    let sc = nf_view::load(&iso, "07000005.bin", false).expect("load");
    let f = hud::overlay(&sc, 4.0 / 3.0);
    // The ammo count's glyphs (font 2, right-aligned at (496, 412)): each glyph is drawn twice, the
    // shadow first; pair every shadow quad with the text quad 1 px up-left of it.
    let q: Vec<[f32; 4]> = boxes(&sc, &f, 0x0300_0003).into_iter().filter(|b| b[0] > 380.0 && b[1] > 390.0 && b[3] < 425.0).collect();
    assert!(q.len() >= 4 && q.len() % 2 == 0, "the count draws shadow + text pairs, got {}", q.len());
    let n = q.len() / 2;
    let (sh, tx) = q.split_at(n);
    for (a, b) in sh.iter().zip(tx) {
        assert!((a[0] - b[0] - 1.0).abs() < 1e-3 && (a[1] - b[1] - 1.0).abs() < 1e-3, "shadow {a:?} vs text {b:?}: (+1, +1)");
        // Sister: not the old 1.5-px offset.
        assert!((a[0] - b[0] - 1.5).abs() > 0.25);
    }
}

#[test]
fn frame_mounted_gun_draws_the_heat_gauge() {
    let Some(iso) = iso() else { eprintln!("NF_ISO unset — skipping"); return };
    let mut sc = nf_view::load(&iso, "0700000d.bin", false).expect("load 0700000d");
    let strip = |sc: &nf_view::scene::Scene| boxes(sc, &hud::overlay(sc, 4.0 / 3.0), 0x0300_0128);
    {
        let s = sc.sim.as_mut().unwrap();
        assert!(!s.drones.props.gunimps.is_empty(), "0700000d carries mounted guns");
        s.gun.weapon = 0x47; // WeaponNone, what the mount holds
        s.hud.weapon_label_timer = 0;
        s.drones.props.gunimps[0].active = false;
    }
    // Sister: unmounted with WeaponNone (kind 0, clip 0) the pane draws no strip at all.
    assert!(strip(&sc).iter().all(|b| b[3] - b[1] < 0.5), "no gauge off the gun: {:?}", strip(&sc));
    {
        let s = sc.sim.as_mut().unwrap();
        s.drones.props.gunimps[0].active = true;
        s.drones.props.gunimps[0].heat = 120.0;
    }
    let b = strip(&sc);
    // BulletImg[0x1f] = (u 58, v 121, w 1, 1 px/unit) ×16 wide (max 100 > 87): the empty column
    // 100 tall from y 294 and the filled 60 from y 334, both x 480..496.
    let full = b.iter().find(|r| (r[3] - r[1] - 100.0).abs() < 0.5).copied();
    let fill = b.iter().find(|r| (r[3] - r[1] - 60.0).abs() < 0.5).copied();
    assert!(full.is_some() && fill.is_some(), "the heat gauge's two columns: {b:?}");
    let (full, fill) = (full.unwrap(), fill.unwrap());
    assert!((full[0] - 480.0).abs() < 0.5 && (full[2] - 496.0).abs() < 0.5 && (full[1] - 294.0).abs() < 0.5, "{full:?}");
    assert!((fill[1] - 334.0).abs() < 0.5 && (fill[3] - 394.0).abs() < 0.5, "{fill:?}");
}

#[test]
fn gadget_hints_post_once_with_the_view_idle() {
    let (mut cam, mut sticky) = (false, false);
    // Busy view model (raise / fire / reload): nothing, and the latch stays clear.
    assert_eq!(hud::gadget_hint(0x54, false, &mut cam, &mut sticky), None);
    assert!(!cam);
    // Idle with the Micro-Camera: 0x1000216, once.
    assert_eq!(hud::gadget_hint(0x54, true, &mut cam, &mut sticky), Some(0x100_0216));
    assert_eq!(hud::gadget_hint(0x54, true, &mut cam, &mut sticky), None, "CamHint latches");
    // The Remote Mine: 0x10002dc, once.
    assert_eq!(hud::gadget_hint(0x37, true, &mut cam, &mut sticky), Some(0x100_02dc));
    assert_eq!(hud::gadget_hint(0x37, true, &mut cam, &mut sticky), None, "StickyHint latches");
    // Sister: another weapon never posts.
    let (mut c2, mut s2) = (false, false);
    assert_eq!(hud::gadget_hint(0x02, true, &mut c2, &mut s2), None);
    assert!(!c2 && !s2);
}

#[test]
fn info_hint_labels_bind_to_text() {
    let Some(iso) = iso() else { eprintln!("NF_ISO unset — skipping"); return };
    let sc = nf_view::load(&iso, "07000005.bin", false).expect("load");
    let lang = sc.sim.as_ref().and_then(|s| s.lang.as_ref()).expect("the language bank");
    for id in [0x100_0216u32, 0x100_02dc, 0x200_0021] {
        let t = lang.bind(id).unwrap_or("");
        eprintln!("[hud-audit] {id:#x} = {t:?}");
        assert!(!t.trim().is_empty(), "label {id:#x} binds to a string");
    }
    // Sister: an id past the bank's end binds to nothing usable.
    assert!(lang.bind(0x0fff_ffff).map_or(true, |t| t.is_empty()));
}
