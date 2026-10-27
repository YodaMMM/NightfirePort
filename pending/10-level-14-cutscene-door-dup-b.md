## Your task: `level-14-cutscene-door-dup`, the control-room cut-scene door has a static copy

On Countdown Part 1 (07000014), in the control-room cut-scene (Bond at (40.87, 2.98, 40.85)) the door now moves but a static copy of it stays in place behind it, so it still looks shut. The scene's door leaf is keyed by the script streams (level-14-cutscene-door-2); the level's own placed door (or its static World instance) still draws. Find which instance duplicates it [D placements + the scene's Entity/stream objects] and the game's rule that hides one while the script moves the other [M — Script_HideObj 0x1db808 / SP hide cascade, the door's own draw gate, or the script entity replacing the placement], and fix it so exactly one door draws and it moves with the scene.

A previous session started this on `origin/s/level-14-cutscene-door-dup` and ended before any change landed (its report only holds the plan). Continue from that branch.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-cutscene-door-dup-b`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

