## Your task: `level-06-searchlight-state`, The Exchange's searchlights on and off like the game's

On 07000006 a couple of searchlights look switched on (bright) but are not on and cast no beam, while the green-wire fusebox cut correctly turns the others off. Find each searchlight's placement, its on/off channel(s), what the fusebox cut writes, and which look (lit housing, beam, glow) the game shows in each state [D placements; M the searchlight create/update and its draw-state fields]. Port any wrong state in the sim and expose the on/off state per light for the draw; list what each state must DRAW for the main agent. Test the states through the real fusebox path. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-06-searchlight-state`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

