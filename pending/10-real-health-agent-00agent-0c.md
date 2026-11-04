## Your task: `real-health-agent-00agent-0c`, walk 0700000c on real health at Agent and 00 Agent

0700000c is the one level with a green real-damage walk and no difficulty sisters: walkreal_0700000c_live has only `walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage`. Read docs/reports/level-0c-yard-climb-real-health-4.md (on origin/s/level-0c-yard-climb-real-health-4 if it is not on master) for the route and the aimed act on the deck sentries. Add an Agent sister (`..._agent`, difficulty 2) and a 00 Agent sister (`..._00agent`, difficulty 3), loaded the menu way as walkreal_07000011_live does, and assert `Drones::difficulty == d` at load. The pick scales Bond's pain (Player_HandlePain 0x1902c8) and the guards' accuracy (DroneWeap_DoBulletAccuracy 0x179740:74-88), and spawns the min-difficulty placements (Drone_Create 0x136be0:49) [M]. Where Bond dies, read the damage log against that pipeline, then fix the port or improve the tactics, gated on difficulty so the Operative walk is unchanged. A sister may stay ignored only with a measured reason and the next lever. Real input only: no health, ammo or channel writes, and no teleports. Never lower damage or health to make it pass. Fix the port only where it differs from the game, with the address. Shared test helpers (crates/nf-view/tests/common/) change only behind a per-level opt-in. Run walkreal_0700000c_live, the climb/mantle group (fix_ladder*_live, ladder_02*_live) and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-agent-00agent-0c`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

