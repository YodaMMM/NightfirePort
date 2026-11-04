## Your task: `guard-push-every-tick-3`, finish the every-tick actor push without stalling the real-path walks

Continue origin/s/guard-push-every-tick-2: branch from it, merge origin/master in, and read blocked/10-guard-push-every-tick-2.md on the queue branch. The port pushes Bond and the bots on every tick, as the game does (Collide_Jointy 0x1ec180:229-281, with the per-bot arming from bot-push-per-bot-gates [M NDrone2_DoCollision 0x149ec8]). That session's walker push-dodge is implemented but doesn't clear: the 0a hall-door civilian still holds the leg after 6 dodge cycles (d swings 0.95 to 1.43). Do the report's next pass. Either widen the swing (dodge until d > 2.6, biased toward the walkable side) or make the walkers' nav legs cost live bodies. These are test-side walker changes, opt-in per level. Then re-run every binary it lists as red: walkreal 01/02/07/0c/11, escort_follow_07000001, exit_aim_01, interactables_07000001/03, party_door_lock_07000007, speech_facial_fallback, guest_faces and probe_07_route. Fix each one. No assertion may be weakened. Use tools/test/chunk_suite.py for the full run. Run nf-game and every walkreal_*_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-push-every-tick-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-04T10:57Z
