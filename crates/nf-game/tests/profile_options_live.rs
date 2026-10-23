//! profile-options-2: the codename options' save round-trips. Every option field round-trips through the game's own block format (the
//! bit widths of `LS_MakePlrSettings` 0x1cd418 / `LS_MakeMPSettings` 0x1ce508 / 
//! `LS_MakeGlobalSettings` 0x1cd9d0 / `LS_MakeCheats` 0x1ce260 [M]) with the CRC trailer intact.
//! No ISO needed: this pins the persisted model, not a level. The pages' evidence table is
//! `docs/systems/profile-options.md`; the runtime consumers are options-census's `drain_options`
//! (`crates/nf-view/src/lib.rs`) and its census table (`crates/nf-view/src/menu.rs`).

use nf_game::profile::Profile;

#[test]
fn round_trips_every_field() {
    let mut p = Profile::default_codename("ROUNDTRIP");
    // Flip every field off its default (and the volumes to non-default values). The option
    // fields carry the pages' own values: +0x03 = Manual Aim Toggle (row 0x1000_0126 [M
    // 0x213f08]), +0x0B = HUD auto-hide On (row 0x1000_023b's inverted items — byte 1 is "Off"
    // [M 0x213f08:66-71]), +0x0C = 2 (the Flashing Objects row's On value [M 0x213f08]),
    // MPSettings+0x2c = −75 (the Health Handicap row's lowest item [M 0x2144d8 — SIGNED]).
    p.plr.exists = true;
    p.plr.controller_style = 5;
    p.plr.ptp_option = 3;
    p.plr.auto_aim = false;
    p.plr.auto_aim_alt = true;
    p.plr.manual_aim_toggle = true;
    p.plr.auto_equip_best = false;
    p.plr.crouch_toggle = false;
    p.plr.rumble = false;
    p.plr.crosshair = false;
    p.plr.aux_style = 2;
    p.plr.hud_auto_hide = true;
    p.mpsg.radar = false;
    p.mpsg.health_handicap = -75;
    p.gset.music_volume = 33;
    p.gset.sfx_volume = 12;
    p.gset.subtitles = true;
    p.gset.widescreen = true;
    p.gset.sfx_mode = 4;
    p.chet.cheat_info = true;

    let mut q = Profile::default_codename("ROUNDTRIP");
    q.decode(&p.encode()).expect("the mutated profile decodes");
    assert_eq!(q.plr, p.plr, "PlayerOptions round-trip");
    assert_eq!(q.mpsg, p.mpsg, "MpOptions round-trip");
    assert_eq!(q.gset.music_volume, 33);
    assert_eq!(q.gset.sfx_volume, 12);
    assert!(q.gset.subtitles && q.gset.widescreen);
    assert_eq!(q.gset.sfx_mode, 4);
    assert!(q.chet.cheat_info);
}

/// The two fields whose model this re-land fixes, pinned against the pages that define them.
/// `health_handicap` is SIGNED — the page's −75 item must decode as −75, not the u32 wrap
/// 4294967221 [M P_CNMPOPTIONS_Handler 0x2144d8's row 0x1000_011c items]. `aux_style` is the
/// 2-bit +0x0C enum whose page On value is 2 and whose Input_Init default is 1 [M 0x213f08's
/// row 0x1000_0226; 0x17edd8:14] — a bool split would misread the page's On.
#[test]
fn the_signed_handicap_and_the_flash_enum_decode_as_the_pages_write_them() {
    let mut p = Profile::default_codename("SIGN");
    p.mpsg.health_handicap = -75;
    p.plr.aux_style = 2;
    p.plr.manual_aim_toggle = true;
    p.plr.hud_auto_hide = true;
    let mut q = Profile::default_codename("SIGN");
    q.decode(&p.encode()).unwrap();
    assert_eq!(q.mpsg.health_handicap, -75, "the −75 item stays −75 through the 32 raw bits [M 0x2144d8]");
    assert_eq!(q.plr.aux_style, 2, "the Flashing Objects On value 2 keeps its bit [M 0x213f08 + 0x17edd8:14]");
    assert!(q.plr.manual_aim_toggle, "Manual Aim Toggle round-trips [M 0x213f08 row 0x1000_0126]");
    assert!(q.plr.hud_auto_hide, "the HUD auto-hide byte round-trips [M 0x213f08 row 0x1000_023b]");
    // Sister (can fail): the extremes of both signed/2-bit fields.
    let mut r = Profile::default_codename("EXT");
    r.mpsg.health_handicap = -1;
    r.plr.aux_style = 3;
    let mut s = Profile::default_codename("EXT");
    s.decode(&r.encode()).unwrap();
    assert_eq!(s.mpsg.health_handicap, -1);
    assert_eq!(s.plr.aux_style, 3, "both bits of the +0x0C pair survive the save");
}
