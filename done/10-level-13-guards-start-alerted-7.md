## Your task: `level-13-guards-start-alerted-7`, the vision-bit census with the escort and Countdown walks green

Continue `origin/s/level-13-guards-start-alerted-6` (merge `origin/master` - it now carries the un-ignored 07000014 delta-exit walk). On the merged tree the full suite has two reproducible reds (both green on master; re-run alone, still red):
1. escort_follow_07000001_live `escort_at_walking_speed_keeps_the_games_distance_and_the_exit_opens` - "the escort left the ally machine: state 0x86" (:137). Your -6 report predicted this one (the corpse fix shifts the RNG stream and the guard-15 fight resolves differently). Find what the game's escort does there [M state 0x86's entry and the ally machine's exits] and fix the port or re-derive the test's fight with the cite.
2. walkreal_07000014_live `walk_the_infiltration_to_the_delta_exit` - "Rook's death channel 0x54 fires" (:858) never fires: with the vision bit Rook now sees/acts differently in the final leg. Find what the game's Rook does [M his states with the sight bit] and fix the port or re-derive the leg (how the player actually beats Rook) with the cite.
Double Cross Part 1 and Countdown Part 1 must both stay finishable. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-guards-start-alerted-7`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-28T14:18Z

---
## Report (done, 2026-10-28T15:41Z)

Closed by the driver: the report is on `s/level-13-guards-start-alerted-7` (docs/reports/level-13-guards-start-alerted-7.md); the worker didn't run finish.sh.
