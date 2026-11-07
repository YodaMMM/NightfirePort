## Your task: `mp-score-lines`, the match score panel shows the game's lines in every scenario

`MP_GetPlayerScore` 0x189730 prints three lines (score, kills, deaths) for Assassin and KOTH FFA and two for everything else [M :13-27], and `HUD_MPUpdatePane` 0x19f0a0:50-51 formats it beside `MP_GetTeamScore`. `MpScoreRow::score_lines` in crates/nf-game/src/mp/hud.rs ports the split, but nothing calls it. The live rows go through `display_score`. Read 0x19f0a0 in full and compare what the port's panel prints in each scenario: which values, which order, the team score colour arm at :53, and the number format. Then use the ported split on the live panel and delete whatever it replaces. Work only in the data the panel reads where you can, and list the frames for a render check under the report's live checks for the main agent, one for Assassin and one for a two-line scenario. Pin through real matches on Assassin, KOTH FFA and Deathmatch: score a kill and read the panel's text. Run the mp hud binaries (grep tests for `MpScoreRow`), every mp_*_live binary and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-score-lines`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

