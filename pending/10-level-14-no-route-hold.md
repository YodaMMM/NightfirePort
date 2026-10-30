## Your task: `level-14-no-route-hold`, guards with no route hold still like the game

Master now has the A* door-link gate (locked doors block routes [M AINetwork_DoAStarPath 0x1571b0:262-271]) and the 0.5 mission-route arrival [M NDrone2_InitAIPath 0x153720:113]; `door_guards_locked_door_07000014_live` pins that 07000014's released guards no longer work the locked west door. One divergence remains there: in the game the guard holds COMBAT_NO_ROUTE 0x5f the whole time (17 live samples at 30-frame spacing, CUR 0x5f / PREV 0x5a, opponent pointer set, grace 0 [L 2026-10-30 PCSX2, Bond held in the generator hall after ch 0x87+0x88]); the port's guard (#25) cycles 0x5f -> STEP_AIM_LEFT 0x73 / STEP_AIM_RIGHT 0x74 / COMBAT_NO_SIGHT 0x5c every ~90 ticks. The game's NoRoute UPDATE [M NDrone2_DSTATE_CombatNoRoute 0x164c08 case 3] keeps the state when NDrone2_ChooseCombatMove returns 0, 0x67 or 0x6f, and only changes for other picks. Find why the port's ChooseCombatMove picks the step-aims here (no los, no seen) and what the game's returns [M NDrone2_ChooseCombatMove; cite lines], fix it, and extend the 07000014 test to pin: the guards at the door hold 0x5f. Run every *guard*/*drone*/*combat* binary and the walkreal binaries; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-no-route-hold`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

