## Your task: `level-0b-parachute-step-up`, 0b: walking onto the parachute pickup

level-0b-teleports-to-walks (merged; read its report) found the deck walk arrives under the parachute (#280 at (2.53, 58.45, 42.34) [D]) but the helicopter body's volume blocks Bond 2.4 m short (cel 189), so the test falls back to a stance sweep. The pickup floor is 57.92 vs the deck 57.03 at z 42-44. Find how the game lets Bond reach it [D the copter's collision/glist and the pickup's touch volume; M Pickup_Handler 0x18e650, the player step-up in Player_Collision 0x194df0], fix any port gap with evidence, and replace the stance sweep with a walked take. Run walkreal_0700000b_live and any *pickup* binary; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0b-parachute-step-up`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-31T02:20Z

---
## Report (done, 2026-10-31T03:48Z)

