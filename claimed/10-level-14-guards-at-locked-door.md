## Your task: `level-14-guards-at-locked-door`, Countdown Part 1's guards outside the sabotage room

On 07000014, once both west fuseboxes are cut (ch 0x87/0x88 -> AND #717 -> 0x8C), the guard spawners #720/#1333 start on 0x8C [D]. On the port a player hears guards trying to open a door outside the room for the whole sequence, and they never come through. Find what the game's spawned guards do [M the spawner's spawn, their scripted/route states, the door-use path for a drone at a locked door (Door_Activate by a drone, the lock channel 0x85 and the hatch doors), their barks] - do they wait at a locked door until the cascade opens it, take another route, or come in at once - and port the missing link so they behave as the game's do. Real-input test from the fusebox cuts: the guards spawn, their states/positions over time, the door interaction, and what happens after the E-use cascade opens Door_Slide_11A/B. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-guards-at-locked-door`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-10-28T14:20Z
