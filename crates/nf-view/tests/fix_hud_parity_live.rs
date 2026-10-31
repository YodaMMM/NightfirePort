//! fix-hud-parity: the HUD divergences fixed against the decompile (docs/systems/hud.md), each pinned
//! with a sister that proves the check can fail:
//! - the crosshair's integer placement and its option gate (`HUD_UpdateCrossHair` 0x19e6d0);
//! - the damage glow's independent direction bits (`HUD_UpdateHealthPane` 0x19b9c8:92-107);
//! - the air / grip pane's enable, show and colour rules (`HUD_UpdateAirPane` 0x1a11c8 +
//!   `Player_MonitorAir` 0x191a70);
//! - the objective banner's flash window and body-only fade (`HUD_UpdateStatusPane` 0x19c310 case 3);
//! - the Q-Claw marker's pick and tint (`HUD_UpdateAmmoPane` 0x19a7f0:301-350);
//! - the AIMS-20 boot list and the Micro-Camera shutter (0x19cef8 / 0x1a12d8).
//! The pure-function pins run always; the frame pins are opt-in via `NF_ISO`.

use nf_view::hud;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn tex_quads(scene: &nf_view::scene::Scene, f: &nf_view::gpu::HudFrame, hash: u32) -> usize {
    let Some(ti) = scene.hud_textures.iter().position(|(h, _)| *h == hash) else { return 0 };
    f.runs.iter().filter(|r| r.0 == ti + 1).map(|r| r.3 as usize / 6).sum()
}

#[test]
fn crosshair_position_is_the_games_integer_placement() {
    // (short)(int)(256·cx + 256) − (w >> 1): the 11-px normal cell centred starts at 251 / 219.
    assert_eq!(hud::crosshair_pos(0.0, 0.0, 11, 11), (251.0, 219.0));
    // The 29-px aim cell: 256 − 14, 224 − 14.
    assert_eq!(hud::crosshair_pos(0.0, 0.0, 29, 29), (242.0, 210.0));
    // Offsets truncate toward zero: 256·0.501 + 256 = 384.256 → 384; −cy 0.25 → 224 − 56 = 168.
    assert_eq!(hud::crosshair_pos(0.501, 0.25, 11, 11), (379.0, 163.0));
    // Sister: the old float centring (256 − 5.5) is NOT what the game draws.
    assert_ne!(hud::crosshair_pos(0.0, 0.0, 11, 11).0, 256.0 - 5.5);
}

#[test]
fn damage_glow_bits_light_independently() {
    assert_eq!(hud::damage_glow_lit(0xF), [true, true, true, true], "0xF lights all four sprites (20..23)");
    assert_eq!(hud::damage_glow_lit(0xC), [false, false, true, true]);
    assert_eq!(hud::damage_glow_lit(0x5), [true, false, true, false]);
    // Sister: the old 0xC collapse dropped bits 1/2 whenever both 4 and 8 were set.
    assert_ne!(hud::damage_glow_lit(0xD), [false, false, true, true]);
}

#[test]
fn air_pane_follows_the_enable_and_show_rules() {
    use nf_game::player::{SS_SWIM, SS_WALK, SS_WIRE};
    // Disabled outside substates 3/6 — even with the head under water or the air refilling.
    assert_eq!(hud::air_pane(SS_WALK, true, 50.0, 0.0), None);
    assert_eq!(hud::air_pane(SS_WALK, false, 60.0, 0.0), None);
    // Swimming at the surface: enabled but BL+0x8C8 & 0x10 clear → hidden.
    assert_eq!(hud::air_pane(SS_SWIM, false, 50.0, 0.0), None);
    // Under water: the blue swim fill, (int)(50·2 + 28) = 128 wide, no icon.
    assert_eq!(hud::air_pane(SS_SWIM, true, 50.0, 0.0), Some((0x68b1d6, 128.0, false)));
    assert_eq!(hud::air_pane(SS_SWIM, true, 0.0, 0.0), Some((0x68b1d6, 28.0, false)));
    // The wire: forced shown, the green fill, (100 − 25)·2 + 28 = 178, the icon up.
    assert_eq!(hud::air_pane(SS_WIRE, false, 100.0, 25.0), Some((0x68d6b1, 178.0, true)));
    // The width floor of 1.
    assert_eq!(hud::air_pane(SS_WIRE, false, 100.0, 120.0).map(|p| p.1), Some(1.0));
    // Sister: the swim colour is not the wire's.
    assert_ne!(hud::air_pane(SS_SWIM, true, 50.0, 0.0).map(|p| p.0), Some(0x68d6b1));
}

#[test]
fn objective_banner_flashes_then_fades_only_the_body() {
    // NEW: flash while time + 60 > 300 — the label blinks on fmod(time, 40) < 20, no body, no shadow.
    assert_eq!(hud::objective_rows(false, 290.0), (0x785a_14ff, false, 0));
    assert_eq!(hud::objective_rows(false, 270.0), (0, false, 0));
    // time 240: 300 < 300 is false — steady: gold label with its shadow, body 0x7d6d59ff.
    assert_eq!(hud::objective_rows(false, 240.0), (0x785a_14ff, true, 0x7d6d_59ff));
    // The fade rewrites the body's alpha only (10·17 = 170); the label stays opaque.
    assert_eq!(hud::objective_rows(false, 10.0), (0x785a_14ff, true, 0x7d6d_59aa));
    // COMPLETE: flash while time + 60 > 180, then the label alone (body 0).
    assert_eq!(hud::objective_rows(true, 150.0).2, 0);
    assert_eq!(hud::objective_rows(true, 100.0), (0x785a_14ff, true, 0));
    // Sister: a NEW notice at 290 is not yet showing its body.
    assert_ne!(hud::objective_rows(false, 290.0).2, 0x7d6d_59ff);
}

#[test]
fn grapple_marker_picks_the_nearest_and_tints_in_range() {
    let pts = [(100.0, 50.0, 30.0), (200.0, 60.0, 12.0), (300.0, 70.0, 40.0)];
    // Nearest (12 units), in range 50 with context 2 → green 0x00ff00ff.
    assert_eq!(hud::grapple_marker(&pts, 50.0, 2), Some((200.0, 60.0, 0x00ff_00ff)));
    // The context byte not 2 → white.
    assert_eq!(hud::grapple_marker(&pts, 50.0, 0xff), Some((200.0, 60.0, 0xffff_ffff)));
    // Out of the claw's range → white.
    assert_eq!(hud::grapple_marker(&pts, 10.0, 2), Some((200.0, 60.0, 0xffff_ffff)));
    // Past the 10000-unit search start: nothing.
    assert_eq!(hud::grapple_marker(&[(1.0, 1.0, 20000.0)], 50.0, 2), None);
    // Sister: the farthest candidate is not the pick.
    assert_ne!(hud::grapple_marker(&pts, 50.0, 2).map(|m| m.0), Some(300.0));
}

#[test]
fn oicw_boot_list_rises_then_flashes_then_parks() {
    let mut b = nf_game::hud_pane::OicwBoot::new(false);
    b.tick(true);
    // Timer 149: 5 − 149/30 = 1 visible line, "COMMAND.COM" at (50, 408 − 17).
    let l = b.lines();
    assert_eq!((l[0].layer, l[0].x, l[0].y), (0x1c, 50, 391));
    assert_eq!(l[1].layer, 0xff);
    for _ in 0..200 { b.tick(true); }
    assert_eq!(b.mode(), 2, "after 150 + 30 ticks the boot list parks");
    assert!(b.lines().iter().all(|l| l.layer == 0xff));
    // Sister: zooming out resets nothing once parked (mode 2 holds until the pane re-creates).
    b.tick(false);
    assert_eq!(b.mode(), 2);
}

#[test]
fn frame_crosshair_option_gate_skips_the_aim_cross() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut sc = nf_view::load(&iso, "07000005.bin", false).expect("load");
    let cross = |sc: &nf_view::scene::Scene| tex_quads(sc, &hud::overlay(sc, 4.0 / 3.0), 0x0300_0026);
    {
        let s = sc.sim.as_mut().unwrap();
        s.drones.crosshair = false;
        s.aiming = false;
    }
    assert_eq!(cross(&sc), 0, "Crosshairs off hides the normal cross");
    // The held start weapon is not scoped: aim mode draws cell 1 regardless of the option.
    sc.sim.as_mut().unwrap().aiming = true;
    let w = sc.sim.as_ref().unwrap().gun.weapon as usize;
    let scoped = u32::from_le_bytes(sc.sim.as_ref().unwrap().weapons.records[w][0x68..0x6c].try_into().unwrap()) & 0x40 != 0;
    assert!(!scoped, "the start weapon is unscoped");
    assert_eq!(cross(&sc), 1, "the aim cross ignores the Crosshairs option [M 0x19e6d0:84-96]");
    // Sister: option on, not aiming → the normal cross draws.
    let s = sc.sim.as_mut().unwrap();
    s.drones.crosshair = true;
    s.aiming = false;
    assert_eq!(cross(&sc), 1);
}

#[test]
fn frame_damage_glow_and_air_pane() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut sc = nf_view::load(&iso, "07000005.bin", false).expect("load");
    {
        let s = sc.sim.as_mut().unwrap();
        s.drones.health.dir_bits = 0xF;
        s.drones.health.flash = 128.0;
    }
    let f = hud::overlay(&sc, 4.0 / 3.0);
    assert_eq!(tex_quads(&sc, &f, 0x0300_018d), 2, "bits 1 and 2 → sprites 20 and 21");
    assert_eq!(tex_quads(&sc, &f, 0x0300_018e), 2, "bits 4 and 8 → sprites 22 and 23");
    // Air: dry with the tank refilling → nothing (the old draw showed a dim pane here).
    {
        let s = sc.sim.as_mut().unwrap();
        s.drones.health.flash = 0.0;
        s.bond.air = 60.0;
        s.bond.underwater = false;
        s.bond.substate = nf_game::player::SS_WALK;
    }
    let f = hud::overlay(&sc, 4.0 / 3.0);
    assert_eq!(tex_quads(&sc, &f, 0x0300_017f), 0, "no air pane on dry land");
    assert_eq!(tex_quads(&sc, &f, 0x0300_017d), 0);
    // Swimming under water: the track + the fill, no icon.
    {
        let s = sc.sim.as_mut().unwrap();
        s.bond.substate = nf_game::player::SS_SWIM;
        s.bond.underwater = true;
    }
    let f = hud::overlay(&sc, 4.0 / 3.0);
    assert_eq!(tex_quads(&sc, &f, 0x0300_017f), 2, "track + fill under water");
    assert_eq!(tex_quads(&sc, &f, 0x0300_017d), 0, "the icon is the wire's only");
}

#[test]
fn frame_camera_shutter_and_oicw_boot_draw() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut sc = nf_view::load(&iso, "07000005.bin", false).expect("load");
    // The Micro-Camera view with the shutter slot up: one extra 0x03000002 full-screen quad.
    let solid = |sc: &nf_view::scene::Scene| tex_quads(sc, &hud::overlay(sc, 4.0 / 3.0), 0x0300_0002);
    {
        let s = sc.sim.as_mut().unwrap();
        s.gun.weapon = 0x54;
        s.zoom_target = 2.0;
        s.hud.camera_shutter = 0;
    }
    let base = solid(&sc);
    sc.sim.as_mut().unwrap().hud.camera_shutter = 4;
    assert_eq!(solid(&sc), base + 1, "the shutter sprite draws while the slot is non-zero");
    // The AIMS-20 view: the boot list's first line in font 2 (more font quads than without it).
    let font = |sc: &nf_view::scene::Scene| tex_quads(sc, &hud::overlay(sc, 4.0 / 3.0), 0x0300_0003);
    {
        let s = sc.sim.as_mut().unwrap();
        s.gun.weapon = 0x1a;
        s.hud.oicw_boot = Some(nf_game::hud_pane::OicwBoot::new(false));
    }
    let parked = { let mut b = nf_game::hud_pane::OicwBoot::new(false); for _ in 0..400 { b.tick(true); } b };
    sc.sim.as_mut().unwrap().hud.oicw_boot = Some(parked);
    let without = font(&sc);
    let mut b = nf_game::hud_pane::OicwBoot::new(false);
    b.tick(true);
    sc.sim.as_mut().unwrap().hud.oicw_boot = Some(b);
    let with = font(&sc);
    assert_eq!(with - without, "COMMAND.COM".len(), "one glyph quad per COMMAND.COM letter");
}

#[test]
fn frame_grapple_marker_shows_with_the_claw() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut sc = nf_view::load(&iso, "07000005.bin", false).expect("load");
    let have_tex = sc.hud_textures.iter().any(|(h, _)| *h == 0x0300_004c);
    let pts: Vec<[f32; 3]> = sc.sim.as_ref().unwrap().drones.grapple_points.iter().map(|g| g.pos).collect();
    eprintln!("[hud-parity] marker texture loaded: {have_tex}; grapple points {pts:?}");
    assert!(have_tex, "0x0300004C (AmmoSprInfo[7]) is in 07000005's archive");
    assert!(!pts.is_empty(), "07000005 places grapple points");
    let p = pts[0];
    // Stand 6 units south of the point, below it, looking at it.
    {
        let s = sc.sim.as_mut().unwrap();
        s.gun.weapon = 0x50;
        let at = [p[0], p[1] - 3.0, p[2] - 6.0];
        s.bond = nf_game::player::Bond::new(at, 0.0);
        let head = s.bond.head();
        let d = [p[0] - head[0], p[1] - head[1], p[2] - head[2]];
        s.bond.yaw = d[0].atan2(d[2]);
        let horiz = (d[0] * d[0] + d[2] * d[2]).sqrt();
        s.bond.pitch = d[1].atan2(horiz) / std::f32::consts::FRAC_PI_2;
    }
    let s = sc.sim.as_ref().unwrap();
    let on = hud::project_hud(s, p);
    let los = s.world.segment(p, s.bond.head()).is_none();
    eprintln!("[hud-parity] projected {on:?}, LOS {los}");
    let f = hud::overlay(&sc, 4.0 / 3.0);
    let n = tex_quads(&sc, &f, 0x0300_004c);
    if on.is_some() && los { assert_eq!(n, 1, "the marker draws over a visible grapple point"); }
    // Sister: another weapon held → no marker.
    sc.sim.as_mut().unwrap().gun.weapon = 2;
    let f = hud::overlay(&sc, 4.0 / 3.0);
    assert_eq!(tex_quads(&sc, &f, 0x0300_004c), 0);
}
