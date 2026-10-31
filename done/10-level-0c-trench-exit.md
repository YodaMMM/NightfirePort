## Your task: `level-0c-trench-exit`, 0c: out of the display-case trench on real input

level-0c-remaining-teleports-to-walks (merged; read its report) kept the trench exit write `[5.0, 12.35, -112.9]` in `walkreal_0700000c_live`: the glass lintel is at y 13.36 so the crouch capsule (top ~13.16 [M Player_Collision 0x194df0 case 3]) should clear it, yet the crouch exit stalls at z -111.81 (grounded false) while every ray at y >= 11.9 there comes back clear — so the block is below ray height or a port-side push-out on the case-floor seam. Find it [D the cels under z -112.0..-111.6; M the capsule contact path]; and check whether the game's vent drop lands inside the case (~7.3) rather than in the trench (the port's drop) [D the vent/duct collision; M the drop]. Fix the port gap with evidence and walk the exit (delete the write). Run walkreal_0700000c_live, interactables_0700000c_live and every walkreal_* binary; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-trench-exit`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-31T03:01Z

---
## Report (done, 2026-10-31T04:38Z)

