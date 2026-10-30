//! The Mission Stats page's flow — the results page's 0x5e opens 0x40000037, △ returns
//! (docs/systems/mission-stats.md). Data + state only (no pixels asserted).
//!
//! fe-audit: P_NFRESULTS 0x208e78:222-230 sends 0x5d → 0x4000002b (Dossier) and 0x5e → 0x40000037
//! (Stats); Button_Update sends 0x5d on action 0x20 and 0x5e on action 0x21 [M 0x1ef948:80-87,
//! :115-125] — the hint "~X Stats ~Y Dossier". The port's Side (Tab/E, the ~Y keycap) is 0x5d; the
//! port's key for 0x5e here is Extra (Q) [S unverified — the pad meaning of 0x20/0x21 is U-011].
//!
//! results-stats-key RESOLVES that open part [M psiInput_MapInputs 0x10cd80:377-424 — the default
//! arm runs for EVERY controller style: action 0x1f ← pad & 0x40, 0x20 ← & 0x80, 0x21 ← & 0x20,
//! 0x22 ← & 0x10, 0x1a-0x1d ← & 0x1000/0x4000/0x8000/0x2000] + [L docs/systems/input.md §2/§3, the
//! WORKLOG 0006 live trace: pad bits △ 0x10, ○ 0x20, ✕ 0x40, □ 0x80, Up 0x1000, Down 0x4000, Left
//! 0x8000, Right 0x2000]: **action 0x20 = □ (the Dossier's key), action 0x21 = ○ (the Stats key)**
//! — exactly the hint's own glyphs [D USATxt.dat Txt 0x2d0 = "~A Continue  ~X Stats  ~Y Dossier",
//! ~X = the ○ glyph, ~Y = □ (specialchar 0x2dd8d8)]. The port's Action::Side (Tab/E) carries 0x5d
//! and Action::Extra carries 0x5e; the keycaps name their keyboard keys (pc_key: ~Y → "Tab",
//! ~X → "Del") — Del is bound to Extra so the hint names a key that works.

use nf_view::menu::{key_action, menu_overlay, pad_menu_action, pc_key, Action, FrontEnd, KeyCap, Outcome, Page, PAGE_RESULTS, PAGE_STATS};

fn crate_texture(hash: u32) -> nf_view::scene::TextureImage {
    let (w, h) = if hash == 0x0300_0075 { (256, 128) } else if hash == 0x0300_0005 { (512, 128) } else { (8, 8) };
    nf_view::scene::TextureImage { key: (0, hash), width: w as u32, height: h as u32, rgba: vec![255; (w * h * 4) as usize], more: vec![], rate: 1 }
}

fn test_font() -> nf_formats::hud::Font {
    let glyph = |code: u16| nf_formats::hud::Glyph { u: 0, v: 0, w: 8, h: 12, yoff: 0, pre: 0, post: 8, code, kerns: vec![] };
    nf_formats::hud::Font { first: 33, last: 126, gap: 0.0, space: 3.0, glyphs: (33..=126).map(glyph).collect() }
}

/// Results (Complete) + 0x5e (Extra) → Mission Stats; `stats_enter` fills the rows; △ returns to
/// the results page. [M P_NFRESULTS 0x208e78:225-226 case 0x5e → Manager 0x44 0x40000037]
#[test]
fn the_0x5e_action_opens_stats_and_back_returns() {
    let font = test_font();
    let hfont = test_font();
    let textures: Vec<(u32, nf_view::scene::TextureImage)> =
        nf_view::menu::MENU_TEXTURES.iter().map(|&h| (h, crate_texture(h))).collect();
    let mut fe = FrontEnd::new();
    fe.outcome = Outcome::Complete;
    fe.page = Page::Results;
    fe.flow.page = PAGE_RESULTS;

    // 0x5e opens the stats page and the flow word follows [M 0x208e78:225-230].
    assert!(fe.action(Action::Extra), "0x5e opens Mission Stats");
    assert_eq!(fe.page, Page::Stats);
    assert_eq!(fe.flow.page, PAGE_STATS);
    assert_eq!(fe.page_id(), PAGE_STATS);

    // The page idles every other key — P_NFSTATS arms only the 0x4c build [M 0x209938:31-124].
    assert!(!fe.action(Action::Confirm));
    assert!(!fe.action(Action::Up));
    assert!(!fe.action(Action::Side));

    // △ returns to the opener [S — the menu-def script target; live check in the report].
    assert!(fe.action(Action::Back));
    assert_eq!(fe.page, Page::Results);
    assert_eq!(fe.flow.page, PAGE_RESULTS);

    // The page draws (the rows list may be empty before stats_enter — the draw must not panic).
    let frame = menu_overlay(&fe, &font, &hfont, None, &textures, 4.0 / 3.0);
    let _ = frame;
}

/// `stats_enter` against a live `Progress`: the rows land on the front end and the page draw
/// carries them (numbers/ids only — no pixel compare here).
#[test]
fn stats_enter_fills_the_rows_and_the_draw_takes_them() {
    let font = test_font();
    let hfont = test_font();
    let textures: Vec<(u32, nf_view::scene::TextureImage)> =
        nf_view::menu::MENU_TEXTURES.iter().map(|&h| (h, crate_texture(h))).collect();
    let mut fe = FrontEnd::new();
    fe.outcome = Outcome::Complete;
    fe.page = Page::Stats;
    fe.flow.page = PAGE_STATS;

    let mut progress = nf_game::progress::Progress::default();
    progress.reset_for_mission();
    for _ in 0..10 {
        progress.log_shot_fired(1);
        progress.log_shot_hit_enemy(1);
    }
    progress.log_health(1, 100.0);
    progress.log_update_elapsed(7900);
    assert!(fe.stats_enter(&mut progress, 0x0700_0005, 6, 1, None), "07000005 builds the rows");
    let rows = fe.mission_stats.as_ref().unwrap();
    assert_eq!(rows.len(), 14); // 9 data rows + the Opponents group + the spacer + the 3 tail rows
    let total = rows.iter().find(|r| r.id == 0x163).unwrap();
    assert_eq!(total.total, "295,887"); // the capture's TOTAL

    let frame = menu_overlay(&fe, &font, &hfont, None, &textures, 4.0 / 3.0);
    let _ = frame;
}

/// The results page's 0x5d (the port's Side, "~Y Dossier") goes to the Dossier [M 0x208e78:222-224
/// case 0x5d → 0x4000002b]; △ is not the Dossier (the manager's 0x5f pop — the old test had the
/// two swapped).
#[test]
fn the_results_side_opens_the_dossier() {
    let mut fe = FrontEnd::new();
    fe.outcome = Outcome::Complete;
    fe.page = Page::Results;
    fe.flow.page = PAGE_RESULTS;
    assert!(fe.action(Action::Side));
    assert_eq!(fe.page, Page::Dossier);
    assert_eq!(fe.cursor, 0);
    // Sister: △ does not open the Dossier [S unverified — the results page has no 0x5f arm; the
    // pop target of a load-created manager is not checked in PCSX2].
    let mut g = FrontEnd::new();
    g.outcome = Outcome::Complete;
    g.page = Page::Results;
    let _ = g.action(Action::Back);
    assert_ne!(g.page, Page::Dossier, "△ is not the 0x5d arm");
}

/// The results hint's keycaps name keys that work (results-stats-key). The hint is the game's own
/// text [D USATxt.dat Txt 0x2d0 "~A Continue  ~X Stats  ~Y Dossier"]; its ~X/~Y are the pad's ○/□
/// [M specialchar 0x2dd8d8 + 0x10cd80:411-413 (0x21 ← & 0x20 = ○, 0x20 ← & 0x80 = □); L
/// docs/systems/input.md §2]. The keycap source is `pc_key` (~X → "Del", ~Y → "Tab"), and the
/// port binds those exact keys: Del (and Q) produce the 0x5e arm's Action::Extra, Tab/E the 0x5d
/// arm's Action::Side [M the arms: P_NFRESULTS 0x208e78:222-230; Button_Update 0x1ef948:80-88].
#[test]
fn the_results_hint_keycaps_name_keys_that_work() {
    use winit::keyboard::KeyCode;
    // ~A Continue: ✕ = Confirm = Enter [M Button_Update 0x1ef948:74,105-107 — action 0x1f edge →
    // msg 0x4b; action 0x1f ← pad & 0x40 = ✕ (0x10cd80:410); L §3].
    assert_eq!(pc_key('A'), Some(KeyCap::Text("Enter")));
    assert_eq!(key_action(KeyCode::Enter), Some(Action::Confirm));
    // ~X Stats: ○ = action 0x21 → msg 0x5e → the Stats page [M 0x1ef948:84-88,116-121;
    // 0x208e78:225-230]. The keycap says Del; Del (and the older stand-in Q) must open it.
    assert_eq!(pc_key('X'), Some(KeyCap::Text("Del")));
    assert_eq!(key_action(KeyCode::Delete), Some(Action::Extra));
    assert_eq!(key_action(KeyCode::KeyQ), Some(Action::Extra));
    let mut fe = FrontEnd::new();
    fe.outcome = Outcome::Complete;
    fe.page = Page::Results;
    fe.flow.page = PAGE_RESULTS;
    assert!(fe.action(Action::Extra));
    assert_eq!(fe.page, Page::Stats);
    // ~Y Dossier: □ = action 0x20 → msg 0x5d → the Dossier [M 0x1ef948:80-83,123-127;
    // 0x208e78:222-224]. The keycap says Tab; Tab (and E) must open it.
    assert_eq!(pc_key('Y'), Some(KeyCap::Text("Tab")));
    assert_eq!(key_action(KeyCode::Tab), Some(Action::Side));
    assert_eq!(key_action(KeyCode::KeyE), Some(Action::Side));
    let mut fe = FrontEnd::new();
    fe.outcome = Outcome::Complete;
    fe.page = Page::Results;
    fe.flow.page = PAGE_RESULTS;
    assert!(fe.action(Action::Side));
    assert_eq!(fe.page, Page::Dossier);
}

/// The pad's menu vocabulary carries the same arms (results-stats-key): the game's menu actions
/// are style-independent [M psiInput_MapInputs 0x10cd80:377-424 — the default arm fills
/// 0x1a-0x22 for every style] and the widgets read their press edges [M Button_Update
/// 0x1ef948:74-91]. The port's `pad_menu_action` maps each edge onto the Action the page arms
/// read; a held button does not repeat (the edge masks are the game's own re-arm rule).
#[test]
fn the_pad_edges_carry_the_results_arms() {
    let mut prev = 0u16;
    // ○ (0x20) — the Stats key [L §2; M 0x1ef948:84-88] — opens the Stats page.
    assert_eq!(pad_menu_action(0x20, &mut prev), Some(Action::Extra));
    assert_eq!(prev, 0x20);
    // Held ○: no repeat (the game sends msg 0x5e on the 0x21 edge only [M 0x1ef948:84]).
    assert_eq!(pad_menu_action(0x20, &mut prev), None);
    // □ (0x80) while ○ is still held — the Dossier arm.
    assert_eq!(pad_menu_action(0x20 | 0x80, &mut prev), Some(Action::Side));
    // Everything released, then ✕ (0x40) — the Confirm/Continue arm.
    assert_eq!(pad_menu_action(0, &mut prev), None);
    assert_eq!(pad_menu_action(0x40, &mut prev), Some(Action::Confirm));
    // △ (0x10) — Back (the manager's pop [M MenuManager_Update 0x21c3d8:78-105]).
    assert_eq!(pad_menu_action(0x10, &mut prev), Some(Action::Back));
    // The d-pad: Up 0x1000 / Down 0x4000 / Left 0x8000 / Right 0x2000 [M 0x10cd80:410-413].
    assert_eq!(pad_menu_action(0x1000, &mut prev), Some(Action::Up));
    assert_eq!(pad_menu_action(0x4000, &mut prev), Some(Action::Down));
    assert_eq!(pad_menu_action(0x8000, &mut prev), Some(Action::Left));
    assert_eq!(pad_menu_action(0x2000, &mut prev), Some(Action::Right));
    // Start (0x800) alone maps to nothing — no menu handler reads action 0x1e's edge
    // [M the Menu_InputAction call sites: 0x1f/0x20/0x21/0x22/0x1a-0x1d only].
    assert_eq!(pad_menu_action(0x800, &mut prev), None);
    assert_eq!(pad_menu_action(0, &mut prev), None);
}

/// menu-stats-page: the page's labels resolve through the game's own string lookup. The header
/// quartet is [M P_NFSTATS 0x209938:54-61 — `Txt_BindLabel(0x1000204)` as the header row's cell
/// 0, then arms 0x19 with cells 1/2/3]: 0x1000204 = 0x01000204 (bank 1 — SEVEN hex digits; the
/// port's old 0x1000_0204 read 0x10000204 = bank 0x10, past USATxt.dat's 7 fixups [D the file's
/// NumFixups 7], so every header bind failed). The strings themselves: [L ee_snapshot
/// FixupTable 0x1f9b200 (fixups[1] = 0x3e8), Bank[1516] = "Category", Bank[1481] = "Current",
/// Bank[1482] = "Target", Bank[721] = "Mission Stats", Bank[1181] = "~B Back"] + [D USATxt.dat
/// bank 0: 0x163 = "TOTAL", 0x165 = "Bond Moves", 0x15a = "Opponents", 0x15b = "Dispatched",
/// 0x15c = "Subdued", 0x16e = "Surrendered", 0x15d = "Stealth Rating", 0x15e = "Accuracy
/// Rating", 0x15f = "Health Remaining", 0x160 = "Time", 0x161 = "007 Bonus", 0x162 =
/// "Subtotal", 0x164 = "Difficulty Bonus"]. The row labels bind at `stats_enter` — the same
/// lookup the draw's page furniture uses (menu-stats-page: menu_after feeds stats_enter the
/// sim's bank when the bare-mission app has no front-end lang).
#[test]
fn the_stats_page_resolves_its_labels_from_the_language_bank() {
    let Some(iso) = std::env::var("NF_ISO").ok().filter(|_| true) else { return };
    let Ok(iso) = nf_formats::iso9660::IsoImage::open(&iso) else { return };
    let Ok((entries, bin)) = nf_formats::disc::files_bin_entries(&iso) else { return };
    let Some(fe_entry) = entries.iter().find(|e| e.name.eq_ignore_ascii_case("USATxt.dat")) else { return };
    let Ok(bytes) = iso.read_bytes(bin.lba as u64 * 2048 + fe_entry.offset as u64, fe_entry.size as u64) else { return };
    let lang = nf_formats::hud::Lang::parse(&bytes).expect("USATxt.dat parses [M Txt_LoadLanguage 0x1caad0's FNames[SelLang]]");

    // The header quartet, through the port's own const (nf_game::stats::STATS_HEADER).
    let (h0, h1, h2, h3) = nf_game::stats::STATS_HEADER;
    assert_eq!(h0, 0x0100_0204, "the decompile's 0x1000204 is bank 1, not 0x10 [M 0x209938:54]");
    assert_eq!(lang.bind(h0), Some("Category"), "[L ee_snapshot Bank[1516]]");
    assert_eq!(lang.bind(h1), Some("Current"), "[L ee_snapshot Bank[1481]]");
    assert_eq!(lang.bind(h2), Some("Target"), "[L ee_snapshot Bank[1482]]");
    assert_eq!(lang.bind(h3), Some("TOTAL"), "[D USATxt.dat 0x163]");
    // The page's furniture labels [D 08000002 page 0x40000037's title/hint controls].
    assert_eq!(lang.bind(0x2d1), Some("Mission Stats"), "[L ee_snapshot Bank[721]]");
    assert_eq!(lang.bind(0x49d), Some("~B Back"), "[L ee_snapshot Bank[1181]]");

    // The rows bind at stats_enter (the same bank), so no category prints as its raw id.
    let mut fe = FrontEnd::new();
    let mut progress = nf_game::progress::Progress::default();
    progress.reset_for_mission();
    assert!(fe.stats_enter(&mut progress, 0x0700_0005, 6, 1, Some(&lang)), "07000005 builds the rows [M 0x209938:36-40]");
    let rows = fe.mission_stats.as_ref().unwrap();
    let cat = |id: u32| rows.iter().find(|r| r.id == id).map(|r| r.category.clone()).unwrap_or_default();
    assert_eq!(cat(0x165), "Bond Moves");
    assert_eq!(cat(0x15a), "Opponents");
    assert_eq!(cat(0x15b), "Dispatched");
    assert_eq!(cat(0x15c), "Subdued");
    assert_eq!(cat(0x16e), "Surrendered");
    assert_eq!(cat(0x15d), "Stealth Rating");
    assert_eq!(cat(0x15e), "Accuracy Rating");
    assert_eq!(cat(0x15f), "Health Remaining");
    assert_eq!(cat(0x160), "Time");
    assert_eq!(cat(0x161), "007 Bonus");
    assert_eq!(cat(0x162), "Subtotal");
    assert_eq!(cat(0x164), "Difficulty Bonus");
    assert_eq!(cat(0x163), "TOTAL");
    for r in rows.iter() {
        assert!(!r.category.starts_with('<'), "no raw id remains: row {:#x} = {:?}", r.id, r.category);
    }
    // The draw takes the bound rows without panic (the page furniture renders too).
    let font = test_font();
    let hfont = test_font();
    let textures: Vec<(u32, nf_view::scene::TextureImage)> =
        nf_view::menu::MENU_TEXTURES.iter().map(|&h| (h, crate_texture(h))).collect();
    let frame = menu_overlay(&fe, &font, &hfont, Some(&lang), &textures, 4.0 / 3.0);
    let _ = frame;
}
