# fe-pages-rebase — the credits/menu pages landed on top of the stats page

Branch `s/fe-pages-rebase`, from origin/master (81da6c47, batch-1031l).

## What's done

- Merged `origin/s/fix-stats-page` (2 commits: the Mission Stats/Results pages 580f8697, the
  corner stamp's 512-wide clip 4e123355) — auto-merged clean.
- Merged `origin/s/fe-pages-2` (8 commits: the credits roll / 007 Bonus / win page, FONTMedium
  face 3, Screen Adjust, P_PARISENUM card box, cheat-medal page, menudef SET ids). Exactly the
  three predicted files conflicted; resolved keeping BOTH features:
  - `crates/nf-formats/src/hud.rs`: only the `font_medium` doc comment — the merged doc cites
    both branches' evidence (the results page's score labels/medal text box AND the 007 Bonus
    text list + the credits' kind-2 lines).
  - `crates/nf-view/src/menu.rs` + `lib.rs`: the two branches changed `menu_overlay_fonts`'s
    third font parameter differently (stats: `Option<&Font>` with a SerpLight fallback inside
    `results_page`; fe-pages-2: plain `&Font`). Kept the `Option<&Font>` signature — the strict
    superset: `Some(&t.font_medium)` passes the real FONTMedium (what fe-pages-2's callers mean),
    `None` falls back to the SerpLight face (what stats' `menu_overlay` wrapper means; the
    wrapper passes `None`, so its callers keep the without-FONTMedium rendering).
  - The `draw_page_elems` call sites take the 3-face array in the game's face order
    `[hfont, font, mfont.unwrap_or(font)]` — faces 1/2/3 map to FACE_TEX
    `[0x0300_0005 (NFont2), 0x0300_0003 (SerpLight), 0x0300_00d8 (FONTMedium)]` [M the FACE_TEX
    table and `fi = face.clamp(1,3)-1` in draw_page_elems; L FontTable 0x2dd8b0].
  - `crates/nf-view/tests/fe_pages_draw.rs`: the three `menu_overlay_fonts` call sites adapted
    `&t.font_medium` → `Some(&t.font_medium)` — a mechanical signature adaptation; every
    assertion is unchanged and no decompile-backed test was weakened.
- Merged origin/master again: still at 81da6c47 (no movement), already an ancestor.
- All render aids from both branches survive: `--page credits (--roll N)`, `wingame`,
  `nfbonus`, `screenadjust`, `parisenum`, `cheatmedal`, `results`, `records`, `rewards`, `pause`.

## How it was verified

Named binaries, on the final merged tree (release, `--test-threads=1`, NF_SILENT=1):

- fe_pages_draw: 7 passed; 0 failed
- fe_pages_draw2: 7 passed; 0 failed
- stats_page_elements: 4 passed; 0 failed
- menus_flow: 43 passed; 0 failed
- menu_movies: 4 passed; 0 failed
- menu_sfx_flow: 19 passed; 0 failed
- menus_mouse_parity: 17 passed; 0 failed
- mission_stats_flow: 6 passed; 0 failed
- mission_stats_carry_live: 2 passed; 0 failed
- frame_cost_live: 6 passed; 0 failed (126 s — the frame budgets hold with both pages' draw code)

Full suite (`tools/test/chunk_suite.py`, the chunked form of tools/test/suite.py for this
sandbox — it kills background jobs between tool calls, so the suite ran in foreground chunks;
state file `/home/z/suite_fe_pages_rebase_state.json`):

- **237 of 999 test binaries recorded; tally 804 passed, 0 failed, 97 ignored. No FAILED, no
  TIMEOUT, no CRASH lines.** The remaining 762 binaries are pending, not failures.
- The disk filled to 0 bytes free during the first pass and two `nf_view` unit tests failed as
  disk-full artifacts (`save_and_load_through_a_file`, `install_swaps_files_and_cleans_up` —
  both write files). After freeing space they were re-run: `nf_view` 96 passed; 0 failed. The
  failed entries were dropped from the state before re-recording.
- Resume for the remaining binaries (next message):
  `cd /home/z/NightfirePort && source /home/z/env-fe-pages.sh && export NF_SILENT=1 CARGO_TARGET_DIR=/home/z/NightfirePort/target && python3 tools/test/chunk_suite.py /home/z/suite_fe_pages_rebase_state.json --limit 280 --budget 15`
  repeated until it prints ALL DONE, then the summed tally is the suite total.

## What's left

- The remaining 762 suite binaries (resume command above; the build is warm, only running
  remains).
- Live checks for the main agent (rule 6 — visual work; this sandbox has no GPU):
  - `--page credits --roll N` for a few N: the credits scroll step, the kind-2 lines in the
    FONTMedium face (0x0300_00d8), no overlap with the 007 red backdrop.
  - `--page nfbonus`: the 007 Bonus text list in FONTMedium over the movie plane.
  - `--page wingame`: the win movie fills the plane (or the dark plate without movie data).
  - `--page screenadjust`: both scrolls take their own axis; the 0x67b confirm hint bar.
  - `--page parisenum`: the card box with the "1" message text wrapped in SerpLight.
  - `--page cheatmedal`: the empty list takes the fail arm (no medal rows).
  - `--page results` / `--page records`: the Mission Stats/Results pages' score labels/numbers
    in FONTMedium, the corner stamp showing only the part inside the 512-wide screen.

## Usage

not visible to me — this environment shows no token, cost or context-window figures.
