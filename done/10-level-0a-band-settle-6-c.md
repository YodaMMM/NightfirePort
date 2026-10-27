## Your task: `level-0a-band-settle-6-c`, the 0a radial tail without lifting Bond everywhere

Start from `origin/s/level-0a-band-settle-6-b` (the radial tail with the flags-99 gate in nf-game world.rs) and merge origin/master. On the full suite it broke 21 tests across many levels: Bond settles about 1 unit high (interactables_07000015 "Bond stands on the south walkway (y 41.315, the floor ~40.35)"), so floor touch plates, walk-over pickups and use-key doors fail (walkreal_07000001/08/0b/11, interactables_0d/11/15/16, interact_06/0c, fix_mission, fix_0c_hatch, fix_08_collision, level_end, walk_0b/12, gt_los, gun_turrets, fix_searchlight_remainder, mp_world_sync, mp_perf_16, mp_bot_armour_online, probe_07_route). Find which part of the change applies outside the case the game gates it to [M Player_FeetOnPoint 0x194188 and the walk's vertical], restrict it to exactly that, and get the whole suite green with the 0a walk tests.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-band-settle-6-c`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-27T06:00Z

---
## Report (done, 2026-10-27T07:31Z)

