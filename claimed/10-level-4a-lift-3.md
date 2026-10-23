## Your task: `level-4a-lift-3`, 0700004a: step off the lift at its stop, as the real game requires

Start from `s/level-4a-lift-2` (merge it into a fresh branch from master). Its walk waits for a hand-back at (1.80, -107.66, -6.83) that the real game never reaches.

Live [L fresh boot, PCSX2, god mode, no input — audits data quoted here]:
- The ride descends at a constant 0.0303 units/frame with the player substate 1, Bond at (4.2555, y, -4.7581).
- The lift stops at y -54.1971 and control returns (substate 0) for about 560 frames.
- With no input, a script re-attaches Bond (substate 1) at (1.7994, -55.09, -3.4534) — the marker key's x/z, without the SP-base composition — and carries him down; at y -95.80 his health drops 100 -> 0 in one frame (god mode on) and the mission fails.

1. Make the port match all of that: the stop height and its window, the re-attach if the player stays, and the death below. Find the script commands and the kill [M/D] (the stream that re-attaches, the volume or script that kills at -95.8) and fix the hand-back composition if the port's differs from the live x/z.
2. Rewrite `ride_the_lift_from_the_spawn_and_walk_into_the_lobby`: ride, then walk off into the lobby during the stop window, real input only; un-ignore it.
3. Sister: staying on the lift ends in the death (mission failure) like the game.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-lift-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-23T23:01Z
