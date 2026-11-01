## Your task: `level-16-chaser-real-damage`, 16: the chaser fight on real health

Bond now arrives at the 07000016 chaser fight armed (the real 15 -> 16 chain save carries the Delta Repeater; origin/master's `walkreal_07000016_live` pins it). Make the chaser fight survivable on real health through walker tactics (return fire with the carried gun, range keeping — the chaser's punch does 50 raw / 20 a hit [M], cover, the blast-room shelters already pinned) — never game code without a cited difference — and add the real-damage variant of `walk_the_chasers_defeat_completes_the_mission` (the overlook route on real input is pinned by s/level-16-stuck-overlook: E at SecurityStation #597). Run walkreal_07000016_live, interactables_07000016_live and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-16-chaser-real-damage`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-01T13:52Z

---
## Report (done, 2026-11-01T17:34Z)

