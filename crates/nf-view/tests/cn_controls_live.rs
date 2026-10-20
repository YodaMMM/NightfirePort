//! cn-controls-page tests: the codename wheel's row 1 — P_CNCONTROLS 0x40000022.
//!
//! The page's data against the disc's own menu def [L 07000048.bin entry 08000002, page
//! 0x40000022, walked 2026-10-20 via nf_formats::menudef]; the flow against the decompile [M
//! P_CNCONTROLS_Handler 0x213cd0 + C_RBCONTROL_Handler 0x217908; the reach onto the pad config
//! [M Input_Update 0x17ee88:50-53 / Input_ChangeControllerStyle 0x17f5f0]]; the persistence
//! through the port's settings file. docs/systems/cn-controls-page.md carries the full chain.
//!
//! Every flow step moves by actions only (no fe.page/fe.cursor writes on the flow path).

use nf_formats::disc;
use nf_formats::levelbin;
use nf_formats::menudef::*;
use nf_view::menu::{default_opts, Action, FrontEnd, Page, ADV_ROWS, AV_ROWS, CN_CONTROLS_ROWS, CN_CONTROLS_STYLES, MP_ROWS};
use nf_game::input::PadConfig;

fn load_menu_def() -> MenuDef {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO not set");
    let iso = nf_formats::iso9660::IsoImage::open(&iso_path).expect("open ISO");
    let (files, bin) = disc::files_bin_entries(&iso).expect("FILES.BIN");
    let fe = files
        .iter()
        .find(|f| f.name.eq_ignore_ascii_case("07000048.BIN"))
        .expect("07000048.bin in FILES.BIN");
    let data = iso
        .read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + fe.offset as u64, fe.size as u64)
        .expect("read 07000048.bin");
    let lb = levelbin::LevelBin::parse(&data).expect("parse 07000048.bin");
    let e = lb.entries.iter().find(|e| e.hash() == 0x0800_0002).expect("kind-8 entry");
    MenuDef::parse(lb.data(&data, e), 0x8000_0002).expect("parse the menu def")
}

fn ctrl<'a>(p: &'a PageDef, id: u32, rect: (i16, i16, i16, i16)) -> &'a ControlDef {
    // Control ids repeat within a page (0x10000001 is three different controls here [L]); the
    // citation carries the full rect.
    p.controls.iter().find(|c| c.id == id && (c.x, c.y, c.w, c.h) == rect)
        .unwrap_or_else(|| panic!("control {id:#x} at {rect:?} not on page 0x40000022"))
}

/// The page's data is the disc's [L]: the wheel row 1's page exists with the cited controls —
/// the style list (0x10000025), the Y-Axis value (0x100000c9, disc label 518 = 0x206), the
/// Y-Axis static label (text 1170), the header 499 and the hint 1175.
/// Sister: the other codename pages' row tables are untouched (the same lens/first rows the
/// master tree pins) and the wheel table's row 1 still reads "Controls" 0x1f3.
#[test]
fn cncontrols_page_matches_the_disc_dump() {
    let def = load_menu_def();
    let p = def.page(0x4000_0022).expect("page 0x40000022 exists on the disc");
    assert_eq!(p.aux, 0x0735_0048, "the page's aux movie [L the PAGE record]");
    assert_eq!(p.kind, 3);
    // The style list control the open handler fills [M 0x213cd0:12-27 target 0x10000025].
    let list = ctrl(&p, 0x1000_0025, (60, 109, 316, 24));
    assert_eq!(list.f2, 5, "the list control's f2 [L the dump]");
    // The Y-Axis row: the value control and its static label [L ctrl[48]/ctrl[50]].
    let inv = ctrl(&p, 0x1000_00c9, (461, 109, 99, 25));
    assert_eq!(inv.label, Some(518), "the disc's placeholder label is 518 = 0x206 ('Inverted') [L]; \
        the runtime replaces it per PlayerSetting [M 0x213cd0:31-38]");
    assert_eq!(inv.f2, 7, "the value control's f2 [L the dump]");
    let yaxis = ctrl(&p, 0x1000_0001, (376, 113, 83, 16));
    assert_eq!(yaxis.label, Some(1170), "the static 'Y-Axis' label text [L ctrl[50]]");
    // Header + hint [L ctrl[39]/ctrl[43]].
    assert_eq!(ctrl(&p, 0x1000_0001, (50, 40, 376, 17)).label, Some(499), "the header 499 = 0x1f3");
    assert_eq!(ctrl(&p, 0x1000_0001, (50, 419, 540, 21)).label, Some(1175), "the hint 1175 = 0x497");
    // Sister: the port's row model carries exactly the disc-cited rows/labels — the style names
    // are the 8 Txt_BindLabel ids [M 0x213cd0:12-27].
    assert_eq!(CN_CONTROLS_STYLES, [0x82, 0x91, 0xa0, 0xaf, 0xbe, 0xcd, 0x1000_018b, 0x1000_019a]);
    assert_eq!(CN_CONTROLS_ROWS.len(), 2);
    assert_eq!(CN_CONTROLS_ROWS[0].ctrl, 0x1000_0025);
    assert_eq!(CN_CONTROLS_ROWS[1].ctrl, 0x1000_00c9);
    assert_eq!(CN_CONTROLS_ROWS[1].label, 0x492);
    // Sister: the other codename pages' tables and the wheel row are unchanged.
    assert_eq!(ADV_ROWS.len(), 8);
    assert_eq!(MP_ROWS.len(), 3);
    assert_eq!(AV_ROWS.len(), 10);
    assert_eq!(nf_view::menu::CN_MENU[1], (0x0300_0121, 0x1f3, 0x100_0051), "the wheel's row 1 label",);
    // The fresh-boot defaults [M Input_Init 0x17edd8:8-27 — style 0, inversion 0].
    let d = default_opts();
    assert_eq!(d.get(&0x1000_0025), Some(&0), "the style default [M 0x17edd8:22]");
    assert_eq!(d.get(&0x1000_00c9), Some(&0), "the inversion default [M the memset]");
}

/// The boot → wheel → Controls flow, actions only; the staged toggle, the commit and the
/// Back-discard [M 0x213cd0:30 (stage on open), 0x217908:24-33 (the 0x5e flip), :11-17 (the
/// 0x4b commit + close), the no-0x4d Back].
#[test]
fn cncontrols_flow_toggles_commits_and_discards() {
    let mut fe = FrontEnd::new();
    for _ in 0..8 { fe.tick(); if fe.flow.get_state() == nf_view::menu::ST_RUNNER && fe.flow.current_level == nf_view::menu::FRONTEND_LEVEL { break; } }
    fe.action(Action::Confirm);
    for _ in 0..8 { if fe.intro_movie { break; } fe.tick(); }
    for _ in 0..nf_view::menu::ESTHERO_GRACE { fe.tick(); }
    fe.action(Action::Confirm);
    fe.tick();
    assert_eq!(fe.page, Page::Main);
    fe.cursor = 2;
    fe.action(Action::Confirm);
    for _ in 0..nf_view::menu::DELAYED_PAGE_TICKS + nf_view::menu::MAIN_OPEN_BLOCK { fe.tick(); }
    assert_eq!(fe.page, Page::CnSelect);
    fe.action(Action::Confirm); // "Default" → the Edit wheel [M C_SBCNSELECT 0x212f58]
    assert_eq!(fe.page, Page::CnMenu);
    // Row 1 opens the Controls page [M C_SBCNOPTIONS 0x213528:38-39].
    fe.cursor = 1;
    assert!(fe.action(Action::Confirm));
    for _ in 0..30 { fe.tick(); }
    assert_eq!(fe.page, Page::CnControls);
    assert_eq!(fe.page.id(), 0x4000_0022);
    // The committed defaults on open [M Input_Init: style 0, inversion 0].
    assert_eq!(FrontEnd::cn_player_settings(&fe.opts), (0, false), "the fresh-boot pair [M 0x17edd8]");
    // Row 1 (the Y-Axis): the flip. The game's toggle is one unconditional flip [M 0x217908:24-33].
    assert!(fe.action(Action::Down));
    assert!(fe.action(Action::Right));
    assert_eq!(fe.opts.get(&0x1000_00c9), Some(&1), "the staged flip");
    // Back discards: the staged bit never reaches the committed pair [M — no 0x4d arm; the next
    // open re-stages from PlayerSetting].
    assert!(fe.action(Action::Back));
    for _ in 0..30 { fe.tick(); }
    assert_eq!(fe.page, Page::CnMenu);
    assert_eq!(fe.cursor, 1, "back on the wheel's Controls row");
    assert_eq!(FrontEnd::cn_player_settings(&fe.opts), (0, false), "the discard [M 0x213cd0:30]");
    // Re-enter, flip, Confirm commits [M 0x217908:11-17] and closes to the wheel.
    assert!(fe.action(Action::Confirm));
    for _ in 0..30 { fe.tick(); }
    assert_eq!(fe.page, Page::CnControls);
    assert!(fe.action(Action::Down));
    assert!(fe.action(Action::Right));
    assert_eq!(fe.opts.get(&0x1000_00c9), Some(&1), "the staged flip");
    // Back discards: the staged bit never reaches the committed pair [M — no 0x4d arm; the next
    // open re-stages from PlayerSetting].
    assert!(fe.action(Action::Back));
    for _ in 0..30 { fe.tick(); }
    assert_eq!(fe.page, Page::CnMenu);
    assert_eq!(fe.cursor, 1, "back on the wheel's Controls row");
    assert_eq!(FrontEnd::cn_player_settings(&fe.opts), (0, false), "the discard [M 0x213cd0:30]");
    // Re-enter, flip on row 1, then commit from row 0 (the game's 0x4b rides the list control
    // [M 0x217908:13-14]) and the page closes to the wheel.
    assert!(fe.action(Action::Confirm));
    for _ in 0..30 { fe.tick(); }
    assert_eq!(fe.page, Page::CnControls);
    assert!(fe.action(Action::Down));
    assert!(fe.action(Action::Right));
    assert!(fe.action(Action::Up));
    assert!(fe.action(Action::Confirm));
    for _ in 0..30 { fe.tick(); }
    assert_eq!(fe.page, Page::CnMenu);
    assert_eq!(FrontEnd::cn_player_settings(&fe.opts), (0, true), "the committed inversion [M 0x217908:15]");
    // The style spinner on row 0: Right wraps the 8 styles [M the value range < 8, 0x1f3558].
    assert!(fe.action(Action::Confirm)); // reopen
    for _ in 0..30 { fe.tick(); }
    assert_eq!(fe.page, Page::CnControls);
    for _ in 0..9 { assert!(fe.action(Action::Right)); }
    assert_eq!(fe.opts.get(&0x1000_0025), Some(&1), "9 rights wrap 0..7 → 1");
    assert!(fe.action(Action::Confirm));
    for _ in 0..30 { fe.tick(); }
    assert_eq!(fe.page, Page::CnMenu);
    assert_eq!(FrontEnd::cn_player_settings(&fe.opts), (1, true), "the committed style [M 0x217908:13-14]");
}

/// The committed pair reaches the in-game setting: the pad config the driver maps with
/// [M Input_Update 0x17ee88:50-53 negates the vertical look floats on the bit;
/// Input_ChangeControllerStyle 0x17f5f0 writes the style]. Sister: the untouched map leaves the
/// config at the boot defaults.
#[test]
fn cncontrols_commit_reaches_the_pad_config() {
    let mut cfg = PadConfig { curve: vec![0x7f; 129], style: 0, invert_look: false, player0: true };
    let mut opts = default_opts();
    FrontEnd::apply_cn_pad(&mut cfg, &opts);
    assert!(!cfg.invert_look, "the boot default [M 0x17edd8]");
    assert_eq!(cfg.style, 0);
    opts.insert(0x1000_00c9, 1);
    opts.insert(0x1000_0025, 5);
    FrontEnd::apply_cn_pad(&mut cfg, &opts);
    assert!(cfg.invert_look, "the committed inversion reaches the pad [M 0x17ee88:50-53]");
    assert_eq!(cfg.style, 5, "the committed style reaches the pad [M 0x17f5f0]");
    // Out-of-range values clamp (a corrupt file must not wedge the style jump [M 0x1f3558's < 8]).
    opts.insert(0x1000_0025, 99);
    FrontEnd::apply_cn_pad(&mut cfg, &opts);
    assert_eq!(cfg.style, 7);
}

/// The committed values persist across a save and load [M LS_MakePlrSettings 0x1cd418:37-48 /
/// LS_LoadPlrSettings 0x1cd5b8 — the game packs the pair into the codename profile; the port's
/// option persistence carries it in the file's [cn] section].
#[test]
fn cncontrols_persist_roundtrip() {
    let mut opts = default_opts();
    opts.insert(0x1000_00c9, 1);
    opts.insert(0x1000_0025, 3);
    let path = std::env::temp_dir().join(format!("nf-cn-controls-{}.toml", std::process::id()));
    nf_view::settings::save_opts(&path, &opts).expect("save");
    let text = std::fs::read_to_string(&path).expect("read back");
    assert!(text.contains("[cn]"), "the cn section exists");
    let (loaded, warn) = nf_view::settings::load_opts(&path);
    assert!(warn.is_none(), "the file parses clean: {warn:?}");
    let _ = std::fs::remove_file(&path);
    assert_eq!(FrontEnd::cn_player_settings(&loaded), (3, true), "the pair survives the round-trip");
    // Sister: the other sections' rows still collect (an [adv] key survives beside [cn]).
    let mut mixed = default_opts();
    mixed.insert(0x1000_011a, 0); // Advanced: Vibration Off
    mixed.insert(0x1000_00c9, 1);
    nf_view::settings::save_opts(&path, &mixed).expect("save mixed");
    let (loaded, _) = nf_view::settings::load_opts(&path);
    let _ = std::fs::remove_file(&path);
    assert_eq!(loaded.get(&0x1000_011a), Some(&0), "the [adv] row survives");
    assert_eq!(loaded.get(&0x1000_00c9), Some(&1), "the [cn] row survives");
}
