## Your task: `los-mask-repairs`, the guard sight masks go on without breaking allies, doors or blasts

Branch from origin/s/level-0b-roof-sprint-6 (the guard sight masks: LOS_MASKS_ON = true with the Collide_PickObj 0x1eae80 / Collide_Intersect 0x1ebaa8 envelope in crates/nf-game/src/world.rs), merge current origin/master, and read docs/reports/los-mask-join.md and los-mask-envelope.md. The masks match the decompile and PCSX2 shows the 0b guards seeing across the roof parapet, but a full suite on master + this branch has 13 reds that master does not have. Behaviour first: these look like real envelope gaps, not walk drift:
1. zoe_gondola_realpath_live::zoe_rides_the_carriage_and_never_targets_bond: the ally's round hit Bond 1411 times (team 2 never takes the player [M 0x1424e0:101]). Find which masked query now lets Zoe pick or hit Bond.
2. interactables_07000012_live::doors_open_on_the_walk_and_stay_shut_untouched: E no longer opens door plc 545 [M Door_Activate 0x135be0].
3. fix_mission_live::the_07000001_exit_works_end_to_end: the courtyard shoji ping-pongs 0x8/0x10 and never reaches OPEN (state 4) [M NDrone2_DoorIsOpen 0x14aaa0].
4. fix_ally_death_live::each_protected_actor_fails_the_mission_on_the_games_killers: 07000001 #161, the blast kills the actor.
5. walkreal_09 the_alarm_switch_fails_objective_1 (the walk no longer reaches the ch-124 switch).
For each, find the query (the caller's mask, the object word or class it reads) and fix it the game's way with the address, red on the old code. Then the walks: walkreal_07000001 (TIMEOUT), 02 castle 00agent, 06 castle interior real damage, 11 lower floor real damage x2, 13 warm chain x2, 14 tripbomb route. Re-pin with real input only if the game-correct sight moved the fight; never lower damage or health. Run every walkreal_*_live binary one level per tool call, plus the five binaries above.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-los-mask-repairs`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
