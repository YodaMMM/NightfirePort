## Your task: `mp-bot-combat-moves-2`, multiplayer bots' combat moves pass every multiplayer test

Branch from origin/s/mp-bot-combat-moves, merge current origin/master, and read its docs/reports/mp-bot-combat-moves.md. The port of chooseCombatMove's evasive gates compiled and its unit pins pass, but the required runs did not fit the window. Run every mp_*_live binary, mp_world_sync_live and the nf_game lib whole, one binary per tool call, with --test-threads=1 on the memory-heavy ones. Fix any red the change causes in the game's way (never by removing the game's draws), and say for each whether it was the change or master. Then pin through a real bot match that a bot under fire takes the game's evasive moves. Multiplayer state must stay equal on every machine online.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-bot-combat-moves-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

