## Your task: `level-0a-ceiling-gun-route-2`, 0a: finish the crouch-under-#607 real-damage route

Continue `origin/s/level-0a-ceiling-gun-route` (merge `origin/master` first; read its report — blocked/10-level-0a-ceiling-gun-route.md on the queue branch). It found beam #607's band at 1.49-1.76 [D] is clear of a crouched body (1.3 [M Player_Collision 0x194df0 case 3]) so a crouched player never raises ch 0x1F and ceiling gun #640 stays dormant, while the low beams (#621 at y 0.26) are dodge-only. Finish: the outbound nav via and the fatal band legs so `walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch_real_damage` completes on real health crouching under #607 (un-ignore it), pin the laser band rule (crouched passes #607, standing raises 0x1F; the low beam fires either way), and run every *0700000a* binary and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-ceiling-gun-route-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-01T12:26Z
