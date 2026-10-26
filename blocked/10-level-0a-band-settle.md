## Your task: `level-0a-band-settle`, Bond wedges on 0700000a's doorway bands

Three walkreal_0700000a_live tests are ignored because the port's walk differs from the game's on the ring route: the port glides at the settled 1.0891 but does not reproduce the doorway base climb's +0.0256 band settle [L 1.1147 fresh boot]; at the corner the port wedges (y dithers across the band edge 1.1347) where the game walks through at y 1.127 -> 1.120 [L fresh boot]; and the ring walk (:227) wedges at 'the ai lane to the N-S corridor'. Find the step/ground-settle rule the game applies on these band edges (`Player_Update` ground probe + step-up [M]), port it, and un-ignore the three tests. Every other walkreal test must stay green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-band-settle`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-26T14:02Z

---
## Report (blocked, 2026-10-26T14:36Z)

