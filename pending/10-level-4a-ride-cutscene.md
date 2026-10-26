## Your task: `level-4a-ride-cutscene`, the 0700004a lift-ride cut-scene shows its satchel and floor

On 0700004a the ride cut-scene (SP #289, pack 0x060006cf, ticks ~1611-2186) plays with pieces missing: the satchel the guard is holding is not there, and Bond stands on nothing (the lift floor/platform is absent during the scene). Decode the scene's streams [D the pack 0x060006cf: Entity / Anim / attach ops], list every object it spawns or attaches (the satchel prop on the guard's hand bone, the lift car/floor glists) with the op that does it [M the script player 0x1d94c0 family, Script_GetObj 0x1db688, the attach/link ops], and port the missing ops or the missing object lookups so every object the game spawns exists in the port's scene at the right bone/transform. Tests: during the ride ticks the scene carries the satchel entity attached to the guard's hand and the lift floor object under Bond [M + D].

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-ride-cutscene`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

