## Your task: `reload-and-bond-hitboxes`, verify the guards' reload animation and Bond's 16 hit boxes

Two items are marked unverified: the guards' reload animation, and Bond's own 16 hit boxes (no port code found by name).

1. Reload: find the game's reload path for a guard [M DroneWeap_* reload functions, the anim id it plays, when the clip refills] and compare with the port. Fix any difference; pin the anim id and the refill tick with a test.
2. Bond's hit boxes: find the game's player hit volumes [M the player's collision/hitdata setup, the 16 boxes' bones and sizes] and where the port decides what a guard's round hits on Bond. Port them if missing; a test fires a round at each region and checks the hit part (and the damage scale per part if the game has one).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/reload-and-bond-hitboxes`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

