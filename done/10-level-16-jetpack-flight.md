## Your task: `level-16-jetpack-flight`, Countdown Part 3: the jetpack escape flight

Read the level-16-every-objective report (on master). The 07000016 walk is real up to obj1's escape; the jetpack (pack) flight substate is not ported, so the escape uses a documented position stand-in. Port the pack flight the game runs [M the player substate family that runs the jetpack: its controls, thrust, gravity, the camera, the end condition], fly the escape on real input, and assert ch 0x90 -> ch 100 -> MissionState 6 -> the 0x7300017 hand-off on the walk. Replace the stand-ins in `fix_finale_live` / `kiko_button_real_walk_live` where the flight now covers them. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-16-jetpack-flight`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-29T21:02Z

---
## Report (done, 2026-10-29T21:55Z)

