## Your task: `level-01-mayhew-wait-2`, Mayhew waits at the door and the Double Cross exit still works

Start from `origin/s/level-01-mayhew-wait` (the escort holds at the kind-8 route's ally-wait node 33, flag 0x20, until the ambush beyond the door is cleared [M NDrone2_ReachedDestNode 0x153358:85-94]) and merge origin/master. On the full suite it turned two tests red: `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact` ("the AND-touch must fire with Bond AND the contact on the plate") and `fix_mission_live::the_07000001_exit_works_end_to_end`. Both walks reach the exit without clearing the ambush, so the contact now stays at node 33. Make the walks do what the game requires (clear the ambush the hold waits for, on real input) or show from the game's rule that the hold releases another way [M], keep the hold, and get both green plus escort_follow_07000001_live, mayhew_hold_live and fix_npc_doors_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-01-mayhew-wait-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-27T05:54Z

---
## Report (done, 2026-10-27T07:16Z)

