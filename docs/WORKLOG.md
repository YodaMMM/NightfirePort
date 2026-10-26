# WORKLOG

### capsule-cull-and-los-masks-2 (started 20:46 UTC, 2026-10-24)

Task: finish the head-on cull and the per-caller LOS masks. Start from
origin/s/capsule-cull-and-los-masks (f0cda139, not merged): both collision rules are already
unconditional [M Intersect_CylGeom 0x109da0:302-322 cull; Collide_Intersect 0x1ebaa8:300-319 /
Collide_Pick 0x1eb320:140-152 masks], 4 of 10 test groups green. (1) interact_07000009_live's
body-armour take never fires though Bond is placed right — log Bond's path and the take-gate
inputs (Pickup_Update 0x18e148: the distance gate and the take ray Collide_LineOfSight(…, 0x721)),
fix the cause [M]. (2) Run and fix on the final tree: level_end_live (07000016 straddle door),
walkreal_07000008_live (laser burn ch 134), gun_turrets_live, bathhouse_shout_mixer_live,
mp_net_slots_post_move_live, zoe_gondola_realpath_live, gt_los_live (master's turret track LOS
mask 0x72f) — re-derive shifted pins from the game's behaviour, no re-seeding until green.
(3) Full suite on the final tree (merged origin/master), no failures outside slim-ISO/no-GPU.

Plan: merge origin/master into the branch first (final tree base), start the fast runner as soon
as the tree is stable; meanwhile diagnose the armour take from the decompile; then item 2's
groups individually; finish with the runner totals.

## capsule-cull-and-los-masks-2 (resumed 2026-10-26)

Progress:
- resumed in a fresh sandbox: branch re-created from origin/s/capsule-cull-and-los-masks
  (f0cda139), origin/master merged (kept the branch's corrected unconditional head-on cull arms
  and the unconditional per-caller LOS masks; kept master's LOS_TURRET_TRACK 0x72f const used by
  gt_los_live and the drone turret track), this worklog file merged from the first sandbox's
  branch start (534b939d), pushed.
- next: reproduce the failing groups on the final tree, read the round-1 report
  docs/reports/capsule-cull-and-los-masks.md, fix by game evidence [M], full suite, report.

Progress (this window, continued):
- origin/master merged (LOS_TURRET_TRACK 0x72f kept; the branch's unconditional cull arms and
  per-caller masks kept; the master-side scratch gates the branch deleted are deleted in the merged
  tree too).
- GT_Track's per-candidate ray now ends at the candidate's obj+0x80 world-sphere centre
  [M 0x17f9c8:271-278, Control_BuildWorldSph 0x133070] — TrackCand.sph; guards keep the feet
  stand-in [S]. nf-game lib 556 green.
- [L] ee_start_0700000a: gun #636 (ext 0x1b9e400) ext+0xd0 = 1.9865224 — the game's track ray starts
  ~0.29 above the deck; the port's muzzle_offset is 0.0 behind the script-VM gap (GT_Create's
  script-bbox formula [M 0x17ffa0:140]) — the turret deploy groups' blocker, next window's item.
- bathhouse_shout_mixer_live already green on the final tree; mp_net's digest pin moved exactly one
  top byte (0xfb5e...->0xbc5e..., low 56 bits identical); the other groups reproduced, causes in
  docs/reports/capsule-cull-and-los-masks-2.md.
- suite runner: background dies at the tool-call boundary (measured twice); foreground slice ran
  104 binaries / 0 failures; a full pass does not fit the window.

## capsule-cull-and-los-masks-3 (started 2026-10-26)

Plan: branch from origin/master (d4f15b9e) + merged origin/s/capsule-cull-and-los-masks-2
(53c15437), pushed. Work list from round 2's report:
1. GTurret muzzle_offset = script-A union model bbox max.z x 1.5 [M GT_Create 0x17ffa0:140],
   track ray starts at t.pos + fire_dir * muzzle_offset [M 0x17f9c8:230-258] — turret deploy groups.
2. interact_07000009_live armour take (:329): trace the 0x721 take ray's hit faces + surface bytes.
3. mp_net_slots_post_move_live digest pin: find the state byte that moved the top byte.
4. zoe_gondola_realpath_live, level_end_live (:270), walkreal_07000008_live (:164).
Suite runner starts next; the round-2 sandbox measured that background nohup dies at the
tool-call boundary — will retry once, then run foreground slices if it dies again.

## capsule-cull-and-los-masks-3 (2026-10-26, this window)

Progress:
- item 1 done: muzzle_offset fills at attach (script-A static Entity union of the level glist
  bounds max.z x 1.5 [M 0x17ffa0:118-140]) — the 0700000a gun reads 1.9865224 = the live
  ext+0xd0 [L ee_start_0700000a]; the track ray starts at t.pos + fire_dir*offset [M 0x17f9c8:
  232-242]; the GT fire arm runs Bullet_init's 3-draw spread cone [M 0x12c5a0]. gt_los 2/2,
  gun_turrets 2/2.
- the standing-height fixture class: three tests teleported Bond to the bare floor point /
  floor+0.1 — 1.1 under the game's stance [M 0x190d88] — and he fell out of the world
  (measured -569.8 / -372.9). Fixed in gt_los_live, gun_turrets_live, interact_07000009_live
  (settle_near), level_end_live (step_at). interact 16/16, level_end 6/6.
- zoe: the test was wrong (the perp>2.0 proxy contradicts the game's geometry — her opp_targets
  are guards 19/20/24, never the player [M 0x1424e0:101]); re-pinned to the opponent identity.
  Green.
- left: walkreal_07000008 (thread-order-flaky at the :164 burn gate after the final master
  merge — instrumentation plan in the report), mp digest pin (baseline-diff method in the
  report), the full-suite total line.
