## Your task: `level-02-bunker-stick`, Bond stuck in mid-air between the bunker doors

On 07000002 Bond can hang in mid-air at (-45.42, 7.1, -22.15), between the two bunker doors, for thousands of ticks — not grounded, the floor 5.33 below (found while making the real-health walk pass; origin/s/fix-02-real-health avoids the move that leads there — read its walkreal_07000002_live.rs changes). Reproduce on real input in a test, find the cause (a door leaf's collision or pose [M Door_Interp 0x135738; Door_ReactToHit 0x135980], a capsule push that holds him [M 0x10a1f0-0x10a2e8, 0x10a330], the grounded/feet-band rule [M Player_FeetOnPoint 0x194188], a ledge), fix with evidence, and pin it (Bond lands and stays grounded; sister: the old behaviour sticks). Run walkreal_07000002_live, interactables_07000002_live and every door/collision binary the change touches. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-02-bunker-stick`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-31T20:33Z

---
## Report (done, 2026-10-31T20:57Z)

