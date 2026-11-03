## Your task: `real-health-00agent-06`, finish 07000006 on real health at 00 Agent

origin/s/level-06-agent-guardroom-descent made 07000006's Agent sister green (the wedge was the west-wall pilaster corner catch). Branch from origin/master and merge that branch in if it is not on master yet. Add a 00 Agent sister (`walk_the_castle_interior_to_the_party_real_damage_00agent`, difficulty 3, loaded with `load_with_carry_difficulty`) that asserts `Drones::difficulty == 3` at load. Check the difficulty-3 rows the game reads: the pain scale, the accuracy, the min-difficulty spawns and any 00-only gates [M]. Where Bond dies or wedges, fix the port where it differs, otherwise the tactics. The sister must pass, and the Operative and Agent walks stay green. Real input only: no health, ammo or channel writes, and no teleports. Never lower damage or health to make it pass. Fix the port only where it differs from the game, with the address. Shared test helpers (crates/nf-view/tests/common/) change only behind a per-level opt-in. Run walkreal_07000006_live, rook06_realpath_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-06`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-11-03T19:57Z
