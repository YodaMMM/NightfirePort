## Your task: `capsule-cull-and-los-masks-3`, finish the head-on cull + per-caller LOS masks

Start from `origin/s/capsule-cull-and-los-masks-2` (merge it, then origin/master). Round 2 kept the head-on cull [M Intersect_CylGeom 0x109da0:302-322], the per-caller LOS masks [M Collide_Intersect 0x1ebaa8:300-319, Collide_Pick 0x1eb320:140-152] and the GT_Track ray end at the candidate's world sphere [M 0x17f9c8:271-278]. Its report's "What's left" list is this task:
1. Turret deploy (gt_los_live `the_open_front_spot_still_deploys_the_gun`, gun_turrets_live `raising_ch18_deploys_the_wall_gun_and_its_shots_wound_bond`): fill `GTurret::muzzle_offset` = ext+0xd0 = the script-A union model bbox max.z x 1.5 [M GT_Create 0x17ffa0:140] and start the track ray at `t.pos + fire_dir * muzzle_offset` [M 0x17f9c8:230-258]. The live 0700000a gun reads 1.9865224 [L ee_start_0700000a.bin]; check the placement model's bbox max.z x 1.5 against it.
2. interact_07000009_live armour take (:329): trace the 0x721 take ray's hit faces and surface bytes.
3. mp_net_slots_post_move_live `four_human_session_digest_is_pinned`: find the state byte that moved the digest's top byte, then re-derive the pin from the game's rule.
4. zoe_gondola_realpath_live, level_end_live (:270), walkreal_07000008_live (:164): instrument the shifted rays and fix or re-pin from the game's behaviour.
Every one of these must be green on the full suite before the branch is done.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-26T14:38Z

---
## Report (done, 2026-10-26T16:07Z)

