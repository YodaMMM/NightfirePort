## Your task: `level-07-library-guards-attack`, The Exchange party fails when a guard is attacked first, reached on the real path

Branch from origin/master. walkreal_07000007_live::the_prop52_guard_first_attack_fails_the_mission is ignored: the library door guards #404/#417 hold the doorway until Drake's reveal (ch 21), which takes the hall's 0x52 carriers off and ropes the west doorway (#495); #424/#425 never get a line of sight in 1800 ticks. Find a real-path point where a 0x52 carrier can be attacked before ch 21 (another carrier, another room, or before the library scene) [D the carrier list and channels], and pin that attacking one fails the mission the way the game does [M]. If no carrier is reachable in the game either, prove it [M/D], delete the test, and say so. Run the whole walkreal_07000007_live binary and interactables_07000007_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-07-library-guards-attack`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-11-10T07:52Z

---
## Report (done, 2026-11-10T09:23Z)

