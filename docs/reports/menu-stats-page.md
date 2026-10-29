# menu-stats-page — the Mission Stats page resolves its labels and draws from its own data

**Status: the three reported defects (raw-id labels, missing headers, overlapping columns) are
root-caused and fixed; the page furniture (title, hint, stamp, logo, stripes) is drawn from the
page's own [D] control data. One more defect found and fixed on the way: the pause SCORE tab's
header ids carried the same transcription slip.**

## Root causes

1. **The row labels printed as `<0x165>` … because `stats_enter` got no language bank.**
   `menu_after` (`crates/nf-view/src/lib.rs`) passed only the front-end `self.lang`, which a
   bare-mission boot never fills (the front end loads `USATxt.dat` only in `run()`'s front-end
   branch). The page's own furniture bound fine off the sim's bank — the rows around it didn't.
   Fixed with the draw sites' existing pattern: `self.lang.clone().or_else(‖ sim.lang.clone())`.

2. **The headers never bound: `STATS_HEADER` transcribed the decompile's seven-digit ids with
   the wrong digit run.** `P_NFSTATS_Handler` 0x209938:54-61 binds `0x1000204` = **0x01000204
   (bank 1)**; the port's const said `0x1000_0204` = 0x10000204 (**bank 0x10**) — past
   `USATxt.dat`'s 7 fixups [D the file's NumFixups 7], so every header bind failed. The real
   strings: 0x01000204 → "Category", 0x010001e1 → "Current", 0x010001e2 → "Target" [L
   ee_snapshot FixupTable 0x1f9b200 (fixups[1] = 0x3e8), Bank[1516]/[1481]/[1482]], 0x163 →
   "TOTAL" [D USATxt.dat bank 0]. Fixed in `crates/nf-game/src/stats.rs` and in the pause SCORE
   tab's draw (`menu.rs`, the same slip — its hardcoded fallbacks had masked it [M C_GCPAUSE
   0x204408:455-459 binds the same bank-1 ids]).

## The layout (the page's own data + the list widget's draw code)

- **[D] 08000002 page 0x40000037's six controls** (the port's menu-def parser, scratch dump):
  the title (50,40,376×17) LBL 0x2d1 "Mission Stats" [L Bank[721]], the hint (50,419,540×21)
  LBL 0x49d "~B Back" [L Bank[1181]], the stamp 0x10000224 TEX 0x0300014c at (340,198,255×255),
  the logo TEX 0x0300016d at (434,38,156×43), and the list 0x10000022 at (60,78,520×324).
- **[M] The 0x4c build's list configuration** (`List_SendMessage` 0x2196e8, read
  instruction-side): arm 0x17 resets the list (widths[0] = 100% left); arm 0x28 sets
  widths[0] = 0x23 = **35%**; arms 0xe append 21%/right, 21%/right, 24%/right — four columns;
  arm 0x10 ADDS the header row with cell 0 ("Category") and arms 0x19 fill cells 1..3
  ("Current"/"Target"/"TOTAL") — the header is ROW 0 of the list; `Menu_AddRow` 0x201c10 packs
  each data row as `"label|cur|tgt|total"` (the timer rows via `Timer_Seconds2String`; arm 0x73
  arms the even-row stripes).
- **[M] `List_Update` 0x21a598's column math**, now in the port's draw: x = list.x + comp6 + 2 +
  Σ widths[j]·(w−16)·0.01, right-aligned cells at + widths[k]·w·0.01 − ink → Category left at
  x 68, Current right edge 353.6, Target 459.4, TOTAL 580.9 (640 space). No more `100%50,000`
  overlap: every value column is right-aligned in its own box.
- **The stripes**: [M] arm 0x73 sets +0x548 = 0xf050580 and 0x21a598 draws an even-row plate —
  the port draws a translucent plate on even rows [S the exact look; the game uses the
  0x03000042 atlas sprite].
- **[S] The row pitch stays 16.7** (the pause tab's measured value); the game's own stride is
  `Font_GetTextExtent(def size)·1.1`, not measurable headlessly.

## Verification

- New test `the_stats_page_resolves_its_labels_from_the_language_bank`
  (`crates/nf-view/tests/mission_stats_flow.rs`): pins the bank-1 header ids, the [L] header and
  furniture strings, and every row category through the real `stats_enter` + `menu_overlay` path
  (no raw `<…>` id remains). 6/6 in the binary.
- **The fast runner subset** (`tools/test/suite.py --only nf_game,nf_formats,nf_audio,
  nf_view,mission_stats_flow,menu_graph_walk,menus_flow,fix_scores_handoff_live,
  menudef_script_runtime,menudef_list_widget,codename_entry`): **871 passed, 0 failed, 5
  ignored (2 min)** — every touched test binary plus the four crates' unit tests.
- Also green in the cycle: `walkreal_07000004_live`'s
  `the_results_page_opens_stats_and_dossier_and_scores_the_real_tallies`.
- **The full suite (874 test binaries) did not run** — this sandbox kills background jobs
  between tool calls, and the foreground build cycles (a full-disk cleanup, two environment
  invalidations of the alsa build script, and the 874-binary enumeration) consumed the window.
  The subset covers the touched crates' unit tests and every menu/stats-family binary.

## Live checks / render spots for the main agent

- `--page stats` render against the PS2 (Del from the results page on 07000048): the title
  "Mission Stats" at (50,40) left, the hint "~B Back" on the (50,419) bar, the 007 stamp at
  (340,198), the logo at (434,38); the table's Category column left at x 68 and the three
  right-aligned value columns ending at 353.6 / 459.4 / 580.9 (640 space); the header row reads
  Category / Current / Target / TOTAL; the row pitch (16.7 [S]) against the game's — if the
  game's stride is visibly larger, the fix is the `pitch` constant in the `Page::Stats` arm.
- The pause SCORE tab (07000005, the pause page's third tab): the header row should now read
  Category / Current / Target from the bank (previously the fallback literals).

## What's left

- The full-suite run on the final tree (the driver's next window can start the runner
  immediately; the build is cached).
- The row-pitch confirmation against PCSX2 (the render spots above).

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures to the
session.
