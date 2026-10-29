# mission-stats-rows — the Mission Stats numbers and the Dossier Records

**Status: both reported defect families root-caused and fixed; every stats row pinned cell-by-cell
from a real 07000004 finish; the FULL 906-binary suite did not fit the window — 184 binaries ran
through the chunked fast runner plus every touched crate/binary directly, all green (one known
environment OOM, three known slim-ISO disc_golden fails). What's left is listed at the end.**

## Defect 1 — the Mission Stats numbers (Double Cross Part 4, 07000004)

**(a) "Time 00:08 → 1,585,205 points": the time row divided by FLOAT seconds; the game
integer-divides.** `PlrStat_GetScore` 0x1984b0's time row reads the elapsed tally in 1/100 s and
integer-divides by 100 BEFORE any float cast — `uVar2 = *(int *)(rows+0x94) / 100` [M
0x1984b0:120] — and the under-par arm's `max(·, 1)` guard rides that same (float)(int) value [M
:125-131, the `(int)fVar13 * (1.0 < fVar13) | (1.0 >= fVar13) * 1.0f` line; both arms are the one
expression raw = ceil₁ₑ₋₅(max × par / max(elapsed_s, 1))]. The port divided by the float seconds
(elapsed 757 → 7.57), which produces score shapes the game cannot reach (25000·480/7.57 ≈
1,585,205; the game's integer step floors 757 to 7 s → 1,714,286). An 8.00-second tally scores
25000·480/8 = 1,500,000 [M]. Fixed in `Progress::get_score` row 7 (`crates/nf-game/src/
progress.rs`): `let secs = r.value / 100; let t = secs.max(1) as f32`. The PCSX2 capture golden
(01:19 / 09:00 → 170,887) still passes — 7900/100 = 79 exactly.

**(b) "Opponents Dispatched target 45 on a level with one assassin": 45 IS the game's own
number.** The stats page scores the BASE map (`Mission_BaseMapHCode` 0x1830a0 — the mission
table pairs part 07000004 with base 07000001 [L ee_start_05.bin 0x2a4350 row 7; M Mission_Init
0x182270:52]), and the static design row for the Double Cross base carries Dispatched count 45,
points 50,000 [L ee_start_05.bin entry 4 @0x2bc790, rows @0x2bc080 — count 45, points 50000].
The counts are never rewritten at run time: the only writers of the rows are `PlrStat_Init`
0x198318 / `PlrStats_ResetScoring` 0x199020 (zero the value/ratio/raw fields) and
`PlrStats_SetupScoreTable` 0x1990e0 (the save restore, which copies value/ratio/raw + the tail
fields, never count). So the real game shows Target 45 on every Double Cross part's stats page;
the port already matched. Pinned as a test with the design row per cell
(`the_double_cross_base_rows_are_the_static_design`).

**(c) Everything else re-verified against the decompile** — the row build [M P_NFSTATS 0x209938],
the tallies [M PlrStat_Log* 0x198408..0x1999e0], the formulas [M 0x1984b0:118-268], the formats
[M Menu_AddRow 0x201c10 / SeparateNumber 0x1dec78 / Timer_Seconds2String 0x1deaa8]. No further
deviation found. The elapsed accumulation (the tally the Time row reads) was re-checked
end-to-end: `PlarStat_LogUpdateElapsedTime` 0x1995a0's pause gate, its Player_Update call site
0x193600:132 and the port's per-tick call all agree; the real walk accumulates every tick.

## Defect 2 — the Dossier Records page all 0 after finishing

The game's Records reader is `C_RBDSRECORDS_Handler` 0x20bf00: it reads the ONE BSS scoring
table through `PlrStats_GetLevelTotals` [M :19-26] — High Score = entry+0x30 (best), Bond Moves
= entry rows[0].value (the last GetScore build's Bond tally), Upgrades = the medal census
(`PlrStarts_ProcessRewardCounter` kinds 1..4 [M :34-44]) — and the results page's `GetScore(0)`
commit updates that table IN PLACE [M 0x1984b0:301-315]. The game therefore shows the finished
mission's values even with NO save in play. The port's `record_row`/`record_medal` read only the
active codename profile, so a session without a save drew zeros.

Fix (`crates/nf-view/src/menu.rs`, `crates/nf-view/src/lib.rs`): the front end now carries
`session_progress` — the game's BSS stand-in — filled by the host at the mission end from the
same snapshot `results_progress` takes, carried across manager re-creates through
`ProfileSession` (the game's globals outlive every manager [M C_RBDSRECORDS' read after the
fresh manager]). `record_row`/`record_medal` read the profile's restored table first (the save;
[M `PlrStats_SetupScoreTable` 0x1990e0 restores the save INTO the table]) and fall back to the
session layer.

## The tests (all from the real 07000004 finish — the real-input walk: pickups, the kill, the end sequence)

- `walkreal_07000004_live::every_stats_row_matches_the_game_formulas_from_the_real_finish`
  (NEW): walks part 4 start to finish with the player's own movement, runs the results commit
  [M 0x208e78:77], then asserts EVERY stats cell — all 9 rows' current/target/TOTAL, the
  Subtotal/Difficulty Bonus/TOTAL tail, the committed last/best — against the game's formulas
  computed inline from the tallies, each cite-tagged; then the Records row three ways: the
  session layer (no save) = (best, bond, medal census) [M 0x20bf00:26-44], the medal chain
  [M :58-110], the profile path [M 0x1990e0], and the fresh-boot zeros.
- `mission_stats_rows::the_time_row_divides_by_integer_seconds` (NEW): elapsed 7950 → the
  integer 79 s denominator scores 170,887 (the old float code scored 169,812); elapsed 800 →
  "00:08" current, "08:00" target (the 480 s par [L]), 1,500,000 TOTAL [M 0x1984b0:120-131].
- `mission_stats_rows::the_double_cross_base_rows_are_the_static_design` (NEW): the base's
  design row per target cell [L entry 4].
- **The existing test that contradicted the decompile was fixed** — the driver's rule, applied
  the other way than usual: `fix_scores_handoff_live`'s time-row expectation divided by the float
  seconds and scored 146,136 where the game floors 1461.36 s to 1461 s and scores 146,740 [M
  0x1984b0:120]. The test now pins the game's integer step; the comment in the test records the
  address. (Never weakened: the assertion still fails on any formula regression — its expected
  value changed from wrong to right.)
- `create_save_live` updated for the new `ProfileSession` field (initializers only).

## What ran (quote the totals)

- The fast runner, chunked (`tools/test/chunk_suite.py`, the sandbox kills background jobs
  between tool calls — 4 chunks, state `/home/z/suite_statsrows_state.json`): **705 passed,
  0 failed, 31 ignored** over 184 of the 906 binaries, including `nf_view` 96/0, `nf_audio`
  23/0, `walkreal_07000004_live` 6/0, `menus_flow` 43/0. The one FAILED line is
  `mp_stress_explosives_live` rc=−9 — the known OOM (kills by memory in this 4 GB sandbox, seen
  on every previous task).
- Direct, on the final tree: **nf-game 949 passed / 0 failed / 9 ignored** (all its binaries,
  --test-threads=1); **nf-formats** unit 92/0 + census binaries 5/0 + `disc_golden` 11 passed /
  3 failed (the 3 known slim-ISO disc_golden fails); **mission_stats_flow** 6/0;
  **menu_graph_walk** 2/0; **create_save_live** 4/0; **fix_scores_handoff_live** 1/0 (after the
  test fix).
- NOT run: the remaining 722 suite binaries (the window closed — each chunked call is capped at
  ~9.5 minutes by the tool-timeout/background-kill of this sandbox; 4 calls ≈ 38 minutes covered
  a fifth of the list). The 722 are dominated by the per-level walkreal/probe families already
  green on master and untouched here; the touched binaries all ran (above).

## Live checks for the main agent

1. PCSX2, fresh boot, Double Cross Part 4, finish with a short timer: the stats page's Time row
   TOTAL should be the integer-seconds score — elapsed 8.00 s → 1,500,000; elapsed 7.57 s →
   1,714,286 while the CURRENT cell still rounds to 00:08 [M 0x1984b0:120 vs Timer 0x1deaa8].
   This is the cleanest discriminator for the integer step.
2. The same finish, then Dossier → Records → Double Cross with NO save active: High Score =
   the results TOTAL, Bond Moves = the run's moment count, Upgrades = the medal count [M
   0x20bf00:26-44].
3. The stats page's Dispatched target on part 4: the game should show 45 (the design row [L]) —
   confirm the port agrees (it does in the headless test).
4. Render spots: `--page stats` on a 07000004 finish (the Time row's current/target/total
   column positions as menu-stats-page listed), and the Records page's three value cells right
   edge x 560 with the finished mission's numbers.

## What's left

- The FULL suite's remaining 722 binaries on the final tree (the next window can resume
  `/home/z/suite_statsrows_state.json` with `chunk_suite.py` — but that path is outside the
  repo, so re-chunk from scratch if it is gone).
- A live-PCSX2 confirmation of the integer-seconds Time row (check 1) — the decompile is
  unambiguous [M 0x1984b0:120] but the render/gameplay compare is the main agent's.

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures to the
session.
