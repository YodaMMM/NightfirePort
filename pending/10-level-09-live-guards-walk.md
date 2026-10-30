## Your task: `level-09-live-guards-walk`, walk 09 with the guards live

`walkreal_07000009_live::walk_tower_a_start_to_finish` sets `drones.hold_npcs = true` (walkreal_07000009_live.rs:76), so the guards never act. The level-09-every-objective report (`origin/s/level-09-every-objective`) left: a challenge-state guard took zero damage from the player's rounds. Root-cause that [M the damage path; D the guard's records], fix the port side that is wrong, then make the walk pass WITHOUT hold_npcs on real input (clear each patrol beat, resupply when dry). Every test in the file green. Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-09-live-guards-walk`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

