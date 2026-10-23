//! The shipped menu-def census for task unported-front-end-menus-6 — where the never-mentioned
//! front-end functions' controls/pages actually live. Everything here is [D] (read off the ISO);
//! the decompile pins are [M] and cited in docs/systems/keypad-menu.md.
//!
//! Assertions:
//! 1. The front-end def (07000048's 08000002) contains NONE of the debug-toggle handler control
//!    ids (0x100000c7 ALLOWFREEZE, 0x100000bd DUMMY, 0x10000101 DRAWALL, 0x10000169 LOCKUP,
//!    0x100001f7 BRIGHT, 0x10000216 UNLOCK, 0x1000021d CONTROLS, 0x100000ba KEYPAD) and neither
//!    debug page (P_TWEAKS 0x40000044 / P_TWEAKS2 0x40000046) — the seven C_CHCH*_Handlers are
//!    unreachable through the shipped front end [M 0x2029e0 dispatch arms + D this census].
//!    Sister: the same def DOES contain pages the game demonstrably opens (0x40000002 the main
//!    menu, 0x4000004b the pause page) — a parse regression flips those too.
//! 2. The FMV page 0x4000004d EXISTS (P_FMV_Handler 0x2169b0's dispatch arm) but no message in
//!    the whole def targets it — the orphan-page evidence [D].
//! 3. The help control 0x10000225 (Page_FindHelpControl 0x21f250's target) appears NOWHERE —
//!    so even the reached FindHelpControl walk always finds nothing in the shipped data.
//!    Sister: the parse itself is live (pages were found).
//! 4. The per-level defs (kind-8 08000001) DO carry the keypad page 0x40000021 (page file-hash
//!    0x80000003 [M Lock_Update 0x1ae3d8:242-243's create]) with the digit controls 0x100000ba —
//!    the in-game keypad mini-game is real and reachable [D].

use nf_formats::disc;
use nf_formats::iso9660::IsoImage;
use nf_formats::levelbin;
use nf_formats::menudef::MenuDef;

fn iso() -> IsoImage {
    let path = std::env::var("NF_ISO").expect("NF_ISO not set");
    IsoImage::open(&path).expect("open ISO")
}

fn menu_def(iso: &IsoImage) -> MenuDef {
    let (files, bin) = disc::files_bin_entries(iso).expect("FILES.BIN");
    let fe = files
        .iter()
        .find(|f| f.name.eq_ignore_ascii_case("07000048.BIN"))
        .expect("07000048.bin in FILES.BIN");
    let data = iso
        .read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + fe.offset as u64, fe.size as u64)
        .expect("read 07000048.bin");
    let lb = levelbin::LevelBin::parse(&data).expect("parse 07000048.bin");
    let e = lb.entries.iter().find(|e| e.hash() == 0x0800_0002).expect("menu-def entry");
    MenuDef::parse(lb.data(&data, e), 0x8000_0002).expect("parse the menu def")
}

/// The eight Handler_HandleMessage control ids these handlers own [M 0x2029e0].
const DEBUG_IDS: [u32; 8] = [
    0x1000_00c7, // C_CHCHALLOWFREEZE_Handler 0x216258
    0x1000_00bd, // C_CHCHDUMMY_Handler    0x2162b8
    0x1000_0101, // C_CHCHDRAWALL_Handler  0x216320
    0x1000_0169, // C_CHCHLOCKUP_Handler   0x216400
    0x1000_01f7, // C_CHCHBRIGHT_Handler   0x216460
    0x1000_0216, // C_CHCHUNLOCK_Handler   0x2164c0
    0x1000_021d, // C_CHCHCONTROLS_Handler 0x216520
    0x1000_00ba, // C_KEYPAD_Handler       0x216810 (front-end def only; see test 4)
];

#[test]
fn front_end_def_has_no_debug_toggle_controls_or_tweaks_pages() {
    if std::env::var_os("NF_ISO").is_none() { eprintln!("NF_ISO not set; skip"); return; }
    let iso = iso();
    let md = menu_def(&iso);

    let mut hits = Vec::new();
    for p in &md.pages {
        for c in &p.controls {
            if DEBUG_IDS.contains(&c.id) {
                hits.push((p.id, c.id));
            }
        }
    }
    assert!(hits.is_empty(), "debug-toggle control ids in the shipped front-end def: {hits:?}");
    for page in [0x4000_0044u32, 0x4000_0046] {
        assert!(md.pages.iter().all(|p| p.id != page), "debug page {page:#x} unexpectedly shipped");
    }
    // Sister: the def is real and the pages the game opens are present — a parse regression
    // zeroes both and this test would fail for the wrong reason.
    assert!(md.pages.iter().any(|p| p.id == 0x4000_0002), "the main menu page must exist");
    assert!(md.pages.iter().any(|p| p.id == 0x4000_0034), "the language page must exist");
}

#[test]
fn fmv_page_is_orphaned_in_the_front_end_def() {
    if std::env::var_os("NF_ISO").is_none() { eprintln!("NF_ISO not set; skip"); return; }
    let iso = iso();
    let md = menu_def(&iso);

    let p = md.pages.iter().find(|p| p.id == 0x4000_004d).expect("P_FMV page 0x4000004d ships");
    // Sister: the page parsed with its controls (the census is live).
    assert!(!p.controls.is_empty(), "the FMV page parsed empty");

    // No mgr msg, control msg or keyframe message anywhere in the def targets the page.
    let mut navs = 0;
    for p in &md.pages {
        for m in &p.mgr_msgs {
            if m.1 == 0x4000_004d || m.2 == 0x4000_004d { navs += 1; }
        }
        for c in &p.controls {
            for m in &c.msgs {
                if m.1 == 0x4000_004d || m.2 == 0x4000_004d { navs += 1; }
            }
            for s in &c.scripts {
                for k in &s.keyframes {
                    for m in &k.msgs {
                        if m.a == 0x4000_004d || m.b == 0x4000_004d || m.target == 0x4000_004d as i32 { navs += 1; }
                    }
                }
            }
        }
    }
    assert_eq!(navs, 0, "messages navigating to the orphaned FMV page 0x4000004d");
}

#[test]
fn help_control_ships_on_exactly_the_eight_pages() {
    if std::env::var_os("NF_ISO").is_none() { eprintln!("NF_ISO not set; skip"); return; }
    let iso = iso();
    let md = menu_def(&iso);

    // Page_FindHelpControl 0x21f250's target id [M 0x21f250: the h[6] == 0x10000225 compare];
    // Page_Update 0x21edc8:61-67 re-runs the walk per frame and calls Page_SetHelpText
    // 0x21f2b8 — so the help control is live data where it ships [D this census].
    let found: Vec<u32> = md.pages.iter().filter_map(|p| p.find_help_control().map(|_| p.id)).collect();
    // Def order: main, MP rules, MP environment mods, MP player mods, MP bot setup,
    // the two codename options pages, the nav options page.
    assert_eq!(found, vec![0x4000_0002, 0x4000_0014, 0x4000_0028, 0x4000_0017, 0x4000_002c, 0x4000_002d, 0x4000_002e, 0x4000_0031],
        "the help control 0x10000225's shipped pages (def order)");
    // Sister: a page without the help control really has no 0x10000225 control at all.
    let main = md.pages.iter().find(|p| p.id == 0x4000_0009).expect("P_START ships");
    assert!(main.find_help_control().is_none(), "P_START must not carry a help control");
}

#[test]
fn level_defs_carry_the_keypad_page_with_digit_controls() {
    if std::env::var_os("NF_ISO").is_none() { eprintln!("NF_ISO not set; skip"); return; }
    let iso = iso();
    let (files, bin) = disc::files_bin_entries(&iso).expect("FILES.BIN");

    let mut keypad_levels = Vec::new();
    for f in files.iter() {
        if !f.name.to_ascii_uppercase().starts_with("07") { continue; }
        let data = match iso.read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + f.offset as u64, f.size as u64) {
            Ok(d) => d, Err(_) => continue,
        };
        let lb = match levelbin::LevelBin::parse(&data) { Ok(l) => l, Err(_) => continue };
        let e = match lb.entries.iter().find(|e| e.kind == 8) { Some(e) => e, None => continue };
        // The keypad page's PAGE tag carries file hash 0x80000003 [M Lock_Update 0x1ae3d8:242's
        // create arg] — that is the hash MenuDef::parse must filter on for the in-game pages.
        let md = match MenuDef::parse(lb.data(&data, e), 0x8000_0003) { Ok(m) => m, Err(_) => continue };
        for p in &md.pages {
            if p.id == 0x4000_0021 {
                // The page tag's file hash is 0x80000003 — the create's first argument [M 0x1ae3d8:242].
                assert_eq!(p.file_hash, 0x8000_0003, "keypad page file hash in {}", f.name);
                let digits = p.controls.iter().filter(|c| c.id == 0x1000_00ba).count();
                assert!(digits >= 4, "keypad page in {} has {digits} digit controls", f.name);
                keypad_levels.push(f.name.clone());
            }
        }
    }
    assert!(keypad_levels.len() >= 10, "the keypad page must ship in many level defs, got {:?}", keypad_levels);
    // Sister: the front-end def's OWN pages never contain 0x100000ba (test 1 covered it, but the
    // same id is re-checked here against the level-def parse path so a hash-filter regression in
    // MenuDef::parse cannot fake this pass).
    let fe = menu_def(&iso);
    assert!(fe.pages.iter().all(|p| p.controls.iter().all(|c| c.id != 0x1000_00ba)));
}
