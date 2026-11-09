## Your task: `level-0d-operative-walk-2`, 0700000d can be walked to the end on Operative

Branch from origin/s/level-0d-operative-walk, merge current origin/master, and read its docs/reports/level-0d-operative-walk.md plus the Operative walk's ignore message. Take its levers in order. First the crouch-dodge test at the corner: if the bay/yard opening line clips a crouched chest, the corner wait goes quiet and the hop fires clean. Then the drop-then-sweep: land in the yard and sweep at once, while the garrison is still asleep. The standing-capsule change belongs to its own task. Helpers stay gated to 0700000d and the difficulty, never lower damage or health. Real input only, no top-ups. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_0700000d_live binary, interactables_0700000d_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-operative-walk-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-09T04:03Z

---
## Report (done, 2026-11-09T05:19Z)

