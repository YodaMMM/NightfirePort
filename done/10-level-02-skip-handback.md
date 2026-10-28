## Your task: `level-02-skip-handback`, where the game puts Bond when a cut-scene is SKIPPED

Master hands Bond back from an animated (kind-2) cut-scene at the anim object's matrix [M Script_EventHandler 0x1d94c0:73-84,:125-134], matching the game when the scene PLAYS (07000002 intro: (-27.504, 1.131, 32.348) [L]). When the player SKIPS, the port fast-forwards and lands at the anim's END pose (-27.30, 1.23, 28.90) - unverified. Read the game's skip path [M Script_FFwd 0x1dac78, SP_Update 0x1dbda8's skip arm, how event 6 and the anim's interp run (or not) during the fast-forward] and make the port's skip hand-back exactly the game's, for every SP with a kind-1/kind-2 hand-back (the census test lists them). Test both paths on 07000002/03/14 through the real skip input. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-02-skip-handback`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-28T17:28Z

---
## Report (done, 2026-10-28T18:13Z)

Closed by the driver: the report is on `s/level-02-skip-handback` (docs/reports/level-02-skip-handback.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-10-28T18:23Z)

