## Your task: `guard-noise-attack-states`, alerted guards react to gunfire as the game's do

The real-health-00agent-01-hall report (origin/s/real-health-00agent-01-hall, docs/reports/real-health-00agent-01-hall.md, live check 2) found that the port's noise chain exists (DroneFunc_SetNoiseHeard 0x149920, HandleSoundAlerts 0x148a30), but the alerted attack states have no handler for the sound message (0x14). Read the game's combat and attack state handlers in full and list which of them handle message 0x14 and what each does, with addresses. Where the port differs, port it in the game's order, keeping the Rand count, and test it red on the old code. Pin it through real play: a sustained pistol burst near alerted guards draws them as the game's handlers say. Then run every walkreal_*_live one level per tool call and repair any walk that shifts, gated to its level. Real input only, never lower damage or health. Run the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-noise-attack-states`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

