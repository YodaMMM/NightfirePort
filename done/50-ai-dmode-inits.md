## Your task: `ai-dmode-inits`, the per-mode guard inits

The 29 `NDrone2_init_DMODE_*` functions (0x14d978..0x14e568: Guard, Retreater, Sniper, Stealth, Attacker, RunToPoint, Assassin, HostageKiller, Hostage, HostageTied, JustStand4Demo, DeleteMe, Civilian, CivilianScared, MissionFailer, Mayhew, Ninja, AlarmRaiser, SearchLight, Zoe, PartyGirl, Interogator, CivDoorGuard, TruckDriver, CastleChatGuard1, CastleChatGuard2, SniperAlert, PartyGirlLooker, Bot) are never mentioned in `crates/*/src`. The port sets up guards through `NDrone2_DefaultInit` 0x14b300's mode table in its own words.

1. Find the dispatch that calls them (the mode byte -> init table) [M] and, from the placements [D], which modes each on-foot level uses.
2. For every init: compare what it writes (state, flags, weapon, voice, speed, alert set, the first state) with what the port sets for that mode; fix every difference [M] with a test per mode family on a level that uses it, and put the function name and address at the port's code for that mode. A mode no on-foot level uses: cite the proof (the placement census) in a comment instead.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/ai-dmode-inits`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-10-24T18:10Z

---
## Report (done, 2026-10-24T20:20Z)

Closed by the driver: the report is on `s/ai-dmode-inits` (docs/reports/ai-dmode-inits.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-10-24T20:42Z)

