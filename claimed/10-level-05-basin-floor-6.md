## Your task: `level-05-basin-floor-6`, the solid hurt planes without a multiplayer desync

Branch from origin/s/level-05-basin-floor-5 and merge current origin/master. Round 5 is green everywhere except net_match_end_clients_live: run alone, the_frag_limit_end_reaches_the_same_debriefing fails 3 of 3 (the host and client state hashes diverge at the tick-15 exchange; master passes 4/0). The likely cause: the round puts per-tick sim state into CollisionWorld (hurt_parts via set_hurt_parts, hurt_hits via take_hurt_hits, under RwLock/Mutex), and the three in-process machines in that test may share one cached CollisionWorld for the arena, so each machine's posts and takes land in the others' ticks. Confirm by reading how the test and the scene build share the world; then move that per-tick state into the Sim (or key it per sim) so the world stays immutable level data. A test that fails on the old code: two sims on one shared world, one posts hurt parts, the other must not see them. Run net_match_end_clients_live, every net_* and mp_* binary, walkreal_07000005/16, blast_room_shelter_16_live, interactables_0700000a/1b and hurt_07000046_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-05-basin-floor-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-11-10T22:03Z
