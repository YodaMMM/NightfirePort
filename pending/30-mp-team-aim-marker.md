## Your task: `mp-team-aim-marker`, the crosshair marks teammates and team objectives as the game's does

`Check_Target` 0x1a7810:129-150 has a multiplayer arm: for an aim-ray hit of class 3 it asks `MP_isObjectOnTeam` 0x18b720 and sets the marker bytes at +0x133/+0x132, and for class 2 it asks the same about the body behind `ext+0xc`, which decides the hostile latch. `mp_is_object_on_team` in crates/nf-game/src/mp/rules.rs ports the check, but nothing calls it. Read both functions in full, including which MPSettings words gate them (the decompile reads `_388_4_` and `_408_4_`; the port's comment names other offsets, so settle that and correct the comment). Find what reads +0x132/+0x133 (the crosshair or HUD side) and what the port's match aim does today. Port the arm so aiming at a teammate or your own team's objective behaves as in the game, and so aiming at a teammate does not set the hostile latch, in team games and Assassin alike. The marker itself is a visual, so list the frame a render check should look at under the report's live checks for the main agent. State must stay equal on every machine online and must not touch the shared Rand stream. Pin through a real team match: aim at a teammate, an enemy and a team objective, and read the values for each. Run every mp_*_live binary, mp_world_sync_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-team-aim-marker`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

