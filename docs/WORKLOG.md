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

## capsule-cull-and-los-masks-4 (started 2026-10-26)

Plan: branch from origin/s/capsule-cull-and-los-masks-3 (ca2e7389) + merged origin/master,
pushed. The two round-3 leftovers:
1. walkreal_07000008_live walk_the_gondola_to_the_briefcase_and_back (:164, ch 134 never
   written) — env-gated [burn] trace at the contact site + NF_WALK_TRACE on the fusebox->safe
   legs, separate the stance stall from the burn, fix the cause (push-tail gate suspect).
2. mp_net_slots_post_move_live four_human_session_digest_is_pinned (:285) — baseline tree at
   the pin's commit a7afd219 in its own CARGO_TARGET_DIR, per-tick component hashes on both
   trees, first divergent component+tick, fix the port or re-derive the pin with the cause.
Both must be green; the turret/armour/zoe/level_end groups must stay green.

### level-11-rope-guard-hitbox (started 18:53 UTC, 2026-10-26)

Task: on 07000011 (Phoenix Fire stairwell) shots at a roping guard go through him (holes land on
the wall behind) and spent casings hang frozen in mid-air. Compare the port's per-bone boxes for
the abseil states (the rope pose, DroneFunc_HandleImpact, the bone hit test Collide_Jointy
0x1ec180) with the game's [M], from the stairwell landings at (14.11, 34.03, 0.26) and the east
connector looking at p258 (18.895, 46.516, 1.434) on ch 0x13. Casings must fall and settle
(Bullet_DoTrails/casing physics [M]). Master has segment_bullet_scoped + the abseil damage arms
(Init/Slide/Hang case 8 [M 0x169610:48-50, 0x169730:68-70, 0x169940:50-52]).

Plan: branch s/level-11-rope-guard-hitbox from origin/master (7b4cabf2) pushed as checkpoint 1;
suite runner started in the background; census the bullet-pick path + the abseil pose/jointy hit
test in the port; write the failing mouse-look tests per abseil state; fix with [M] citations;
casing settle test; full suite; report + finish by 20:30 UTC.

## level-11-rope-guard-hitbox (18:51–20:30 UTC, 10-26)

- Branch from origin/master 7b4cabf2 pushed as checkpoint 1 (18:56). Probes committed (19:36,
  20:03, 20:25): the sim-side Jointy pick hits dead-on chest aims in INIT/SLIDE/HANG over the
  flat AND the scoped worlds (nf-view live test green, the scoped flight pick never stops the
  aim ray); the HANG state's CallAnim(2) never starts (danim 11 persists) — recorded, not the
  miss's cause.
- The frozen-casings fix (20:25): kept casings now rest on the static floor beneath the eject
  point (`rest_y` down-ray at eject/arm; casing.rs `eject_with_floor`/`reserve_with_floor`, the
  tick_inner bones closure computes the floor) instead of the mid-air [M] sound plane — the rope
  guard's kept shells fell 10+ units to the deck. keep=0 behaviour untouched (game-exact).
- Ran: nf-game lib (0 failed, incl. casing 7/7), nf-view rope_guard_hitbox_07000011_live (3
  passed / 1 ignored). The background suite predates the final tree — rerun is next round's
  first step. Report: docs/reports/level-11-rope-guard-hitbox.md.

## level-11-abseil-hang-pose (started 20:14 UTC, 10-26)

- Branch s/level-11-abseil-hang-pose from origin/master a9298ef5, merged
  s/level-11-rope-guard-hitbox (e61f0f95: the rest_y casing fix, the per-state shot tests, the
  ignored kept-casing pin), pushed as checkpoint 1 (20:16).
- The abseil anim chain AUDITED against the decompile (the task's premise included): the game's
  CallAnim 0x13ab58 queues a pending record (+0x558, +0x55a=1) — the danim is picked per frame by
  CallHandler 0x13add0 → GetDAnimForCall 0x13db30 (the current state's transition list, else
  state 0's) → SetAnimCall 0x13def8 → SetDAnimInternal 0x13aee8 (+0x56a = info[danim].state,
  reached+ZeroAnimCall when the new state equals the folded pending want, SetEndAnim 0x1e when
  the danim has no script). The abseil danims exist ONLY in anim sets 15/17 [D tables]: 0xb
  (0x892, state 2), 0xc (0x89c, state 3), 7 (0x893, death 0x40). The port forces set 0xf on the
  abseil placements [M DefaultInit's +0xdc save] and its call_anim_variant/call_handler model
  the queue — a tick-by-tick probe (both worlds) shows the port ALREADY walking the game's
  chain: Init danim 11 → Slide danim 12 (the state-2 transition key 3) → Hang RESTARTS danim 11
  (state 3's key 2 — the hang pose IS state 2's clip, the same id as the rope-grab; the
  predecessor's "danim 11 persists / CallAnim(2) never started" read the final state only — the
  12→11 restart was invisible to it). Death 0x92 plays danim 7 then the class-9 follow-on 191.
- Pinned (nf-view rope_guard_hitbox_07000011_live, 7 passed / 0 ignored):
  the_abseil_states_play_their_own_anims — per-state danim/script/anim-state with the restart's
  fresh anim.time, the death pair, and hang-pose ≠ slide-pose [M 0x169610/0x169730/0x169940/
  0x169b28 + the tables]. The kept-casing pin UN-IGNORED: the census answers it — the roper's
  weapon 0x19, ammo byte 5, casing_mesh[5] = 0x02000047 ≠ 0 [D]; the old fixture's Bond sat on
  the landing the stairwell flights shadow from the rope (los never rose, the guard never
  fired); the pin's eye now sits at the rope top and the shell arms at tick 10 through the real
  sight→fire→reserve→arm path, sounds on the [M] plane (45.22 = shooter − 1.38) and RESTS on the
  floor below (15.003). Fixture gained the real `surfaces` (AmmoData) — the old None left the
  census table empty.
- Suite: full-run constraints this window (2-core sandbox, background jobs killed between tool
  calls) — see the report for what ran on the final tree.
- Final tree (merged origin/master dcc4a733): nf-game lib 578/0/5i, nf-view
  rope_guard_hitbox_07000011_live 7/0/0i, nf-game rope_guard_hitbox_live 4/0. The full
  four-crate suite run is the next round's first step (background jobs die between tool calls
  here; a 10-min foreground chunk never passed the build). Report:
  docs/reports/level-11-abseil-hang-pose.md.
