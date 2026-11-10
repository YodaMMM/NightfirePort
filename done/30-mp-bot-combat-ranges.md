## Your task: `mp-bot-combat-ranges`, multiplayer bots choose their fighting distance as the game's do

The markers-mp-bots report (origin/s/markers-mp-bots, docs/reports/markers-mp-bots.md, "Findings beyond the marker list") found that `NDrone2_DefaultInit` writes Drone+0xEC = 4.0 [M 0x14b300:378] and computes +0xF0 per skin and class [M 0x14b300:383-400], while the port's `BotVars::default` combat ranges are [2.0, 20.0, 40.0]. Settle from the decompile and the disc which values a multiplayer bot really carries (whether its init path runs DefaultInit's writes or overwrites them later), with addresses. If they differ, port them so the too-close and engage bands and the 0xD7 exit bound match, and pin through a real bot match that a bot holds the game's distance. Multiplayer state must stay equal on every machine and must not touch the shared Rand stream. Run every mp_*_live binary and mp_world_sync_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-bot-combat-ranges`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-10T01:41Z

---
## Report (done, 2026-11-10T03:08Z)

