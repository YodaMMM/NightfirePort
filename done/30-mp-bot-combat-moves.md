## Your task: `mp-bot-combat-moves`, multiplayer bots pick their combat moves with the game's checks

The markers-mp-bots report (origin/s/markers-mp-bots, docs/reports/markers-mp-bots.md) found that `BOTSTATE_chooseCombatMove`'s EvasiveMove and getMovePossibility gates draw Rand the port never draws. Read chooseCombatMove, getMovePossibility and BOTSTATE_EvasiveMove in full with addresses: each gate, its Rand draws and which move each result picks. Port them in the game's order with the game's draw count, so bots strafe, dodge and close as the game's do. The roll arms (0xD9/0xDA) need the roll animations: port them too if the animations exist on the disc [D], otherwise list them. Pin through a real bot match that a bot under fire takes the game's evasive moves. Multiplayer state must stay equal on every machine online; say which Rand stream the bots draw from and keep it. Run every mp_*_live binary, mp_world_sync_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-bot-combat-moves`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-10T01:25Z

---
## Report (done, 2026-11-10T02:21Z)

