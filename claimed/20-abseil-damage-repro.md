## Your task: `abseil-damage-repro`, 07000011's rappel guards take no damage: find the case the current test misses

Symptom: on 07000011, guards on the rappel rope cannot be killed. `abseil_damage_live` (real rounds at the 0x06000892 rope guard, aimed by writing Bond's yaw/pitch at the guard) passes on master AND on the older code that showed the symptom, so it does not reproduce it.

1. Widen the search, real input only: every abseiling placement on 07000011 at difficulties 1, 2 and 3 (not just the one difficulty-1 rope guard), every phase (Init 0x8e, Slide 0x8f, Hang 0x90), from the stairwell landings a player actually shoots from, with aim from the mouse-look path (`Input` look deltas) rather than yaw writes, and with auto-aim on and off.
2. For each, log where the round ends (the hit object, the rope, the world) and whether the guard's health drops. Find the case where rounds reach the guard without damage, or never reach him.
3. Fix it from the decompile (`Collide_PickObj` 0x1eae80, the abseil DSTATEs' impact cases [M 0x169610 / 0x169730 / 0x169940]) and pin it with a test that fails before the fix. If nothing reproduces, report exactly what was covered.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/abseil-damage-repro`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-24T03:03Z
