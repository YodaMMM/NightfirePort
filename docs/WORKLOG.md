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

## capsule-cull-and-los-masks-4 (2026-10-26, this window)

Progress:
- walkreal_07000008 :164 red NAILED: a stance stall against the OPEN secure-room door leaf's
  collision. Three env-gated traces landed (NF_BURN_TRACE drone.rs lock block, NF_PUSH_TRACE
  world.rs capsule_tri_contact, NF_DOOR_TRACE walker.rs). The stall: Bond freezes at
  (154.50,43.1..43.4,-120.64) 1600+ ticks; the bolt sits 4.18 out vs the laser reach 4.0 [L]
  — the walk, not the burn. The blocker face (x=155.003, z -118..-123, y 36.75..42.75, mat
  0x0a) tracks door #10's open pose; the door's keys [D] swing it 180 deg by quat (pos moves
  0.004) and the leaf's natural pose IS that panel. The port poses every glist door into
  set_dynamic unconditionally; the game's door solidity rides the CLOSED-state zone [M
  door.rs]. Fix recipe: gate the door parts by state==CLOSED in drone.rs's doors.extend +
  run the door battery. NOT landed (shared collision surface, window closed).
- NF_WALK_GHOST cross-check: the push is load-bearing on 08 (Bond falls out of the world) —
  the through-push stays. The mp digest pin: method only (NF_NET_HASHDBG needs no code change;
  baseline build at a7afd219 didn't fit).
- left: land the door-state fix + full battery, the mp pin diff, the suite total line.

## capsule-cull-and-los-masks-5 (2026-10-26, this window)

Progress:
- Branch from origin/s/capsule-cull-and-los-masks-4, merged origin/master (world.rs conflict
  resolved to master's landed band-settle-3 radial-dir arm). This window: land round-4's
  door-state fix in drone.rs set_dynamic (doors collide by Door_IsClosed 0x1367b0 — CLOSED
  blocks; OPEN/OPENING/CLOSING leaves do not), review the SS parts (#205 safe door, #217
  bolt) the same way, run the door family, then the mp digest pin per round-4 item 8.
- LANDED: drone.rs set_dynamic's doors.extend now filters by Door::is_closed(&group_states)
  (Door_IsClosed 0x1367b0: kind-1 always closed, grouped reads DoorGroupsStates, else
  obj+0xF4 == CLOSED exactly [M]). walkreal_07000008_live 4/4 GREEN (the briefcase walk
  29.7s, was :164 red 1600+ tick stall). SS parts (#205 safe door, #217 bolt) reviewed: the
  game's script-entity glist path has no state gate [M 0x1d8968/0x1ca6d8/0x1ca798] — they
  stay unconditional; the safe door's swept open pose clears the route (the walk passes
  "before the safe" -> briefcase after ch 136, green). Door family green: fix_npc_doors 1/1,
  npc_doors 6+1ign, portal_links 3/3, portal 2/2, react_path 1/1, aim_ray 2/2, text_labels
  4/4, hint_boot 2/2, glist0 2/2, remainder 4/4, cutscene_14 2/2, party_lock_07 2/2,
  rook_06 1/1, rook_06_walk 1/1 = 37 passed 0 failed. Suite started 21:39 UTC on the fixed
  tree (setsid-detached; round-4's plain nohup died at the tool-call boundary again).
- mp digest (four_human_session_digest_is_pinned): tick0 MATCHES the pin (0x5c0bb60a8cdf8938);
  the final moved to 0xbc5e2155a3786e9c (pin 0xfb7b58f73e340ec3). Baseline built at a7afd219
  (own CARGO_TARGET_DIR /home/z/nfbaseline-target; its NF_NET_HASHDBG run re-verifies the
  pinned values exactly). First divergence: session t142 (sim frame 143), component = bodies
  (slot 3 only: x +0.00122, y -0.00235, everything else identical; the match component moves
  with it through write_net_slot_positions). frame_rng stays aligned at the divergence — the
  round-3 shared-RNG-draw suspect is NOT the first mover. The world component never moves all
  session (Ravine has NO doors, no takes, no channel writes — the door-state fix is provably
  inert on this level). Probe exonerations (env-gated single builds): the band-settle-3
  radial-dir arm (NF_NO_RADIAL) and the bit-0x20 static skip (NF_KEEP_BIT20) do NOT produce
  the divergence. Cause commit narrowed to a7afd219..4a155ddf (rounds 1-3 + their batch
  merges); the diag harness only compiles on trees containing a7afd219 (net_tick/net_rng),
  so older lineage checkouts (level-14-complete-2 etc.) cannot run it — the next window
  bisects the master-lineage merge points (62aa121a, 1875e21f) or probes the grounded-gates
  fix (be9d1c6f, chute-lip FeetOnPoint band) as an env-gated revert on HEAD. Diagnostic test
  landed: crates/nf-view/tests/mp_digest_diag.rs (the pin's session shape printing the
  per-tick fold h0; the component API version is in the report).
- suite: background runners die at the tool-call boundary in this sandbox (setsid included,
  fifth sandbox to measure it); the full four-crate suite did NOT fit. What ran (foreground):
  nf-game lib 581/0, the door family 37/0 (fix_npc_doors, npc_doors, portal x2, aim_ray,
  text_labels, hint_boot, glist0, remainder, cutscene_14, party_lock_07, rook_06 x2,
  react_path), walkreal_07000008_live 4/4, the mp pin test (red, documented above). The
  next window must run the full suite per the standing rule.
- mp digest bisect continued (merge points, the diag harness runs only on trees containing
  a7afd219): 62aa121a (1024h) CLEAN, 15c0e34a (1026c) CLEAN, 82362142 (1026d, rigid-body
  incl.) CLEAN, 18ca71c8 (1026f) CLEAN — each matches the pinned final fb7b58f73e340ec3 to
  t239 — 4a155ddf DIVERGENT. Cause bracketed to 4a155ddf's master-side content
  (batch-1026g/h/i/j) or the merge resolution. Next window: build 4a155ddf^1/^2 (both
  contain a7afd219) to pick the side; suspects: 353b1008 script-actor SP-matrix riding,
  display-hatch-2 toucher final shape, the merge edges. Report:
  docs/reports/capsule-cull-and-los-masks-5.md.

---
door-state-collision (branch s/door-state-collision, worker 2026-10-26/27)
Progress:
- LANDED the door-state filter only, from origin/s/capsule-cull-and-los-masks-5 commit
  1475fc25 onto master dcc4a733: drone.rs set_dynamic's doors extend now filters by
  `x.is_closed(&drones.doors.group_states)` — Door_IsClosed 0x1367b0: a kind-1 door always
  reads closed, a grouped door reads DoorGroupsStates[g] == 2, the rest obj+0xF4 == CLOSED
  exactly; OPEN/OPENING/CLOSING leaves do not block [M door.rs Door::is_closed; the zone
  consumer 0x135738/0x1336f8]. The SS parts review landed as a comment: safe door #205 and
  bolt #217 stay UNCONDITIONAL — the game's script-entity path has no state gate
  [M Script_CreateEntity 0x1d8968 / hashtable_set_object_to_entity_gfx 0x1ca6d8 /
  0x1ca798 / 0x1d9a08]. None of the branch's cull/LOS/gt.rs changes came across.
- GREEN: walkreal_07000008_live 4/4 (the briefcase walk included — the stall the filter
  fixes); the 14 nf-view door-family binaries 33 passed 0 failed 1 pre-existing ignored
  (npc_doors escort-reach); nf-game door tests 11 passed 0 failed 1 ignored (glist census);
  nf-game lib 578/0/5; the mp digest pin four_human_session_digest_is_pinned EXACT:
  tick0 0x5c0bb60a8cdf8938, final 0xfb7b58f73e340ec3 — master's digest did not move
  (Ravine 0700004b has no doors; the filter is inert there, as capsule-cull-5 measured).
- Suite: chunk_suite.py (background runners die at the tool-call boundary in this
  sandbox) on the final tree; totals in the report.
- FINDING (end of window): probe_07_route is GREEN on master, RED under the filter
  (A/B via revert --no-commit / reset --hard in-sandbox): the OPEN kind-0 hall door
  (state=0x4 pos01=1.00 ch_open=0) at (94.9,43.0,-47.2) is load-bearing over the
  44.13->43.0 doorway step — filtered out, Bond hovers (grounded=false) at
  (94.17,44.14,-47.02) to the 6000-tick cap. Door_IsClosed 0x1367b0 is the ZONE rule
  [M 0x135738/0x1336f8]; movement collision likely keeps the leaf always solid with
  passability from the pose (same mechanism as the SS review). Next round: pin door
  #10's open POSE (07000008) vs the game's obj+0x90, then replace the filter with the
  pose fix. mp_online_failures OOMs at this sandbox's 3GB cap (dmesg recorded).
  Full report: docs/reports/door-state-collision.md.

## door-state-collision-2 (started 2026-10-27 ~04:45 UTC)

Task: four red tests under the Door_IsClosed 0x1367b0 collision filter carried on
s/door-state-collision (walkreal_07000001_live contact, probe_07_route STUCK,
walkreal_07000014_live east duct, ai_live guard cover). Find what the walker/guard
hits, decide from [M Door_IsClosed 0x1367b0; zone 0x135738/0x1336f8; drone cover pick]:
fix the port or re-derive the pin. All four green + walkreal_07000008_live 4/4 + all
other walkreal/door/mp tests.
- Branch from origin/s/door-state-collision (441a1199) + merge origin/master (1a3ba0b4)
  = 3ac5db7a. Prior round's finding carried: probe_07_route's OPEN kind-0 hall door at
  (94.9,43.0,-47.2) is load-bearing over the doorway step when filtered; movement
  collision likely keeps leaves solid with passability from the pose, not the zone rule.
- Setup verified: work/, ISO (1,061,754,880 B), env.sh, nf-queue all present.

## level-14-full-walk (started 2026-10-27 ~04:55 UTC)

Branch s/level-14-full-walk from origin/master @ 1a3ba0b4. Task: rewrite the ignored
walkreal_07000014_live full-level walk's leg 1a onto the east-duct route (the SkyBridge
Q-Claw crossing [D GRAPPLEPOINT_SkyBridge (45.29, 5.10, 62.94)] -> open slot drop -> trench
nav lane 5->4 -> corridor; mezzanine entry from inside the east duct, throat x 44.75..45.75,
z 49..50 [D probe14_south]), add the shaft entry, then walk legs 2..6 to the level exit on
real input only. Un-ignore the test when green; every other walkreal/door test stays green.

## level-14-full-walk (2026-10-27, this window)
Leg 1a rewritten onto the east route and UN-IGNORED: the pinned crossing chain green to the
stairwell foot, then NEW the z-51 west pit jump onto the 0.60 segment (46.29, 1.26, 50.75)
[D probe14_shaft2], then the SHAFT ENTRY solved as the Q-Claw reel to mouths' grapple #626
(45.30, 4.79, 42.86) — the lip-skimming ray (0.03-0.09 clearance [D probe14_shaft4]) + the 1.0
sweep [M 0x12c5a0:92] latches from the segment, the reel hangs (45.37, 4.59, 43.91) over the
east vent lane. The shaft is a floor-less well (open < -20 [D probe14_shaft3]), the lip is
+1.41 over a 0.78 jump apex [M], zero ladders [D probe14_ladders], no nav nodes above y 1.0
[D probe14_nav] — the reel IS the entry. The one red step: the release-onto-the-lane (the
phase-2 no-fire detach [M gadget.rs:490] vs the observed hold at y 4.54) — the next session
debugs that loop, then leg 1f (the lane south + the connector west to #301 [D probe14_tunnel/
probe14_mouthfin]) and the untouched legs 2..6. Suite did not fit (background kills).
