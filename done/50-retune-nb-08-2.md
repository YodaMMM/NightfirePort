## Your task: `retune-nb-08-2`, 08 whole-mission 00 Agent real-damage walk green again (round 2)

Branch from current origin/master; if origin/s/retune-nb-08 is not on master, merge it in first. Read docs/reports/retune-nb-08.md there and the ignore message on walk_the_whole_mission_on_real_input_real_damage_00agent. The Agent walk is green. Take the next lever from that ignore message and win the fight that now kills Bond. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000008_live binary.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-08-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-18T23:22Z

---
## Report (done, 2026-11-19T01:28Z)


---
## Report from the worker (done, 2026-11-19T01:28Z)

# retune-nb-08-2 — report

Branch `s/retune-nb-08-2` = `origin/master` 73822d3a0 + `origin/s/retune-nb-08` merged in
(the d3 ignore + ledger came with it). Task: the 00 Agent (difficulty 3) whole-mission
real-damage walk `walk_the_whole_mission_on_real_input_real_damage_00agent` stays ignored;
take the next lever from its ignore message and win the fight that kills Bond.

## Done

- **The baseline death reproduced exactly** [L the NF_DMG_TRACE run on this branch]:
  t28298 at (112.25, 13.09, −179.16), 59 hits for 100.0 — the same ledger the 2026-11-18
  window recorded. The whole `walkreal_07000008_live` binary: 6 passed / 0 failed /
  1 ignored, 150.25 s (the d1 real-damage Operative walk, the d2 Agent walk and the ally
  tests all green — the gates keep their timelines byte-exact).
- **New measured geometry** [L the trace]: all 9 of the y-46 trio's hits land at the lane
  mouth (149.5-149.6, 46.1, −93.1) — exactly where the transit nav wedges for ~860 ticks;
  ZERO hits west of x 149.5. #14 at 4.2 u, #11 7.3, #2 12.9, all weapon 0xa. The mid-nav
  return fire kills #3 (t741), #2 (t1013), #11 (t1155) but NEVER #14 — the return fire's
  hurting-first pick filters VISIBLE candidates only, and #14 is buried at 3.4 u from the
  deck-head arrival; his 16.0 lands during the #2/#11 stands. Every one of these fights
  runs on the DRY PISTOL (0x1 clip 1 spare 0 — the resupply found nothing on the walked
  corridor), so each tap costs a 60-tick reload cycle: that is the exposure engine.
- **Shipped lever: the deck-head arm-up + the wide fight's skip-continue.** After the
  proven deck-head nav: the wide fight on any surviving buried hurter (the hurt
  bookkeeping names him, the unseen-cover creep opens the line, back-off suspended — its
  4.0 ring reads the 3.4-u muzzle as a crowd), then `scoop_drops(3)` + `arm_fullest` —
  the guards' own dropped 0xa machine guns. And `fight_guards_wide` now skips a failed
  pick instead of breaking the whole fight (the shared `fight_guards_within` idiom).
  d3-gated (`difficulty >= 3 && fight_pkg`, real damage only); no `crates/*/src/` and no
  `tests/common/` files touched, so no other walk's timeline can move.
- **Measured effect** [L the lever run]: the walk reaches 2.5× deeper — t17664 vs the
  t6408-class deaths this window started from. The walkway pair (#7/#8) dies at ZERO cost
  (was −12.0); the pit guard #15 dies in the excursion (t5449); the whole ledger drops
  from 59 hits to 35. The death moved to the BOARDING LEDGE: t17664 at (110.07, 28.31,
  −121.26), where #27 (skin 0x500002f, (100.8, 28.3, −122.2), 9.3 u) bursts 21.0 in 7
  hits with hp 12 INTACT — the return fire never landed a round on him (the same
  hurting-first visible-only blind spot, now measured at the boarding ledge too).

## Levers tried and retired (each on its own real run)

- The lane-mouth pre-empt: end the transit at the measured lane mouth and fight the live
  snipers there. DEAD — the mouth target's own nav chain wedges at (141.0, 50.4, −93.7)
  grounded=false (both `walk_to` and `walk_nav_to`; the upper hall's bench band eats the
  straight segment). The deck-head route is the only clean approach.
- The pre-nav arm-up (the same scoop/arm before the transit): DEAD as placed — at leg
  start nothing is dead yet, the pick and the scoop are silent no-ops, and the arm cycle's
  ticks alone shifted the transit into the south lane. Re-placed post-nav, where it shipped.

## What ran (release, --test-threads=1)

| binary | result |
|---|---|
| walkreal_07000008_live (whole) | 6 passed / 0 failed / 1 ignored (the d3 ledger), 150.25 s |

The d3 test itself ran four times this window (the baseline trace, the lane-mouth lever,
the pre-fix arm-up, the shipped arm-up + skip-continue), each with NF_DMG_TRACE=1; the
ledgers live in the ignore message and the comment block above the test.

**What did not run.** The full four-crate suite does not fit this sandbox's disk: the
four-crate `cargo test --no-run` build filled the 9.9 GB disk at 00:48 UTC (BUILD at 100%
used, `target/` at 5.5 GB; the same constraint the previous window documented). The changed
set is one test file plus docs/ — no `crates/*/src/` and no `tests/common/` files — so no
other level's walk or library test can move; the whole walkreal binary above is the run
this window could afford, plus the background `tools/test/suite.py` start that the sandbox
killed between tool calls (a fresh `target/` never formed; the process was gone at the
next check).

## What's left

- The boarding ledge: give the mid-leg return fire's hurting-first pick the same
  unseen-cover fallback the plain pick has (opt-in via the UNSEEN_FIGHTS thread-local so
  the d2 sister stays byte-exact) — #27-class buried burst rifles are never engaged today.
- The deck transit's 36.0: an arm-up BEFORE the transit, or a scoop inside the mid-nav
  fight loop (shared machinery, opt-in) — the dry-pistol reload engine.
- With that budget back, pin the machine's route period from the flight log and outlast
  the high phase into the low passes (the d2 phase's y 13-24 passes gave five
  standing-slit launches in 2100 ticks).
- The full-suite pass on the merged final tree, when a sandbox with disk for it exists.

## Usage

not visible to me — this sandbox shows no token, credit or context-window figures.
