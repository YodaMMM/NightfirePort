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
