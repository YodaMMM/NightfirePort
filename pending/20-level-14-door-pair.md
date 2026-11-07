## Your task: `level-14-door-pair`, both 07000014 door guards reach the locked door as they do in the game

`the_door_guards_hold_combat_no_route_like_the_game` in crates/nf-view/tests/door_guards_locked_door_07000014_live.rs is ignored, and its message names the gap: only #783 routes to the door, while the released group-9 pair #778/#780 wake into no-route combat at their own spawns and shuffle there. The game's two guards stop 2.4 apart at (-51.6,41.2)/(-51.2,38.8) [L 2026-10-30]. Read done/10-level-14-no-route-hold.md, done/10-level-14-door-guard-release.md, the 10-level-14-missing-group-guards report and done/20-drone-nobunching-side-bits.md in the queue first, and do not repeat what they rule out. Confirm the end state on current master, because those reports may have changed it. Then find what gives the game's pair their route: follow the release channel, the alert or goal that sends them, and how `NDrone2_SetDroneAlertRoute` 0x154b20 or the no-route arm picks the door side, and name where the port diverges. Land that first. Only then look at `NB_SCAN_LIVE` in drone.rs: it stays inert unless walkreal_07000014_live stays green with it on, and that flip is not this task's goal unless the pair needs it. Remove the ignore when green. If the release needs a live read, list exactly what to read under the report's live checks and keep the ignore with the new reason. Run door_guards_locked_door_07000014_live, walkreal_07000014_live, interactables_07000014_live, walkreal_07000001_live, the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-door-pair`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

