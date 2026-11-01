## Your task: `level-03-fall-through-floor`, 03: Bond falls through the floor while flanking servant 1's guard

On 07000003, while the walk flanked servant 1's guard in the courtyard, Bond fell through the world (y reached -5577) [seen on origin/s/stealth-los-2's first suite run; that walk's route no longer passes there]. A player can do the same. Reproduce on real input near servant 1's captor (find the spot by walking the old flank — see `walkreal_07000003_live` history on origin/s/stealth-los-2 before commit 003102622), find the collision gap (a missing/one-sided floor face, the Dist2Tri flags 0/99 skip [M 0x10a330], a seam between two floor meshes — compare `NDrone2_Collision` 0x140e90's gap bridging for drones; for the player Player_FeetOnPoint 0x194188 / Player_Collision 0x194df0), fix it with the address cited and pin: Bond walking that line stays on the floor (sister: a probe at the seam). Run every *07000003* binary, the 0a/01 walks and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-03-fall-through-floor`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-01T07:14Z
