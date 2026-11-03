## Your task: `real-health-agent-00agent-11`, finish 07000011 on real health at Agent and 00 Agent

walkreal_07000011_live `walk_spawn_to_the_lower_floor_load_point_real_damage` is green on master (level-11-start-fight-real-health-3). Add an Agent sister (`..._agent`, difficulty 2) and a 00 Agent sister (`..._00agent`, difficulty 3). Load them the menu way, with `load_with_carry_difficulty(..., d)` from real-health-agent-difficulty-01-07, merged from origin/s/real-health-agent-difficulty-01-07 if it is not on master. Assert `Drones::difficulty == d` at load. The pick scales Bond's pain (Player_HandlePain 0x1902c8) and the guards' accuracy (DroneWeap_DoBulletAccuracy 0x179740:74-88), and spawns the min-difficulty placements (Drone_Create 0x136be0:49) [M]. Where Bond dies, read the damage log against that pipeline, then fix the port or improve the tactics. Both sisters must pass, and the Operative walk stays green. Real input only: no health, ammo or channel writes, and no teleports. Never lower damage or health to make it pass. Fix the port only where it differs from the game, with the address. Shared test helpers (crates/nf-view/tests/common/) change only behind a per-level opt-in. Run walkreal_07000011_live, start_corridor_guard_07000011_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-agent-00agent-11`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-03T21:09Z
