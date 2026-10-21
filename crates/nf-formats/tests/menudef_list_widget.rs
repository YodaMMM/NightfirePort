//! The M_LISTBOX port (`nf_formats::menudef::ListState`) against the game's own
//! List_* behaviour. Every assertion cites the decompile address that says it —
//! all [M], ACTION.ELF under `work/decomp/ACTION.ELF/`; the survey lives in
//! docs/systems/menu-list-widget.md. Pure unit tests: rows are built through the
//! port's `add_row`/`remove_row` (List_SendMessage 0x2196e8 arms 0xf/0x13).

use nf_formats::menudef::{ListState};

/// A 5-row list in the shape of the game's wrap test: rows a..e, b and d are
/// word-wrap continuations (LIST_ROW+0x2c8 == 1 [M List_SendMessage 0x2196e8:293]).
fn wrapped_list() -> ListState {
    let mut l = ListState::init(16.0, 92, 0, 16);
    for (name, wrap) in [("a", false), ("b", true), ("c", false), ("d", true), ("e", false)] {
        l.add_row(vec![name.to_string()], 0);
        let n = l.rows.len() - 1;
        l.rows[n].wrap = wrap;
    }
    l
}

/// `List_Init` 0x219488's metrics: stride = font height × 1.1 [M :58,75], visible rows =
/// h / stride floored [M :79], the fixed colours/width/alignment inits [M :82-94].
#[test]
fn init_metrics_are_the_creators() {
    let l = ListState::init(16.0, 92, 3, 16);
    assert!((l.stride - 17.6).abs() < 1e-4, "stride = font_h × 1.1 [M 0x219488:75]");
    assert_eq!(l.visible_rows, 5, "92 / 17.6 = 5.22 → 5 [M 0x219488:79]");
    assert_eq!(l.col_widths, [100, 0, 0, 0, 0], "widths[0] = 100 [M 0x219488:94]");
    assert_eq!(l.col_align[0], 1, "align[0] = 1 [M 0x219488:93]");
    assert_eq!(l.colour_normal, 0x4040_40ff, "normal colour [M 0x219488:86]");
    assert_eq!(l.colour_highlight, 0x7f7f_7fff, "highlight colour [M 0x219488:88]");
    assert_eq!(l.highlight_mode, 3, "def+0x04 [M 0x219488:80]");
    assert_eq!(l.cursor, None, "no cursor before rows [M 0x219488 List_Init]");
    // Sister: a taller font shrinks the visible window (the division is real).
    assert_eq!(ListState::init(32.0, 92, 0, 16).visible_rows, 2, "92 / 35.2 = 2.61 → 2");
}

/// `List_GetIndex` 0x21b420: the row's 0-based position, −1 when the walk runs off
/// [M :16-24].
#[test]
fn get_index_and_its_bounds() {
    let l = wrapped_list();
    assert_eq!(l.get_index(0), 0);
    assert_eq!(l.get_index(4), 4);
    // Sister: a row outside the list is "not found" — the game's walk returns −1.
    assert_eq!(l.get_index(5), -1, "off the end [M 0x21b420:19]");
}

/// `List_GetIndex_RealItem` 0x21b470: the counter bumps only when the row just advanced
/// TO is non-wrap [M :21-25] — wrap rows share the preceding real item's index, and the
/// first row's own flag is never consumed (the walk starts there).
#[test]
fn get_index_real_item_skips_wrap_rows() {
    let l = wrapped_list(); // a(0) b(wrap) c(1) d(wrap) e(2) as real items
    assert_eq!(l.get_index_real_item(0), 0, "a is real item 0");
    assert_eq!(l.get_index_real_item(1), 0, "wrap row b returns a's index [M 0x21b470:21-25]");
    assert_eq!(l.get_index_real_item(2), 1, "c is real item 1");
    assert_eq!(l.get_index_real_item(4), 2, "e is real item 2");
    // Sister: out of range walks off → −1.
    assert_eq!(l.get_index_real_item(9), -1, "off the end [M 0x21b470:18-19]");
}

/// `List_SetIndex` 0x21b4c8: index ≤ 0 = the FIRST row; off the end = the game's NULL
/// [M :16-24].
#[test]
fn set_index_clamps_low_and_returns_none_off_the_end() {
    let l = wrapped_list();
    assert_eq!(l.set_index(2), Some(2));
    assert_eq!(l.set_index(0), Some(0), "index 0 = first [M 0x21b4c8:16-24]");
    assert_eq!(l.set_index(-7), Some(0), "negative = first (0 < param_3 gates the walk)");
    // Sister: past the last row is NULL.
    assert_eq!(l.set_index(5), None, "off the end [M 0x21b4c8:19-21]");
}

/// `List_SetIndex_RealItem` 0x21b520: counter starts −1, non-wrap rows bump it, the first
/// row whose counter == index is returned [M :10-26].
#[test]
fn set_index_real_item_maps_through_wraps() {
    let l = wrapped_list();
    assert_eq!(l.set_index_real_item(0), Some(0), "real 0 = a");
    assert_eq!(l.set_index_real_item(1), Some(2), "real 1 = c (b is wrap) [M 0x21b520:17-22]");
    assert_eq!(l.set_index_real_item(2), Some(4), "real 2 = e");
    // Sister: exhausted list → NULL.
    assert_eq!(l.set_index_real_item(3), None, "no real item 3 [M 0x21b520:24-26]");
}

/// Arm 0xf seeds the cursor ONLY when it was NULL [M 0x2196e8:132-135]; arm 0x13 clears
/// it when the CURSOR row is removed, and leaves it alone otherwise [M :151-153].
#[test]
fn cursor_seeds_once_and_clears_only_on_the_cursor_row() {
    let mut l = ListState::init(16.0, 92, 0, 16);
    l.add_row(vec!["r0".into()], 1);
    assert_eq!(l.cursor, Some(0), "the first row becomes the cursor [M 0x2196e8:132-135]");
    l.add_row(vec!["r1".into()], 2);
    l.add_row(vec!["r2".into()], 3);
    assert_eq!(l.cursor, Some(0), "later adds do NOT move the cursor [M 0x2196e8:132-135]");
    l.remove_row(1);
    assert_eq!(l.cursor, Some(0), "removing a non-cursor row keeps it [M 0x2196e8:151-153]");
    l.remove_row(0);
    // Sister: removing the cursor row clears it (the game stores 0 = no selection).
    assert_eq!(l.cursor, None, "the cursor row's removal clears the selection [M :151-153]");
}

/// Arm 0x1d (scroll to real-item index, 0x2196e8:313-324) + `List_Update`'s top clamp
/// (0x21a598:216-233): the cursor lands on the mapped row and the window clamps so the
/// cursor ∈ [top, top + visible_rows).
#[test]
fn scroll_to_real_item_moves_the_window_under_the_cursor() {
    let mut l = ListState::init(16.0, 88, 0, 16); // 5 visible rows (88/17.6)
    for i in 0..9 { l.add_row(vec![format!("r{i}")], i as u32); }
    l.scroll_to_real_item(7);
    assert_eq!(l.cursor, Some(7), "real index 7 of an unwrap-free list [M 0x2196e8:317]");
    assert_eq!(l.cursor_visible, 7, "+0x56b = GetIndex(cursor) [M 0x2196e8:319-323]");
    assert_eq!(l.top, 3, "cursor below the window → top = cursor − visible + 1 [M 0x21a598:227-232]");
    // Sister: scrolling up clamps the window's TOP to the cursor [M 0x21a598:221-226].
    l.scroll_to_real_item(0);
    assert_eq!(l.top, 0, "cursor above the window → top = cursor index");
    // Sister: inside the window the top does not move [M 0x21a598:224-226].
    l.scroll_to_real_item(4);
    assert_eq!(l.top, 0, "cursor inside the window keeps the top");
    // And with no cursor at all the clamp is skipped entirely [M 0x21a598:218].
    l.cursor = None;
    l.scroll_cursor_into_view();
    assert_eq!(l.top, 0, "no cursor: the block never runs");
}

/// `List_Update` 0x21a598:174-179's metric refresh: h/stride and the top clamp.
#[test]
fn refresh_metrics_recomputes_rows_and_clamps_top() {
    let mut l = ListState::init(16.0, 92, 0, 16);
    for i in 0..3 { l.add_row(vec![format!("r{i}")], i as u32); }
    l.top = 9;
    l.refresh_metrics(92);
    assert_eq!(l.visible_rows, 5, "92 / 17.6 → 5 [M 0x21a598:174-179]");
    assert_eq!(l.top, 2, "the top clamps to the last row [M 0x21a598:179]");
    // Sister: a zero-height box zeroes the window (no division crash).
    l.refresh_metrics(0);
    assert_eq!(l.visible_rows, 0, "h 0 → 0 rows");
}
