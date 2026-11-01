## fe-pages-rebase (2026-10-31, s/fe-pages-rebase)

Branch from origin/master (81da6c47, batch-1031l). Task: land the credits/menu pages on top of
the stats page — merge origin/s/fix-stats-page, then origin/s/fe-pages-2, resolving the
conflicts in crates/nf-formats/src/hud.rs, crates/nf-view/src/lib.rs and
crates/nf-view/src/menu.rs so BOTH features survive (one FONTMedium load; both pages' draw
code); then merge origin/master again. Every page must still render as the tests pin
(fe_pages_draw.rs, fe_pages_draw2.rs, stats_page_elements.rs). Run every menu/front-end/stats
test binary and the full suite (tools/test/suite.py); quote every total.

## Progress (2026-10-31, ~19:40 UTC)

- Branch created from origin/master (81da6c47). Origin state checked: both source branches
  diverge from e4550f2e (already in master's history); s/fix-stats-page adds 2 commits
  (580f8697 stats/results pages, 4e123355 corner stamp clip), s/fe-pages-2 adds 8 commits
  (credits roll / 007 Bonus / win page, FONTMedium face 3, Screen Adjust, P_PARISENUM card,
  cheat-medal page, menudef SET ids). Merges start next.

## Progress (2026-10-31, ~21:05 UTC) — the merges and the runs

- s/fix-stats-page merged clean (auto). s/fe-pages-2 merge conflicted in exactly the three
  predicted files; resolved keeping both features: the `Option<&Font>` third-parameter signature
  on `menu_overlay_fonts` (stats' superset — `Some` passes the real FONTMedium, `None` falls back
  to SerpLight inside `results_page`), `menu_overlay` stays on `None`, lib.rs's three call sites
  pass `Some(&tables.font_medium)`, `draw_page_elems` calls take `[hfont, font,
  mfont.unwrap_or(font)]` (face order per the FACE_TEX table), and the `font_medium` doc joins
  both branches' citations. fe_pages_draw.rs's three call sites adapted to `Some(...)` —
  mechanical, no assertion touched. origin/master re-merge: no movement (81da6c47), already an
  ancestor.
- Named binaries on the final tree (all 0 failed): fe_pages_draw 7, fe_pages_draw2 7,
  stats_page_elements 4, menus_flow 43, menu_movies 4, menu_sfx_flow 19, menus_mouse_parity 17,
  mission_stats_flow 6, mission_stats_carry_live 2, frame_cost_live 6.
- Full suite chunked (the sandbox kills background jobs between tool calls): 237/999 binaries
  recorded, tally 804 passed, 0 failed, 97 ignored — no FAILED/TIMEOUT/CRASH. The disk filled
  once (0 free) causing two `nf_view` failures that re-run clean (96/0). The remaining 762
  binaries are the next message's resume; the report has the exact command.
- Pushed checkpoints after every step: 431f721a (branch+WORKLOG), 63709075 (stats merge),
  23cdee53 (fe-pages-2 merge), ade69a7f (the compile fixes).

## pause-hint-2 (2026-10-31) — the hint box against PS2 captures: size, fill, wrap

- **Sizing (failed reading corrected).** The pause-hint entry below says each box is sized for the previous box's
  text, so the first box of a pause is 75 tall. PS2 captures of 07000005 refute it: the first box, the reopened box
  and a second pause's first box are identical and fit their own text. Re-reading the decompile, msg 0x44 (jal at
  0x1f9770) ends with an unconditional `Page_Update(manager, +0x1bc, 0)` on the box page (jal 0x21edc8 at 0x21cf30,
  after the page loop) [M]. That update runs Memo_Update with the page visible, so the text msg 0x18 stored is
  counted before msg 0x65 (0x1f980c). The earlier pass read the 0x4c script arm and missed the post-loop call.
  `pause_hint_sized` / `pause_hint_last` are gone. `menu::hint_box_view` sizes the box for its own lines.
- **Fill.** `psiDrawSprites` 0x120e68 halves the sprite alpha byte (srl at 0x120fe0, SpriteColourA at 0x121004)
  [M], so 0x0000007f draws at 0x3f/128. The PS2 frame halves the backdrop (0.49-0.50) and so does the port now
  (0.50-0.51). `menu::sprite_colour`, used by the hint box only. `word_colour` elsewhere still reads the alpha
  byte unhalved; other menu sprites were not re-checked.
- **Wrap.** `menu::font_word_wrap` ports `Font_WordWrapString` 0x1ef178's measure:
  - spaces and tabs are free;
  - each glyph costs w + pre + post + kern + 1;
  - the width is checked before each character and not after the last.
  05's front-door hint now breaks after "front", "you" and "other", as on the PS2. `Font::layout`'s measure fitted
  "door," on line 1.
- **Kern finding (not changed).** `Font_GetKernAdjust`'s walk stops at the first pair below the character
  (disasm 0x1ee3ac), and the pair lists are ascending. So only a glyph's first pair can ever apply. The port's
  `Font::layout` applies any matching pair. The wrap port follows the game; the drawn layout is unchanged.
- tests/pause_hint_box.rs: 9 tests. The sizing test now pins own-text sizing, the overlay test the half-alpha fill.
  Two new tests cover the wrap: a synthetic one, and 05's hint on the disc.

## pause-hint (2026-10-31) — the objective-hint box drawn and driven

- The pause OBJECTIVES tab's ✕ box (option-box page 0x4000002f) is now drawn: SET 0xb's one fill
  (texel (32,19) of 0x03000042, 0x0000007f) over the button rect, the hint's memo lines and the OK row
  (Txt 0x266) in 0x645a4cff; the objectives list hides under it [D the 08000001 menu defs; M 0x1f9608,
  0x21e490, 0x21a598, 0x204408 0x205b98-0x205c34]. Before, only the input was modal and nothing showed.
- The pitch arithmetic is the EE's chopped `mul.s`: 18 · 0.01f · 100 = 17.999998, integer 17 — plain f32
  maths gives 18 and a box 7 units taller at 7 lines (`nf_formats::ps2float`).
- Read-order finding: msg 0x65 (the memo's line count) is read before the box's own Memo_Update, and the
  text's msg 0x18 cannot update the memo (first box: no focus; later boxes: the page is still hidden from
  the last close). So each box is sized for the previous box's text, the first of a pause for 0 lines.
  Implemented as the decompile reads; no PCSX2 capture yet (GAPS G-054 (4)).
- Input / sounds from Scroll_Update: ✕ 0x1da under the box page's gates (9), △ silent (MENU_SOUND 9; the
  manager's buzz is skipped because the push set +0x1d3), ~V 0x1d8 / 0x1d9 on the one-row wrap.
- Controls page 0x40000022: the Y-Axis label and value now always draw cream 0x645a49ff (their msg 0x1b
  and 0x1c are both cream; the handler sends no colour).
- tests/pause_hint_box.rs (7 tests, sisters included); render aid `--pause-hint-again`.

## page-titles (2026-10-31) — the shared title placement and the 007 Bonus corner stamp

- Every header control in menu-def 08000002 is (50,40,376×17) (0x40000012 y 39; 0x4000001d /
  0x40000020 540 wide) [D]. FixupResolution → (40,37,300×15); Label_Update pen y = (short)(y + h/2
  + extent/2 − 1) with FONTNFont2's extent 20 → 53 [M 0x21b610, 0x218cb8:178-205]. `page_header`
  drew at the fixed 54.5·448/480 = 50.9: every title about 2.5 units of 480 high. Now
  `menu::page_header_pen`; headless ink rows 43.3–56.7 of 480 against the PCSX2 stats/results
  captures' 43.1–56.8 (before: 40.7–54.0).
- The 007 Bonus page's 0x10000224 now draws through `corner_stamp` (unscaled, clipped at x 512,
  layer 3) instead of a generic 255-wide quad that ran past the backdrop's right edge.
- U-052 closed: the placement arms in MenuManager_Create 0x21b908:305-314 are if / else-if, so the
  stamp's forced-position arm never reaches FixupResolution.
- tests/page_titles.rs (3 tests, sisters included).
