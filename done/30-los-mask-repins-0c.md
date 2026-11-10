## Your task: `los-mask-repins-0c`, the 0700000c walks green again under the game's guard sight

Branch from origin/master (823ff190b turned on the game's guard sight: per-object masks, the cel walk, the sight rotation, the guard's own cel, the own-cel boundary push, FellOutMap). Read docs/reports/los-mask-repairs.md on origin/s/los-mask-repairs (reports do not go to master). These walks were ignored because the game-correct sight moved their fights: walk_the_whole_mission_from_the_start_to_the_0d_handoff, walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage, the_ground_route_crosses_to_the_east_side. Read each ignore message. Round-3 note: a guard stands on the west ledge strip, Bond falls into the gap going round him, and shots from the landing do not hit him; check whether those missed shots are game-correct before re-routing. Then re-route each walk with real input so it passes, and remove its ignore. Never lower damage or health, no top-ups; helpers gated to 0700000c and the difficulty. Run walkreal_0700000c_live (and its sister binaries) and interactables_0700000c_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/30-los-mask-repins-0c`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-11-10T20:24Z

---
## Report (done, 2026-11-10T20:47Z)

Closed by the driver: the report is on `s/los-mask-repins-0c` (docs/reports/los-mask-repins-0c.md); the worker didn't run finish.sh.
