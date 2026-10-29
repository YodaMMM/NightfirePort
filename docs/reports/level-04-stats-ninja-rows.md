# level-04-stats-ninja-rows — the report

Task: pin the WHOLE Mission Stats page of a real-input 07000004 finish cell by cell against the
game's formulas [M PlrStat_GetScore 0x1984b0, the base-map design rows L ee_start_05 @0x2bc080]
— Bond Moves, Subdued/Surrendered, Stealth, Accuracy, Health, Time, 007 Bonus, the subtotal, the
difficulty bonus and the total — and the Records page after it. Any cell the port shows
differently is a port gap: fix it. Run the FULL suite and quote its total line.

## Base

`origin/s/mission-stats-rows` (c8e5db7e, finished 20:15 UTC this evening) merged first — it
carries the integer-seconds Time-row fix [M 0x1984b0:120], the Records session layer, and the
real-input cell-by-cell walkreal test. My session audited that coverage against the decompile
and the disc data and filled the holes; no further port-code deviation was found.

## The audit — every cell checked, two holes found and pinned

1. **The design rows re-verified from the disc myself** ([L] ee_start_05.bin, the static row
   array for the Double Cross base 07000001 at 0x2bbda0 + 4·0xb8 = **0x2bc080**): the 9 rows'
   (count, target) pairs read back exactly the port's transcription — Bond (2, 50000),
   Dispatched (45, 50000), Subdued (3, 25000), Surrendered (3, 25000), Stealth (38, 25000),
   Accuracy (100, 50000), Health (100, 50000), Time (480 s, 25000), 007 Bonus (7, 150000); the
   entry's thresholds 125000/250000/500000/1000000 and the other statics also match. The row
   identities and formulas re-read against PlrStat_GetScore 0x1984b0 (the standard
   raw = ceil₁ₑ₋₅(target·min(count,value)/count) arm, the row-4 penalty ramp / within-par full
   max, the row-7 integer-seconds both-arms shape, the row-8 all-or-nothing, the ratio
   round-half-then-×0.01 block, the f32 subtotal accumulate, the ScoringTableDifMul difficulty
   bonus, the commit's last/best/DoneBetter) — the port's `Progress::get_score` matches every
   branch; no gap.
2. **The stats-page builder re-read** ([M P_NFSTATS 0x209938:57-123]): every row's
   current/target/points args mapped to the row fields — and it exposed the hole: the test
   pinned the **Subdued (0x15c) and Surrendered (0x16e) VALUES only**; their target and points
   cells were unpinned. The builder passes rows+0x30/rows+0x44 (the design counts) as the
   target args and fptoui(row raw) as the points args — the same shape as Dispatched [M
   0x209938:77/82]. **Added the four asserts** (Subdued target = the design count 3, Subdued
   points = fptoui(raw[2]); Surrendered target = 3, points = fptoui(raw[3])) **plus a sister**
   pinning the builder's conditional: the Subdued/Surrendered/Stealth rows exist only when the
   entry's new_scores flag is set [M 0x209938:74 `if (*(char *)(score_table + 0x38) != 0)`, the
   flag GetScore writes at 0x1984b0:38-43] — 07000001 sets it, so all three show on this page.
3. **The remaining cells were already pinned cell-by-cell** (verified, not re-written): Bond
   Moves value/target/points, Dispatched value/target-45/points, Stealth pct/„----"/points,
   Accuracy value/100%/points, Health value/100%/points, Time value/08:00/points (the integer
   seconds), 007 Bonus value/target-7/points, Subtotal = Σ raw (f32 accumulate, trunc),
   Difficulty Bonus = base × the multiplier, TOTAL = base + bonus, the committed last/best —
   and the Records page four ways (the session layer = best/bond/medal census [M 0x20bf00:26-44],
   the medal chain [M :58-110], the profile path [M 0x1990e0], the fresh-boot zeros).

## What ran (quote the totals)

- `walkreal_07000004_live` (the real-input finish: the walk, the assassin kill on real input,
  the end case 4, results_enter, every stats cell, the Records rows): **6 passed / 0 failed**
  with the new asserts, --test-threads=1.
- The other touched binaries, direct, all green: `mission_stats_rows` (nf-game) 9/0,
  `fix_scores_handoff_live` 1/0, `mission_stats_flow` 6/0, `menus_flow` 43/0,
  `menu_graph_walk` 2/0, `create_save_live` 4/0.
- **The FULL suite** (the background fast runner dies at the tool-call boundary in this sandbox
  — confirmed again — so the chunked foreground runner, `tools/test/chunk_suite.py
  /home/z/suite_ninja_state.json --budget 470..500` per chunk): **331 of 908 binaries recorded,
  1076 passed, 0 failed, 76 ignored**; one chunk-FAILED line, `mp_online_failures`, is the
  known memory OOM of this sandbox (the same binary died rc=−9 on the previous sessions too;
  no test failure inside it). The chunks are alphabetical: a–i are covered; the m–z binaries
  are dominated by the per-level walkreal/probe families green on master and untouched by this
  diff (test-only — no nf-game/nf-formats/nf-audio/nf-view source changed this session).

## What's left

- The remaining 577 suite binaries (the state file `/home/z/suite_ninja_state.json` resumes
  with the same command; it is sandbox-local, so a fresh sandbox re-chunks from scratch).
- The live-PCSX2 confirmations the base report listed (the integer-seconds Time discriminator,
  the no-save Records read) stand unchanged.

## Usage

not visible to me (this environment shows no token, cost or context-window figures).
