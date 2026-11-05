//! sp-panes-new-drawing: the redeemer pane's sprite 10, the copter lock bracket
//! (`HUD_UpdateRedeemerPane` 0x19e178:137-183) — the audit-sp-panes §3 row the port drew as a
//! no-op. The pure pins cover the game's walk arithmetic (the clamp into [0,512]×[0,448], the
//! `scr.x − w` / `448 − scr.y − h` position, the (256,224)±32 `frame & 8` blink-off, the
//! last-wins order); the live pin rides 07000005's gate-0 copter #241 (it flies from the load
//! with no input, like copter_05_flight_live) through the real `overlay` with the missile cam
//! up — the game's own enable for pane 9 [M `Player_SetCamMode(0xd)` 0x195928:41].
//!
//! Template [L RedeemerSprInfo 0x2bda30 entry 10]: tex 0x03000059, (192,160) 128×128, flags
//! 0x500, layer 0x22 — the runtime layer 0x27 overrides it while drawn [M 0x19e178:170].
//! Numbers and hashes only. Opt-in via NF_ISO; run with NF_SILENT=1.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

use nf_view::hud::copter_lock_bracket;

#[test]
fn the_bracket_parks_with_no_copter() {
    assert_eq!(copter_lock_bracket(&[], 0, 128.0, 128.0), None, "no projecting copter — the sprite stays parked on layer 0xff [M 0x19e178:137]");
}

#[test]
fn the_bracket_positions_at_minus_w_and_flipped_y() {
    // A centre-dead copter, blink phase clear (frame 16: 16 & 8 == 0): x = 256 − 128, y = 448 − 224 − 128.
    assert_eq!(copter_lock_bracket(&[(256.0, 224.0)], 16, 128.0, 128.0), Some((128.0, 96.0)), "[M 0x19e178:167-169]");
    // Off-centre: the raw point is used unclamped when inside (x may go negative).
    assert_eq!(copter_lock_bracket(&[(100.0, 100.0)], 16, 128.0, 128.0), Some((-28.0, 220.0)), "[M :167-169]");
}

#[test]
fn the_bracket_clamps_into_the_frame() {
    // The clamp runs BEFORE the centre test and the position write [M :153-164 then :167].
    assert_eq!(copter_lock_bracket(&[(600.0, -20.0)], 16, 128.0, 128.0), Some((384.0, 320.0)), "clamped to (512,0) [M :153-164]");
    assert_eq!(copter_lock_bracket(&[(-5.0, 500.0)], 16, 128.0, 128.0), Some((-128.0, -128.0)), "clamped to (0,448) [M :153-164]");
}

#[test]
fn the_bracket_blinks_off_when_centred() {
    // |x−256|<32 && |y−224|<32 with GameState&8: the layer re-parks (the draw skips) [M :171-176].
    assert_eq!(copter_lock_bracket(&[(256.0, 224.0)], 8, 128.0, 128.0), None, "frame&8 — blink off [M :173-176]");
    assert_eq!(copter_lock_bracket(&[(270.0, 240.0)], 8, 128.0, 128.0), None, "inside the ±32 box [M :171-172]");
    // Strictly inside: 32.0 away is NOT centred (the game's `32.0 <= ABS(...)` x-branch) [M :171].
    assert_eq!(copter_lock_bracket(&[(288.0, 224.0)], 8, 128.0, 128.0), Some((160.0, 96.0)), "the box is open at exactly 32 [M :171]");
    // Off-centre copters never blink.
    assert_eq!(copter_lock_bracket(&[(100.0, 100.0)], 8, 128.0, 128.0), Some((-28.0, 220.0)));
}

#[test]
fn the_bracket_walk_is_last_wins() {
    // Every copter overwrites the sprite; the last node's verdict (draw or park) wins [M the do/while walk].
    assert_eq!(copter_lock_bracket(&[(100.0, 100.0), (256.0, 224.0)], 8, 128.0, 128.0), None, "the last copter parks it [M :173-176]");
    assert_eq!(copter_lock_bracket(&[(256.0, 224.0), (100.0, 100.0)], 8, 128.0, 128.0), Some((-28.0, 220.0)), "the last copter draws it [M :167-170]");
}

#[test]
fn the_table_holds_the_bracket_template() {
    // [L RedeemerSprInfo 0x2bda30 entry 10] — 11 sprites, sprite 10 = tex 0x03000059, (192,160)
    // 128×128, flags 0x500, layer 0x22, colour 0x7f7f7f78 (the runtime layer 0x27 rides on top).
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let img = nf_formats::iso9660::IsoImage::open(&iso).expect("iso");
    let elf = img.read_file(img.find("/ACTION.ELF").expect("ACTION.ELF")).expect("read");
    let t = nf_formats::hud::HudTables::from_action_elf(&elf).expect("tables");
    assert_eq!(t.redeemer.len(), 11);
    let s10 = &t.redeemer[10];
    assert_eq!((s10.texture, s10.x, s10.y, s10.w, s10.h), (0x0300_0059, 192, 160, 128, 128), "[L 0x2bda30 + 10·0x2c]");
    assert_eq!((s10.flags, s10.layer, s10.colour), (0x0500, 0x22, 0x7f7f_7f78), "[L]");
}

#[test]
fn the_bracket_draws_over_the_flying_copter_during_the_ride() {
    // The bracket texture rides the level HUD archives; probe the copter levels for one that
    // carries tex 0x03000059 (the [L] template above pins the table side level-independently).
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut hit: Option<(&str, nf_view::scene::Scene, usize)> = None;
    for level in ["07000008.bin", "07000005.bin", "0700001b.bin", "07000003.bin", "0700000b.bin"] {
        let Ok(mut scene) = nf_view::load(&iso, level, false) else { continue };
        let has_tex = scene.hud_textures.iter().any(|(h, _)| *h == 0x0300_0059);
        let ci = scene.sim.as_ref().unwrap().drones.copters.iter().position(|c| c.gate == 0);
        if has_tex && ci.is_some() { hit = Some((level, scene, ci.unwrap())); break; }
        eprintln!("[bracket] {level}: tex {has_tex} copter {}", ci.is_some());
    }
    let Some((level, mut scene, ci)) = hit else {
        eprintln!("[bracket] no probed level carries tex 0x03000059 with a copter — the live ride pin needs the render check");
        return;
    };
    let frame = {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..120 { sim.step_bond(Default::default(), false, false, false); }
        // The body capture (Copter_Register fills rec+0x38 from the assembly script's entity [M
        // 0x17e530:7-8]; the test stands in for it) and the ride: the camera sits 10 units +z of
        // the copter looking −z — the copter projects dead centre (256, 224).
        sim.drones.copters[ci].body_obj = Some(1);
        let p = sim.drones.copters[ci].pos();
        sim.missile_cam = Some(nf_view::scene::MissileCam { pos: [p[0], p[1], p[2] + 10.0], dir: [0.0, 0.0, -1.0], restore: 0, riding: true });
        sim.drones.frame
    };
    let f = nf_view::hud::overlay(&scene, 4.0 / 3.0);
    let ti = scene.hud_textures.iter().position(|(h, _)| *h == 0x0300_0059).expect("the bracket texture loads") + 1;
    let drawn = f.runs.iter().any(|(t, _, _, _)| *t == ti);
    if frame & 8 == 0 {
        assert!(drawn, "{level}: the bracket draws on the centred copter at frame {frame} (frame&8 clear) [M :170]");
    } else {
        assert!(!drawn, "{level}: the centred bracket blinks off at frame {frame} (frame&8 set) [M :173-176]");
    }
    // Sister: with the cam down the pane is disabled (the game enables it only via
    // Player_SetCamMode(0xd) [M 0x195928:41]) — nothing draws at all.
    scene.sim.as_mut().unwrap().missile_cam = None;
    let f2 = nf_view::hud::overlay(&scene, 4.0 / 3.0);
    assert!(!f2.runs.iter().any(|(t, _, _, _)| *t == ti), "no ride — no redeemer pane, no bracket [M 0x195928:41]");
}
