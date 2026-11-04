## Your task: `difficulty-gated-placements`, every difficulty-gated placement on the on-foot levels matches the game

The difficulty pick changes more than damage. Drone_Create 0x136be0:49 skips placements whose min-difficulty param is above the current difficulty. Lock_Create 0x1ae1f8:67-69 auto-unlocks only on Operative. The captain criterion reads prop 5 at difficulty 2 [M 0x14b300:504-535]. Find every other placement creator that reads the difficulty (GameState+0x28): pickups, ammo amounts, doors, locks, triggers, spawners, turrets and cameras [M; grep the decompile for the 0x2a3790 global's readers]. For every on-foot level and each difficulty 1/2/3, pin a live test (new file `difficulty_placements_live.rs`) of the port's spawned and locked set against the [D] placements filtered by the [M] rule. Fix every reader the port is missing. Leave the walkreal_* files alone: other tasks are editing them. Run the new test, interactables_*_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/difficulty-gated-placements`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-04T05:24Z
