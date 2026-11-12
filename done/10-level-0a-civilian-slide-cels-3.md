## Your task: `level-0a-civilian-slide-cels-3`, the portal drawn gate without breaking other levels

Start from origin/s/level-0a-civilian-slide-cels-2 merged with origin/master. The gate (the game's portal-visible room rule; patrollers now walk with their gait instead of the no-gait slide) and the 0c re-tunes are right; the branch's own run left these RED: walkreal_07000001_live x4 - FIRST look at the two that are not fight walks: `the_plate_refuses_bond_alone` and `the_exit_fade_starts_on_the_exit_tick_runs_the_games_300_frames_then_the_level_loads` (a drawn-gate change should not move a plate's filter or the exit fade: find the path, it may be a real regression), then the contact walk sisters; walkreal_07000005 `..._real_damage_00agent`; walkreal_07000008 `..._real_damage_agent`; walkreal_07000013 `..._real_damage`; guard1_rounds_0700000c_live `guard1_rounds_at_the_ring_real_health_fight` (Bond off the ring west edge). walkreal_07000011/15 were not run. For each: name the drawn-gated arm that moved it; a real regression gets fixed (with the address), a fight that only moved gets a real-input re-tune or an #[ignore] naming a task. Then the full suite (all binaries, background, long enough). No draw changes.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/30-level-0a-civilian-slide-cels-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-11-12T04:44Z

---
## Report (done, 2026-11-12T05:49Z)

