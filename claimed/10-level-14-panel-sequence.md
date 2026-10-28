## Your task: `level-14-panel-sequence`, Countdown Part 1's sabotage sequence after the two panels

On Countdown Part 1 (07000014), after the capture scene ("Sabotage base security and escape from room"), destroying the two panels in that room does not run the game's scripted sequence: the room is supposed to blow up (and more), and the door out stays locked until that sequence has finished; meanwhile guards are heard trying to open a door outside the room. Find the whole chain the panels start [D the panels' placements, their death channels, the SPs/scripts/explosions/channels downstream, the door's unlock channel] [M Destroy_Smash 0x12bb60, the scripts' ops, the explosion and door-unlock paths] and port every missing link so the sequence plays and the door unlocks when the game unlocks it. Real-input test from the capture hand-back: destroy both panels, the sequence's channels/objects fire in the game's order, the door opens, the escape objective proceeds. List any piece that needs a draw (explosion effects) in the report for the main agent.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-panel-sequence`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-10-28T02:53Z
