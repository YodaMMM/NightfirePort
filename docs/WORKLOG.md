# WORKLOG

## capsule-cull-and-los-masks-2 (branch s/capsule-cull-and-los-masks-2)

Task: finish the head-on cull and the per-caller LOS masks from s/capsule-cull-and-los-masks
(both rules already unconditional there; 4/10 failing groups green): interact_07000009_live's
body-armour take, level_end_live, walkreal_07000008_live, gun_turrets_live,
bathhouse_shout_mixer_live, mp_net_slots_post_move_live, zoe_gondola_realpath_live, gt_los_live
(mask 0x72f from master), then the full suite on the final tree.

Progress:
- branch created from origin/s/capsule-cull-and-los-masks (f0cda139), merged origin/master
  (kept the corrected unconditional head-on cull arms and the per-caller masks from the branch;
  kept master's LOS_TURRET_TRACK 0x72f const used by gt_los_live and the drone turret track),
  worklog started, pushed.
- next: reproduce the 6 failing groups on the final tree, read the round-1 report, fix by
  game evidence [M], full suite, report.
