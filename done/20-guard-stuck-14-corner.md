## Your task: `guard-stuck-14-corner`, guards walk around each other at the 07000014 corner as in the game

Branch from origin/s/guard-stuck-0c-14, merge current origin/master, and read its docs/reports/guard-stuck-0c-14.md, item 1 of "What is left". First pin NDrone2_DroneNearPos's dropped radius from the disassembly at the 0x159dd0 call site [M]. Then port the tier-2 check to the game's drone-list walk, which the three route.follow call sites need (Drone::peers may carry the list), or the leaf push in the collision pass in the game's order (move, then the DeltaP push), whichever the decompile shows. Keep the Rand count and test it red on the old code. Pin it through real play: the 07000014 guards that stalled at the corner walk past each other. Then run every walkreal_*_live one level per tool call and repair any walk that shifts, gated to its level. Real input only, never lower damage or health. Run the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-stuck-14-corner`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-11T04:30Z

---
## Report (done, 2026-11-11T05:07Z)

Closed by the driver: the report is on `s/guard-stuck-14-corner` (docs/reports/guard-stuck-14-corner.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-11T06:09Z)

