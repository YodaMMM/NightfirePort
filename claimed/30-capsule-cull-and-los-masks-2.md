## Your task: `capsule-cull-and-los-masks-2`, finish the head-on cull and the per-caller LOS masks

Start from `origin/s/capsule-cull-and-los-masks` (not merged). It turns the two parked collision rules on unconditionally — `line_of_sight` always applies the caller's game mask (`Collide_Intersect` 0x1ebaa8:300-319, `Collide_Pick` 0x1eb320:140-152) and `capsule_tri_contact` always applies the normalized-spine head-on cull [M `Intersect_CylGeom` 0x109da0:302-322] — and has 4 of the 10 previously failing test groups green. Read `docs/reports/capsule-cull-and-los-masks.md` on that branch first.

1. `interact_07000009_live`'s body-armour take: Bond is placed on the right floor but the take never fires. Log Bond's path and the pickup's take-gate inputs (`Pickup_Update` 0x18e148: the distance gate and the take ray `Collide_LineOfSight(…, 0x721)`) and fix the cause [M].
2. Run, and fix what fails, on the final tree: `level_end_live` (07000016 straddle door), `walkreal_07000008_live` (laser burn ch 134), `gun_turrets_live`, `bathhouse_shout_mixer_live`, `mp_net_slots_post_move_live`, `zoe_gondola_realpath_live`, and `gt_los_live` (master now runs the turret track LOS with mask 0x72f). Where a scenario only shifted because a ray now passes a surface the game's mask drops, re-derive the pin from the game's behaviour, not by re-seeding until it passes: say in the report which pins moved and why.
3. The full suite on the final tree must have no failure outside the slim-ISO and no-GPU set. Merge `origin/master` before the final run.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.



---
claimed-by: sandbox3 at 2026-10-24T17:38Z
