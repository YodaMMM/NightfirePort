//! codename-entry headless tests: the name-entry page's grid model, the validator wiring, and the
//! create / delete / save flows through the port's memory card (numbers/ids only).
//!
//! Evidence: P_CNNAME_Handler 0x213790 (the 0x4c init, the 0x50 tick's box 4 save), C_KEYBOARD_Handler
//! 0x213a98 (the cell ids 1000/1001/1002, the 8-char clamp, del_when_keypressed, the accept), 
//! Menu_IsValidCodename 0x1f9930 (trim + strupr, 0x3bc empty, 0x2c0 reserved-substring against the
//! def_codename labels 0x29b/0x211 at 0x2e1b18/0x2e1b50 [L ee_snapshot]), Menu_UpdateMessageBox 0x1f7de8
//! (box modes: 0 LS_Load, 1 LS_Save, 3 psiDeleteData), C_SBCNSELECT 0x212f58 (the ~X delete, the list
//! rows), C_SBCNOPTIONS 0x213528 (the wheel's last row = Save Codename).

use nf_view::menu::{Action, FrontEnd, Page};

/// A scratch card dir per test.
fn card_dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("nf-cn-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Types a name through the direct-keyboard path after the game's first-press clear
/// [M C_KEYBOARD 0x213a98:18-27 clears on the first press; the port's cn_type_char is the append].
fn type_name(fe: &mut FrontEnd, s: &str) {
    fe.cn_typed.clear();
    fe.cn_clear_on_key = false;
    for ch in s.chars() {
        fe.cn_type_char(ch);
    }
}

/// The game's proposals [M P_CNNAME 0x213790:36/43-44 — DAT_0030d2d8 empty for the cheat entry,
/// DAT_0030d530 "BOND" [L ee_snapshot] for a profile name].
#[test]
fn open_seeds_the_game_s_proposals() {
    let mut fe = FrontEnd::new();
    fe.cn_open(0x4000_0025);
    assert_eq!(fe.page, Page::CnName);
    assert_eq!(fe.page.id(), 0x4000_0020, "the page id is the game's own [M 0x2029e0:372]");
    assert_eq!(fe.cn_typed_name(), "BOND", "the non-cheat proposal [M 0x213790:43-44]");
    assert_eq!(fe.cn_cursor_cell(), 1002, "the ✓ holds the focus on open [M 0x213790:47]");
    fe.cn_open(0x4000_001d);
    // Sister: the cheat entry's proposal is the EMPTY default — the two paths must differ.
    assert_ne!(fe.cn_typed_name(), "BOND");
    assert_eq!(fe.cn_typed_name(), "", "the cheat proposal [M 0x213790:36]");
}

/// The first press after open clears the proposal before typing [M del_when_keypressed
/// 0x213a98:18-27]; the SPACE press is swallowed entirely [M :21-25].
#[test]
fn first_press_clears_and_first_space_is_swallowed() {
    let mut fe = FrontEnd::new();
    fe.cn_open(0x4000_0025);
    assert!(fe.cn_press(0x41), "the letter cell types");
    assert_ne!(fe.cn_typed_name(), "BOND", "sister: the proposal must be gone");
    assert_eq!(fe.cn_typed_name(), "A", "typed on the cleared buffer [M 0x213a98:84-85]");
    fe.cn_open(0x4000_0025);
    assert!(fe.cn_press(FrontEnd::CN_KEY_SPACE));
    assert_eq!(fe.cn_typed_name(), "", "sister: the swallowed space typed nothing");
    assert!(fe.cn_press(FrontEnd::CN_KEY_SPACE));
    assert_eq!(fe.cn_typed_name(), " ", "the second space types [M DAT_0030d380 = 0x20]");
}

/// The 8-character clamp [M 0x213a98:75-77] and the ← key [M id 1000 = :39-42].
#[test]
fn max_eight_chars_and_the_delete_key() {
    let mut fe = FrontEnd::new();
    fe.cn_open(0x4000_0025);
    fe.cn_clear_on_key = false;
    fe.cn_typed.clear();
    for ch in "ABCDEFGH".chars() {
        assert!(fe.cn_type_char(ch));
    }
    assert_eq!(fe.cn_typed.chars().count(), FrontEnd::CN_MAX_CHARS);
    assert!(!fe.cn_type_char('I'), "sister: the ninth character is refused [M :75-77]");
    assert!(fe.cn_press(FrontEnd::CN_KEY_DELETE));
    assert_eq!(fe.cn_typed.chars().count(), 7, "← removes the last character [M :39-42]");
}

/// The grid carries the game's cell ids: digits 0x30-0x39, letters 0x41-0x5a, the four glyph
/// cells 100-103 [M P_CNNAME 0x213790:50-57] and the three special keys 1000-1002 [M 0x213a98].
#[test]
fn grid_shape_matches_the_game_s_cell_ids() {
    let flat: Vec<u32> = FrontEnd::CN_GRID.iter().flat_map(|r| r.iter().copied()).collect();
    assert_eq!(flat.len(), 10 + 26 + 4 + 3);
    assert_eq!(&flat[..10], &[0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39]);
    assert_eq!(&flat[10..36], &(0x41u32..=0x5a).collect::<Vec<_>>());
    assert_eq!(&flat[36..40], &[100, 101, 102, 103], "the glyph cells [M 0x213790:50-57]");
    // Sister: the three special keys must be present and ordered ← → ✓ at the tail.
    assert_eq!(&flat[40..], &[1000, 1001, 1002]);
}

/// The glyph cells type the game's four font-slot bytes [M P_CNNAME 0x213790:50-57 writes
/// 0xc4/0xc2/0xc6/0xc0 to 0x30d8d8/da/dc/de; C_KEYBOARD appends the cell text's first byte].
#[test]
fn the_glyph_cells_type_their_font_bytes() {
    let mut fe = FrontEnd::new();
    fe.cn_open(0x4000_0025);
    fe.cn_clear_on_key = false;
    fe.cn_typed.clear();
    for cell in [100u32, 101, 102, 103] {
        assert!(fe.cn_press(cell), "cell {cell} types");
    }
    assert_eq!(fe.cn_typed, "\u{c4}\u{c2}\u{c6}\u{c0}");
    for cell in [0x41u32, 0x42, 0x43, 0x44] {
        assert!(fe.cn_press(cell), "the letters fill to 8");
    }
    // Sister: the ninth character is refused — the 8-char clamp binds here too [M :75-77].
    assert!(!fe.cn_press(0x45));
}

/// The validator's empty refusal binds label 0x3bc onto box 6 [M C_KEYBOARD 0x213a98:47-51 ←
/// Menu_IsValidCodename 0x1f9930:81] and creates nothing.
#[test]
fn empty_name_refused_with_box_6_label_3bc() {
    let dir = card_dir("empty");
    let mut fe = FrontEnd::new();
    fe.set_profile_dir(dir.clone());
    fe.cn_open(0x4000_0025);
    fe.cn_typed.clear(); // an empty buffer — the validator's first refusal [M 0x1f9930:47-51]
    assert!(!fe.cn_press(FrontEnd::CN_KEY_CONFIRM), "the refused name is not accepted");
    assert_eq!(fe.cn_box, Some((6, 0x3bc)), "the empty label on the validator's box 6");
    assert!(fe.profiles.is_empty(), "sister: nothing was created");
    assert!(fe.cn_clear_on_key, "the next press clears again [M 0x213a98:50]");
    let _ = std::fs::remove_dir_all(&dir);
}

/// The validator's reserved-substring refusal binds label 0x2c0 [M Menu_IsValidCodename
/// 0x1f9930:63-76 — strstr against the def_codename labels, uppercased both sides].
#[test]
fn reserved_name_refused_with_box_6_label_2c0() {
    let dir = card_dir("reserved");
    let mut fe = FrontEnd::new();
    fe.set_profile_dir(dir.clone());
    fe.cn_reserved = vec!["Default".into(), "New Codename".into()];
    fe.cn_open(0x4000_0025);
    type_name(&mut fe, "ADEFAULT");
    assert!(!fe.cn_press(FrontEnd::CN_KEY_CONFIRM), "the reserved name is not accepted");
    assert_eq!(fe.cn_box, Some((6, 0x2c0)), "the reserved label [M 0x2c0]");
    assert!(fe.profiles.is_empty(), "sister: the reserved name created nothing");
    // A name without the reserved word passes the same validator.
    type_name(&mut fe, "MYGUARD");
    assert!(fe.cn_press(FrontEnd::CN_KEY_CONFIRM));
    assert_eq!(fe.cn_box, None, "the accepted name opens no error box");
    let _ = std::fs::remove_dir_all(&dir);
}

/// The accept: the normalized name creates the profile through the card flow ("Saving data…"
/// runs, then the completion routes through the wheel 0x4000001d — whose 0x4c, entered from the
/// name entry 0x40000020 with cheat clear, immediately forwards to Controls 0x40000022 [M
/// C_KEYBOARD 0x213a98:53-65; P_CNMENU 0x213288:38-40]).
#[test]
fn valid_name_creates_saves_and_forwards_to_controls() {
    let dir = card_dir("create");
    let mut fe = FrontEnd::new();
    fe.set_profile_dir(dir.clone());
    fe.cn_open(0x4000_0025);
    let path = nf_game::profile::Profile::path_in(&dir, "TESTING");
    assert!(!path.exists(), "sister: the file must not exist before the accept");
    type_name(&mut fe, "testing"); // lower-case on purpose — the game uppercases [M 0x1f9930:62]
    assert!(fe.cn_press(FrontEnd::CN_KEY_CONFIRM));
    // The write runs in the flow's tick [M HandlePS2CardState 0x110b98]; the message is the
    // game's "Saving data…" (msg 9, Txt 0x26c, no buttons) [M 0x10fa08 sub-8..].
    assert_eq!(fe.card_message(), Some((9, 0x26c)), "the saving message is up");
    assert_eq!(fe.card.buttons, 0, "progress messages carry no buttons [M 0x217bd8]");
    assert_eq!(fe.page, Page::CnName, "the wheel waits for the flow");
    fe.tick();
    assert!(path.exists(), "the chain is on the card [M LS_Save 0x1f7de8:21-29]");
    assert_eq!(fe.cn_typed_name(), "TESTING", "the buffer holds the normalized name [M 0x1f9930:77]");
    assert!(fe.profiles.iter().any(|p| p == "TESTING"), "the list re-enumerates");
    assert_eq!(fe.active_profile.as_deref(), Some("TESTING"));
    assert_eq!(fe.cn_exit, Some(0x4000_001d), "DAT_002e1ad8's continuation [M 0x213a98:58]");
    assert_eq!(fe.card.ops_done, 1, "the trigger wrote once");
    assert_eq!(fe.card_message(), None, "the box closed on completion [M 0x217bd8:62-71]");
    // The wheel's 0x4c forwards the create flow straight to Controls [M P_CNMENU 0x213288:38-40
    // — from 0x40000020 with cheat clear it sends Manager 0x44 → 0x40000022].
    assert_eq!(fe.page, Page::CnControls);
    assert_eq!(fe.page.id(), 0x4000_0022, "the forward's target [M 0x213288:39]");
    assert_eq!(fe.cursor, 0, "the Controls page opens on the style row");
    // The stored name round-trips through the game's own encode/decode.
    let back = nf_game::profile::Profile::load_file(&path).unwrap();
    assert_eq!(back.name, "TESTING");
    let _ = std::fs::remove_dir_all(&dir);
}

/// An existing codename opens the card flow's overwrite question (box 4, msg 4, Txt 0x27d,
/// Yes+Cancel) [M P_CNNAME 0x50:62-76 + ASK_OVERWRITE 0x10ee38]: YES overwrites the file,
/// Cancel keeps it.
#[test]
fn an_existing_codename_asks_the_overwrite_box() {
    let dir = card_dir("exists");
    let mut fe = FrontEnd::new();
    fe.set_profile_dir(dir.clone());
    fe.cn_open(0x4000_0025);
    type_name(&mut fe, "TWICE");
    assert!(fe.cn_press(FrontEnd::CN_KEY_CONFIRM));
    fe.tick();
    let original = nf_game::profile::Profile::load_file(&nf_game::profile::Profile::path_in(&dir, "TWICE")).unwrap();
    fe.unlocked[3] = true; // an edit the overwrite must capture
    assert!(fe.cn_press(FrontEnd::CN_KEY_CONFIRM), "the second accept arms the question");
    assert_eq!(fe.cn_box, Some((4, 0)), "the overwrite question [M P_CNNAME 0x50:62-76]");
    assert_eq!(fe.card_message(), Some((4, 0x27d)), "msg 4 binds Txt 0x27d [M 0x217bd8 case 4]");
    assert_eq!(fe.card.buttons, 0x6, "Yes+Cancel [M ASK_OVERWRITE 0x10ee38 PS2CardFlow = 6]");
    assert!(fe.card.awaiting, "the answer is expected [M DAT_002462d9]");
    // Cancel keeps the original file [M 0x10ee38's PSIOP_State = 4 abort].
    fe.cn_dismiss_box();
    assert_eq!(fe.cn_box, None, "sister: dismissing clears the box");
    assert!(fe.card.failed && fe.card.ops_done == 1, "the abort is the terminal failure [M Save_State 8]");
    assert_eq!(nf_game::profile::Profile::load_file(&nf_game::profile::Profile::path_in(&dir, "TWICE")).unwrap().progress.nightfire_status(),
        original.progress.nightfire_status(), "the abort left the file alone");
    // YES replaces it [M the Yes arm pops back into the write]. The overwritten file is the
    // fresh template again — the game maps Menu_MapDefaultCodename's template onto the slot
    // before the save [M 0x1f7fa8], so the create path never carries the front end's edits.
    assert!(fe.cn_press(FrontEnd::CN_KEY_CONFIRM));
    assert!(fe.card_answer(nf_view::menu::BTN_YES), "Yes continues the save [M 0x10ee38's == 2 arm]");
    fe.tick();
    assert_eq!(fe.card.ops_done, 2, "the overwrite wrote once");
    assert_eq!(fe.page, Page::CnControls, "the overwrite's completion forwards too [M 0x213288:38-40]");
    let replaced = nf_game::profile::Profile::load_file(&nf_game::profile::Profile::path_in(&dir, "TWICE")).unwrap();
    assert_eq!(replaced.progress.nightfire_status(),
        nf_game::profile::Profile::default_codename("TWICE").progress.nightfire_status(),
        "the overwrite wrote the fresh template [M Menu_MapDefaultCodename 0x1f7fa8]");
    let _ = std::fs::remove_dir_all(&dir);
}

/// The Edit list's ~X delete: the ASK_DELETE question (msg 0xd, Txt 0x3e5, Delete+Cancel),
/// YES removes under "Deleting data…", Back keeps it [M C_SBCNSELECT 0x212f58 case 0x5e →
/// box mode 3 → ASK_DELETE 0x10ec98 / DELETE_DATA 0x110320].
#[test]
fn the_edit_list_delete_removes_the_file() {
    let dir = card_dir("delete");
    let mut fe = FrontEnd::new();
    fe.set_profile_dir(dir.clone());
    fe.cn_open(0x4000_0025);
    type_name(&mut fe, "DROPPER");
    assert!(fe.cn_press(FrontEnd::CN_KEY_CONFIRM)); // created + active
    fe.tick();
    fe.page = Page::CnSelect;
    fe.reload_profiles();
    fe.cursor = 2; // the first saved row [M Menu_EnumSaves 0x1f7b78: the card's rows follow the defaults]
    let path = nf_game::profile::Profile::path_in(&dir, "DROPPER");
    assert!(path.exists(), "sister: the file must exist before the delete");
    assert!(fe.action(Action::Side), "the ~X arms the confirm");
    assert_eq!(fe.cn_delete_row, Some(2));
    assert_eq!(fe.card_message(), Some((0xd, 0x3e5)), "the delete question [M 0x217bd8 case 0xd]");
    assert_eq!(fe.card.buttons, 0x24, "Delete+Cancel [M ASK_DELETE 0x10ec98 PS2CardFlow = 0x24]");
    assert!(fe.action(Action::Back), "NO keeps the file [M the ASK_DELETE cancel]");
    assert!(path.exists());
    assert!(fe.action(Action::Side));
    assert!(fe.action(Action::Confirm), "YES runs the walk [M 0x10ec98's == 0x20 arm]");
    assert_eq!(fe.card_message(), Some((0xc, 0x3e6)), "\"Deleting data…\" while the walk runs [M 0x110320 sub-5]");
    fe.tick();
    assert!(!path.exists());
    assert!(fe.profiles.is_empty(), "the list re-enumerates [M P_CNSELECT 0x6e]");
    assert_eq!(fe.card.ops_done, 2, "create + delete each wrote once");
    let _ = std::fs::remove_dir_all(&dir);
}

/// The wheel's LAST row is Save Codename [M C_SBCNOPTIONS 0x213528 case 6 — a saved codename
/// runs box mode 1 = LS_Save(0x2e1ae8)]: the flow writes the loaded profile in place (with the
/// overwrite question — the file exists [M 0x10fa08 sub-7]), then the completion leaves to the
/// list (the queued page message [M 0x217bd8:62-64]).
#[test]
fn the_wheel_s_last_row_saves_the_codename() {
    let dir = card_dir("save");
    let mut fe = FrontEnd::new();
    fe.set_profile_dir(dir.clone());
    fe.cn_open(0x4000_0025);
    type_name(&mut fe, "SAVEME");
    assert!(fe.cn_press(FrontEnd::CN_KEY_CONFIRM)); // create rides the flow
    fe.tick();
    assert_eq!(fe.page, Page::CnControls, "the create's forward [M P_CNMENU 0x213288:38-40]");
    assert!(fe.action(Action::Back), "the port's Controls Back returns to the wheel");
    assert_eq!(fe.page, Page::CnMenu);
    let before = nf_game::profile::Profile::load_file(&nf_game::profile::Profile::path_in(&dir, "SAVEME")).unwrap();
    assert_eq!(before.progress.nightfire_status() & (1 << 5), 0, "sister: the bit must be unset first");
    fe.unlocked[5] = true; // the edit the save must capture
    fe.cursor = 6; // the wheel's last row [L CN_MENU[6] label 0x1f4 = "Save Codename" [D USATxt]]
    assert!(fe.action(Action::Confirm), "Save Codename consumes the press");
    assert_eq!(fe.card_message(), Some((4, 0x27d)), "the existing file asks the overwrite question");
    assert!(fe.card_answer(nf_view::menu::BTN_YES), "YES confirms [M 0x10ee38]");
    assert_eq!(fe.page, Page::CnMenu, "the wheel leaves only after the write");
    fe.tick();
    assert_eq!(fe.page, Page::CnSelect, "the queued page message fired [M 0x217bd8:62-64]");
    let after = nf_game::profile::Profile::load_file(&nf_game::profile::Profile::path_in(&dir, "SAVEME")).unwrap();
    assert_ne!(after.progress.nightfire_status() & (1 << 5), 0, "the save wrote the edit");
    // A default codename ("Default" is not a card file) re-maps and leaves without a save [M case 6's
    // slot < 2 arm].
    fe.page = Page::CnMenu; // the wheel again [M case 6 leaves to the list, the wheel reopens]
    fe.cn_name = "Default".into();
    fe.cursor = 6;
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::CnSelect);
    let _ = std::fs::remove_dir_all(&dir);
}

/// The cheat entry's ✓ runs the special-codename check and creates nothing [M C_KEYBOARD
/// 0x213a98:67-73 — Menu_SpecialCodenameCheck's miss returns without a save].
#[test]
fn the_cheat_entry_s_confirm_creates_nothing() {
    let dir = card_dir("cheat");
    let mut fe = FrontEnd::new();
    fe.set_profile_dir(dir.clone());
    fe.cn_open(0x4000_001d); // from the wheel — cheat_mode [M P_CNNAME 0x213790:33]
    type_name(&mut fe, "PASSPORT");
    assert!(fe.cn_press(FrontEnd::CN_KEY_CONFIRM));
    assert!(fe.cn_cheat_checked, "the special-codename check ran (not ported)");
    assert_eq!(fe.cn_box, None, "sister: no validator box stands on the cheat path");
    assert!(fe.profiles.is_empty(), "the cheat path never saves a profile");
    let _ = std::fs::remove_dir_all(&dir);
}

/// The grid focus moves by `Menu_FindControl` 0x1f16d8's direction scoring over the page data's
/// cell rects [M; D page 0x40000020] and stays put when nothing lies that way.
#[test]
fn the_grid_cursor_walks_and_clamps() {
    let mut fe = FrontEnd::new();
    fe.cn_open(0x4000_0025);
    assert_eq!(fe.cn_cursor_cell(), 1002);
    fe.cn_move(-1, 0);
    assert_eq!(fe.cn_cursor_cell(), 1001, "◀ from ✓ lands on the space key");
    fe.cn_move(0, -1);
    // ▲ from space (290,323,60×16): the x-overlapping cells above are W (280,281) and X (320,281);
    // the score 2|Δy| + 0.1|Δx| = 84 + 1 vs 84 + 3 picks W [M 0x1f16d8 dir 1].
    assert_eq!(fe.cn_cursor_cell(), 0x57, "▲ from space lands on W");
    fe.cn_move(0, 1);
    assert_eq!(fe.cn_cursor_cell(), 1001, "▼ back to the special row's space key");
    // ▼ from space wraps to the farthest x-overlapping cell above: '4' (280,161), −323 vs '5's −321
    // [M 0x1f16d8 dir 2, the back list].
    fe.cn_move(0, 1);
    assert_eq!(fe.cn_cursor_cell(), 0x34, "▼ from the bottom row wraps to the top");
    fe.cn_move(0, -1);
    assert_eq!(fe.cn_cursor_cell(), 1001, "▲ from the top row wraps back to space");
    fe.cn_move(1, 0);
    assert_eq!(fe.cn_cursor_cell(), 1002, "▶ lands on ✓");
    // ▶ from ✓ wraps to ← (the farthest y-overlapping cell to the left, |dy| − dx = −196) [M dir 8].
    fe.cn_move(1, 0);
    assert_eq!(fe.cn_cursor_cell(), 1000, "▶ from ✓ wraps to ←");
    fe.cn_move(-1, 0);
    assert_eq!(fe.cn_cursor_cell(), 1002, "sister: ◀ from ← wraps back to ✓");
    // Confirm presses the focused cell — without a card the accept refuses (NoCard) [M the
    // game's save goes through the card flow; the port's create_codename refuses without one].
    assert!(!fe.action(Action::Confirm));
    assert_ne!(fe.cn_typed_name(), "", "sister: the buffer is not empty — the proposal stood");
}

// --- The NF_ISO sister: the name-entry strings come from the language bank ---

/// Reads USATxt.dat out of FILES.BIN [M Txt_LoadLanguage 0x1caad0 — the same walk menus_flow uses].
fn iso_lang(iso_path: &str) -> Option<nf_formats::hud::Lang> {
    let iso = nf_formats::iso9660::IsoImage::open(iso_path).ok()?;
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).ok()?;
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("USATxt.dat"))?;
    let bytes = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).ok()?;
    nf_formats::hud::Lang::parse(&bytes).ok()
}

/// The name-entry page's label ids resolve against the disc [M every id below is a handler
/// argument: the titles P_CNNAME 0x213790:37/41, the validator's two refusals 0x1f9930:72/81,
/// the def_codename reserved labels 0x2e1b18/0x2e1b50's first dwords [L ee_snapshot], the wheel's
/// last-row label CN_MENU[6] [L table 0x2dfc28], the modified box P_CNMENU 0x6b, the cheat box
/// C_KEYBOARD 0x213a98:71, the Edit-%s header P_CNMENU 0x213288:32-33].
#[test]
fn the_name_entry_strings_come_from_the_language_bank() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set — skipping"); return };
    let Some(lang) = iso_lang(&iso) else { panic!("USATxt.dat must parse — the name entry binds it") };
    let b = |id: u32| lang.bind(id).unwrap_or("");
    for id in [0x1f5, 0x29b, 0x211, 0x2c0, 0x3bc, 0x1f4, 0x100_02d3, 0x100_02d4, 0x100_029d, 0x100_01e4, 0x100_02ff] {
        println!("label {id:#x} = {:?}", b(id));
    }
    assert!(!b(0x1f5).is_empty(), "the normal-mode title resolves [M 0x213790:41]");
    assert!(!b(0x100_02d4).is_empty(), "the cheat title resolves [M 0x213790:37]");
    assert!(!b(0x2c0).is_empty() && !b(0x3bc).is_empty(), "the validator's two refusals resolve");
    assert_eq!(b(0x29b), "New Codename", "def_codename[0]'s label [L 0x2e1b18 dword 0]");
    assert_eq!(b(0x211), "Default", "def_codename[1]'s label [L 0x2e1b50 dword 0]");
    assert_ne!(b(0x2c0), b(0x3bc), "sister: the two refusals are different lines");
}

// ── the create journey on key presses, from Main ─────────────────────────────────────────────

use nf_view::menu;

/// START → P_PARISENUM → P_ESTHERO (movie) → Main, the menus_flow boot chain [M 0x20a648:43,
/// 0x20a060:16-45].
fn tick_to_title(fe: &mut FrontEnd) {
    for _ in 0..8 {
        fe.tick();
        if fe.flow.get_state() == menu::ST_RUNNER && fe.flow.current_level == menu::FRONTEND_LEVEL { return; }
    }
    panic!("boot never reached the front-end map: state {} level {:x}", fe.flow.get_state(), fe.flow.current_level);
}

fn start_to_main(fe: &mut FrontEnd) {
    assert_eq!(fe.page, Page::Start);
    assert!(fe.action(Action::Confirm), "START");
    for _ in 0..8 { if fe.intro_movie { break; } fe.tick(); }
    assert!(fe.intro_movie, "P_ESTHERO's movie is up");
    for _ in 0..menu::ESTHERO_GRACE { fe.tick(); }
    assert!(fe.action(Action::Confirm), "the skip past the grace");
    fe.tick();
    assert_eq!(fe.page, Page::Main);
}

/// Main's rows change page after the 0x13-tick delay with input blocked [M C_GOCODENAMES
/// 0x216b38:29-37; P_CNSELECT 0x212ad0:73-74].
fn main_confirm(fe: &mut FrontEnd) {
    assert!(fe.action(Action::Confirm));
    for _ in 0..menu::DELAYED_PAGE_TICKS + menu::MAIN_OPEN_BLOCK { fe.tick(); }
}

/// The CloseIris walk lands `CLOSE_IRIS_PAGE_TICKS` ticks after the action [M 0x202190:6-9].
fn iris_confirm(fe: &mut FrontEnd) {
    assert!(fe.action(Action::Confirm));
    for _ in 0..menu::CLOSE_IRIS_PAGE_TICKS + 2 { fe.tick(); }
}

/// The task's journey, all on key presses from Main: Esc on Main holds, a codename is created,
/// the flow reaches Controls (the wheel's 0x4c forward [M P_CNMENU 0x213288:38-40]), and the
/// player carries on to the mission list. Evidence per step: C_GOCODENAMES 0x216b38:29-37 →
/// 0x4000001b; C_SBCNSELECT 0x212f58 case 0x4b sel 0 → 0x40000020; C_KEYBOARD 0x213a98:53-65 the
/// accept; P_CNMENU 0x213288:38-40 the forward → 0x40000022; C_RBCONTROL 0x217908:4b the commit
/// → Manager 0x5f; P_MAIN_Handler 0x20a7b0 has no back arm; C_SBNFDFCTY 0x216ca8:10-17 the list.
#[test]
fn from_main_create_codename_reach_controls_then_the_mission_list() {
    let dir = card_dir("journey");
    let mut fe = FrontEnd::new();
    fe.set_profile_dir(dir.clone());
    tick_to_title(&mut fe);
    start_to_main(&mut fe);
    assert_eq!(fe.page, Page::Main);
    // Esc on Main: the game has no back arm there [M P_MAIN_Handler 0x20a7b0] — the page holds
    // and the start movie does not replay.
    assert!(fe.action(Action::Back), "the press is consumed");
    assert_eq!(fe.page, Page::Main, "○/Esc on Main holds [M 0x20a7b0]");
    assert!(!fe.intro_playing && !fe.intro_movie, "no start-movie replay");
    // Codenames (row 2) → Select Codename 0x4000001b [M C_GOCODENAMES 0x216b38:29-37].
    fe.cursor = 2;
    main_confirm(&mut fe);
    assert_eq!(fe.page, Page::CnSelect);
    // New Codename (row 0) → the name entry 0x40000020 [M C_SBCNSELECT 0x212f58 case 0x4b sel 0].
    fe.cursor = 0;
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::CnName);
    // Type the codename on the grid (the ✓ holds the focus on open [M P_CNNAME 0x213790:47]) and
    // submit with ✕.
    fe.cn_typed.clear();
    fe.cn_clear_on_key = false;
    for ch in "AGENT07".chars() { assert!(fe.cn_type_char(ch)); }
    assert!(fe.action(Action::Confirm), "the ✓ submits");
    // "Saving data…" runs in the flow's tick; the completion enters the wheel 0x4000001d whose
    // 0x4c forwards straight to Controls [M P_CNMENU 0x213288:38-40 — from the name entry with
    // cheat clear it sends Manager 0x44 → 0x40000022].
    fe.tick();
    assert_eq!(fe.page, Page::CnControls);
    assert_eq!(fe.page.id(), 0x4000_0022, "the forward's target [M 0x213288:39]");
    assert_eq!(fe.cursor, 0, "the Controls page opens on the style row");
    // ✕ commits the style/invert pair and closes back to the wheel [M C_RBCONTROL 0x217908:4b].
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::CnMenu);
    // Carry on: the Back chain walks out (CnSelect → Main) and the mission list opens through
    // NightFire → Codenames → Play without saving → Difficulty.
    assert!(fe.action(Action::Back));
    assert_eq!(fe.page, Page::CnSelect);
    assert!(fe.action(Action::Back));
    assert_eq!(fe.page, Page::Main);
    fe.cursor = 0;
    main_confirm(&mut fe);
    assert_eq!(fe.page, Page::Codename);
    // Play without saving (row 1): the confirm box arms, ✕ continues through it [M P_NFSELECT
    // 0x20aec0:103 CloseIris → the difficulty wheel].
    fe.cursor = 1;
    assert!(fe.action(Action::Confirm));
    iris_confirm(&mut fe);
    assert_eq!(fe.page, Page::Difficulty);
    assert!(fe.action(Action::Confirm));
    iris_confirm(&mut fe);
    assert_eq!(fe.page, Page::MissionSelect, "the mission list [M C_SBNFDFCTY 0x216ca8:10-17]");
    let _ = std::fs::remove_dir_all(&dir);
}
