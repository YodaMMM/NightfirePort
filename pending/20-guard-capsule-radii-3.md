## Your task: `guard-capsule-radii-3`, a guard bumped by Bond is pushed the way the game's is, proven in play

Branch from origin/s/guard-capsule-radii-2, merge current origin/master, and read its docs/reports/guard-capsule-radii-2.md. The real-input down-bump proof is still the ignored guard_capsule_radii_live. Take its lead: 07000006's start guards, the drop harness's tase subjects on the same floor as the walked approach, whose gatehouse leg already works. Fallbacks in its order: corner a fleeing guard, or the drop-harness tase plus a teleport-free final creep starting inside the AnyoneNear band [M 0x162eb0 case 0xc]. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Real input only, never lower damage or health. Run guard_capsule_radii_live, walkreal_07000006_live, the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-capsule-radii-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

