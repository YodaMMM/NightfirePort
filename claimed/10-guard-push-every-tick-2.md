## Your task: `guard-push-every-tick-2`, the every-tick guard push without breaking 20 tests

Continue `origin/s/guard-push-every-tick` (merge `origin/master` first). On master its every-tick drone-player push [M Collide_Jointy 0x1ec180:203-281, gates NDrone2_DoCollision 0x149ec8] turned 20 binaries red: walkreal 01 (x3 + real_damage timeout), 02 (timeout), 07 (x2), 0c (x2), 11 lower floor; escort_follow_07000001; exit_aim_01 (timeout); interactables_07000001 camera sweep, interactables_07000003 breakables; cutscene_timer_pause; party_door_lock_07000007 (x2); speech_facial_fallback; guest_faces; probe_07_route. Many are non-walk tests (07 party guests and doors, cut-scene timers, speech) — so the push likely moves Bond (or guests/allies) where the game does not: check which drones the gates admit (allies, party guests, scripted/cut-scene actors, non-combat NPCs — voice set +0xd8, +0x538, obj+0xFC drawn, +0x570&2), whether the push applies during cut-scenes/disabled player, and the push magnitude/direction vs the game. Fix the port where it differs; fix walks only where they are fragile. Run every binary above, every walkreal_* and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-push-every-tick-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-01T16:02Z
