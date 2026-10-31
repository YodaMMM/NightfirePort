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
