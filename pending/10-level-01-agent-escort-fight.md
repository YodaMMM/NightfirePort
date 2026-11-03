## Your task: `level-01-agent-escort-fight`, survive 07000001's contact escort on real health at Agent

Branch from origin/master, then merge origin/s/real-health-agent-difficulty-01-07 in if it is not on master yet. It adds the difficulty-parametrized load (`load_with_carry_difficulty(..., d)`, the front end's launch path [M C_SBNFDFCTY 0x216ca8:15 → GameState+0x28]); read its report docs/reports/real-health-agent-difficulty-01-07.md first. Agent = difficulty 2 and 00 Agent = difficulty 3. The pick scales Bond's pain (Player_HandlePain 0x1902c8, Plr_DMod_*) and the guards' hit chance (DroneWeap_DoBulletAccuracy 0x179740:74-88). It also spawns the min-difficulty placements (Drone_Create 0x136be0:49) and changes the stats the difficulty rows set [M].

walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent` fails. Bond dies at tick 4023: 42.0 in 4 hits from the unarmed drone #9 (melee, about 10.5 each at the Agent scale) and 30.4 in 21 hits grinding drone #13. keep_off and back_off 4.0 were already on. First check the melee: the reach, the cadence and the damage of DroneWeap_CloseCombatImpact 0x1789c0 → Player_HandlePain at difficulty 2 [M]. Also check when the game's unarmed drone closes and swings, against the port. Fix any divergence. Then play the fight as a player would: the kill order, keeping the contact moving, and the distance. Make the sister pass. Where Bond dies or wedges, read the damage log against the game's per-difficulty pipeline. Fix the port where it differs, with the address. Otherwise improve the walk's tactics: cover, pickups, kill order, timing. Real input only: no health, ammo or channel writes, and no teleports. Shared test helpers (crates/nf-view/tests/common/) change only behind a per-level opt-in. Every new sister must pass, and the existing Operative walks stay green. Run walkreal_07000001_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-01-agent-escort-fight`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

