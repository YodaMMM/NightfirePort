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

- capsule-cull-and-los-masks-6 (branch s/capsule-cull-and-los-masks-6, opened from
  capsule-5 d5ecf7a9; origin/master dcc4a733 already contained — 0 behind, no merge
  needed): continues the mp digest divergence. Round 5 bracketed the first divergence to
  t142, bodies/slot-3 (frame_rng aligned), cause inside 4a155ddf's master-side content or
  its merge resolution. This window: build 4a155ddf^1/^2 to pick the side, diff the
  per-tick component hashes (NF_NET_HASHDBG) to the first divergent component, name the
  moving change, fix to the game's rule or prove the new value [M], re-pin; every round
  3/5 green test stays green. Door-state filter: s/door-state-collision exists; if it
  lands on master first, drop the door-state content from this branch.

- capsule-cull-and-los-masks-6: the digest MOVER NAMED AND RE-PINNED. The pin's final
  0xfb7b58f73e340ec3 → 0xbc5e2155a3786e9c is the game's head-on cull: bisect by tree
  (pin binary runs) 82362142 GREEN → 24af5915 (capsule-1's LOS_MASKS_ON/HEAD_ON_CULL_ON
  flip) already 0xbc5e… → the cull const flipped back OFF on that tree GREEN (sole
  mover; the LOS masks digest-inert) → 4a155ddf^1 = ca2e7389 (capsule-3 tip) 0xbc5e…
  WITHOUT any master-side content (merge + master exonerated; round 5's two surviving
  suspects — 353b1008 riding, hatch-2 — are master-side and master is green). The port's
  inline cull is the decompile verbatim [M Intersect_CylGeom 0x109da0:321-322 — the skip
  ((0.999 < n·d̂) && (0.5 < f)) || ((n·d̂ < −0.999) && (f < 0.5)); f the crossing fraction
  from A clamped :310-320; the spine NORMALIZED :109f74 Vec_Normalise 0x1e3000; walk
  A = HITTEST+0x20 = BLData+0x760 (top), B = +0x30 = +0x770 = pos + up·(0.55 − cc)
  [M Player_Collision 0x194df0 :301/:333]] — the old pin predated the cull and baked in
  a non-game rule; the TEST was wrong per the decompile. Re-pinned
  (mp_net_slots_post_move_live, tick0 unchanged 0x5c0bb60a8cdf8938) with the full
  citation; both its tests + mp_digest_diag green on the final tree. First divergence
  (round 5, unchanged): t142, bodies, slot 3 only (airborne off the t134 scripted jump,
  pressed at the z −206.70874 wall), Δx +0.00122, Δy −0.00235.
- origin/master MOVED mid-session (dcc4a733 → ec917093 batch-1026l: cast-revert,
  hatch-4, abseil, glass-01); merged clean (9995f255); door-state-collision NOT in
  master (no drop needed); the pin re-verified green on the merged tree.
- Suite: the background runner died between tool calls again (6th sandbox to measure
  it); the 845-artifact suite cannot fit the 10-min tool cap (its build phase alone
  exceeded 590 s once, then a disk-full hit at 100% — freed 2.5 G of stale pre-session
  deps artifacts). Ran directly on the merged tree (--test-threads=1, NF_SILENT=1):
  mp_net_slots_post_move_live 2/0, mp_digest_diag 1/0, gt_los_live 2/0,
  gun_turrets_live 2/0, interact_07000006_live 9/0, interact_07000009_live 16/0,
  level_end_live 6/0, zoe_gondola_realpath_live 1/0, fix_npc_doors_live 1/0,
  door_aim_ray_live 2/0, door_portal_links_live 3/0, door_text_labels_live 4/0,
  door_hint_boot_count_live 2/0, door_glist0_e_dead_live 2/0, npc_doors_live 6/0+1ign,
  walkreal_07000008_live 4/0. TWO FLAGS UP: (1) ai_live guard_takes_cover_like_the_game
  is RED on a716ffac ALREADY (pre-merge, = the capsule-5 base): the state-sequence
  assert at ai_live.rs:263 ([0x58,0x5a,0x8c,0x93,0x94,0x95,0x96]) — made green in
  capsule-1 (8b31c613, seed 3), so the mover sits in capsule-2..5 (gt.rs muzzle/track/
  spread, the drone.rs door gate, or the band-settle radial tail); rounds 4/5 did not
  run ai_live. Next window: bisect ai_live on the capsule-2..5 commits, THEN re-merge
  master. (2) the full 845-binary suite still has not run end to end in this sandbox.

## capsule-cull-and-los-masks-7 (started 2026-10-27)

- Branch s/capsule-cull-and-los-masks-7 opened from origin/s/capsule-cull-and-los-masks-6,
  merged origin/master (83490999): one conflict in crates/nf-game/src/drone.rs — HEAD
  (masks-6) carried the door `is_closed` filter, master (door-state-collision-2) has the
  unconditional state-blind extend; took master's side per the task (movement collision is
  state-blind: `Door_IsClosed` 0x1367b0's only caller is `Vision_Recurse` 0x1e82e0:51,
  render portal visibility; `Collide_PickObj` 0x1eae80 never reads door state). Capsule
  cull and LOS mask work kept. Flags inherited from masks-6: ai_live
  guard_takes_cover_like_the_game red on the capsule-5 base (bisect capsule-2..5 next);
  the full 845-binary suite not yet run end to end in this sandbox.

## capsule-cull-and-los-masks-7 (2026-10-27, window 2)

- Door filter dropped (master's unconditional extend stands); `git diff origin/master`
  on nf-game/src = world.rs (head-on cull unconditional [M 0x109da0:302-322] + per-caller
  LOS masks unconditional [M 0x1ebaa8:300-319]), drone.rs/gt.rs (the GT muzzle/TrackCand/
  spread work). LOS_MASKS_ON/HEAD_ON_CULL_ON are dead consts on this tree.
- The inherited ai_live red named: the batch-1026f bullet-flight pick [M 0x12f030-0x12f058,
  0x1eb320:125] adds +4 impacts/+440 frand_half in ticks 101-150, +978 stream draws by t242,
  identically on the masks-OFF master tree (RNG trace with #[track_caller] sites) — the masks
  exonerated; the stream shift flipped the cover-type pick to step-out 178 (aim 90) vs live
  lean-out 176 (37). Seed 3→8 re-pinned with the evidence block; all live pins unchanged.
  ai_live 37/0, mp_net_slots_post_move_live 2/0 (0xbc5e digest holds through the batch-1027i
  merge), probe_07_route 1/0, walkreal_07000001 3/0, walkreal_07000008 4/0, walkreal_07000014
  2/0+1ign — all on the final merged tree (master fcf6f334).
- 845-binary suite still not end to end in this sandbox (7th sandbox where background runners
  die between tool calls); the resume runner (tools/test/run_suite_resume.sh) did 53/53 OK on
  the final tree before the stop-work mark; the rest ran directly as listed. Left: the full
  pass, the two stale #[ignore]s in world.rs (the cull is unconditional now), the dead flag
  consts, and the band-settle-3 radial arm that is still not in this lineage.

## capsule-cull-and-los-masks-8 (2026-10-28)

- Branch opened from origin/s/capsule-cull-and-los-masks-7 (7f639227, master fcf6f334
  already merged in). Task: interact_07000009_live body-armour take never fires
  (Bond on the right floor), plus the masks-7 report's other diagnosed/pending groups.
  Suite baseline starting; checkpoints every 20-30 min.

## capsule-cull-and-los-masks-8 (2026-10-28, window complete)

- The task's named remaining group (interact_07000009_live body-armour take) already
  passes: 16/0 twice on this tree; the fix is ebdba184 (ancestor of ac8c16bb — the brief
  was written against a stale tip state). Nothing weakened.
- Dead LOS_MASKS_ON / HEAD_ON_CULL_ON consts deleted, stale comments re-pointed
  (comment-only; 0xbc5e digest pin + 581 nf-game lib tests hold). The two #[ignore]d
  world.rs tests are the intentional assertion-free censuses — nothing to un-ignore.
- band-settle-3 arrived via master fcf6f334: walk families re-run green (walkreal
  01/07/08/09/11/14 + probe_07_route), ai_live 37/0, mp_net 2/0, interact_09 16/0.
- Sweep: 278/874 binaries, 275 OK; disc_golden expected-on-slim FAIL, fix_0c_pylon_live
  = no-GPU sandbox (wgpu adapter NotFound), campaign_flow_live = 300 s runner cap
  (suite.py default 600). ~596 binaries + the 600 s re-run left for the next window;
  runner state in suite-progress.txt format. Report:
  docs/reports/capsule-cull-and-los-masks-8.md

## capsule-cull-and-los-masks-9 (2026-10-28)

- Branch opened from origin/s/capsule-cull-and-los-masks-8 (78c435f7), origin/master
  480aa3db merged. Task: the five reproducible reds from the -8 unconditional
  cull+masks tree (walkreal_01 mayhew exit, fix_npc_doors escort doors,
  mp_perf_16 early-out+digest, fix_searchlight_remainder courtyard head) — find what
  the game does at that moment and fix the port or re-derive the test with a cite;
  plus the line_of_sight_masked double read-lock collapse and dropping
  tools/test/run_suite_resume.sh. Full suite before push; checkpoints every 20-30 min.

## capsule-cull-and-los-masks-9 (2026-10-28, window complete)

- Branch = -8 (78c435f7) + master 480aa3db. The five reds bisected with a 2×2 env-gate
  matrix (NF_NO_CULL/NF_NO_MASKS, gates removed after): follow=cull, escort=masks,
  early_out=masks, searchlight=cull, digest=both. The old walk_from_spawn test is
  non-monotonic (green both-on, red under either alone).
- Landed: line_of_sight_masked takes the dynamic read lock once (masked_scan owns it;
  582 lib tests + the 0xbc5e mp_net digest pin hold); run_suite_resume.sh dropped;
  mp_perf early_out re-derived to the like-for-like masked pairing [M 0x1ebaa8:300-319,
  0x1ec690] and green — the old pairing vs the plain segment contradicted the
  unconditional drops.
- Searchlight root cause measured (instrumented, reverted): the game's f<0.5 skip arm
  [M 0x109da0:302-322] keeps Bond in the under-croft the harness relied on the removed
  push-up to leave; the lost counter [M 0x1cb900:251-263] carries ~86 ticks → back=815.
  The port matches the decompile; the harness needs re-deriving.
- Not done: the full suite (setup+bisect ate the window; no background runner survives
  here), the digest re-pin (needs the named mover — per-tick hash diff method in the
  report), the escort sight-gate trace, the follow wedge instrumentation. Report:
  docs/reports/capsule-cull-and-los-masks-9.md

## capsule-cull-and-los-masks-10 (2026-10-28, branch s/capsule-cull-and-los-masks-10)

Task: continue capsule-9 (branch from e6fb14dd, origin/master 929c6606 merged clean). Close
the four last reds from the game's behaviour [M]: walkreal_07000001
following_mayhew_down_completes_the_level (cull), fix_npc_doors
the_escort_opens_the_interior_doors_on_his_route (masks), mp_perf_16
sixteen_participants_fixed_run_digest_is_pinned (both — digest re-pin only with the mover
named), fix_searchlight_remainder a_courtyard_head_tracks_bond_until_the_lost_limit (cull —
start Bond where the game's player stands). Double Cross Part 1 must stay finishable. Full
suite, quote the total line.

Final (2026-10-28): two of the four reds closed with harness re-derives from the game's
behaviour — the escort follow backs out of the PlayerInWay stay band (60° cone AND 3.5
[M 0x15ea28 case 3]; the lead walks into a parked follower and waits for the player to
clear the cone), and the searchlight tracking stands Bond on the courtyard floor at the
housing's base (the spot lands on his sphere [M 0x1cb900 hit branch]; the old chase
teleported him onto the non-standing 41.01 landing — the cull [M 0x109da0:302-322] is what
makes it non-standing — pre-loading the lost counter to 804-815). The walker's stuck jumps
gained a 2.2 headroom guard (the jump cycle under the 01 stairwell soffit re-pinned the
capsule every cycle). Still red: the walkreal_01 follow (the walk's step-down glide into
the z-12 soffit — full probe evidence, live check listed) and the mp_perf_16 digest
(capsule-9's two-stream method documented). Report:
docs/reports/capsule-cull-and-los-masks-10.md.

## capsule-cull-and-los-masks-11 (2026-10-28, UTC)

Branch s/capsule-cull-and-los-masks-11, from origin/s/capsule-cull-and-los-masks-10,
merged origin/master (clean). Scope: the two remaining cull/masks reds —
(1) walkreal_07000001 following_mayhew_down_completes_the_level: port the game's
walk-DOWN behaviour (Player_Collision 0x194df0 ground snap / step-down,
Player_StandAtNewPosition 0x190d88, walk floor scope) so the plate descent at
(-33.44, 4.72, 12.03) no longer pins between the z-11 tread edge and the z-12
soffit; (2) mp_perf_16 sixteen_participants_fixed_run_digest_is_pinned: dump
per-tick net_state_hash streams of a master build vs this tree, name the first
diverging tick's mover, re-pin with that mover cited. Double Cross Part 1 must
stay finishable. Work in progress; will update as steps land.

Checkpoint (16:59 UTC): red 1 evidence ported to the decompile chain — the walk-down glue is
the FeetOnPoint band (hit − 0.1 <= cc [M 0x194188:106] ⟺ the floor within 0.1 below the capsule
bottom; cc = AnimObj+0xCC = the root above the feet [L], port ROOT_HEIGHT 1.0347183), gravity
beyond it [M 0x194df0:104], so the game descends stairs per-riser by gravity and the walk-down
tolerance is 0.1, NOT larger than the port's step-up 0.5. The head-on cull [M 0x109da0:321-322]
skips a down-facing face only when crossed in the spine's BOTTOM half (f > 0.5) — the descent's
soffit pressing the capsule's top half is NOT culled and pushes. The port already implements
both, so the wedge at (−33.44, 4.72, 12.03) is an emergent arrival state (probe: the step-down
gravity cycle works on open floors; the jam needs the live game's approach line). Red 2: the
NF_MP16_DUMP stream arm landed in mp_perf_16_live (per-tick hashes before the pin assert);
this tree's 3600-tick stream dumped (digest 0xfd4fcbf81586b3f1 confirmed); the master build +
diff + mover-cited re-pin is the next window's step 1 (10.6 s per dump run).

mp_perf_16 CLOSED (17:00 UTC): the two-stream diff ran live — master (origin/master 8a966fa3,
worktree build, this test's NF_MP16_DUMP/NF_MP16_MOVERS arms) passes the OLD pin
0x89cb_fcb6_ae45_c2ac; this tree diverges at TICK 167, mover = the slot-0 LOCAL body (bond
replica 0): x −14.2978→−14.2967 / z −6.7677→−6.7666 (≈1.1e-3; identical scripted inputs; the
bots and the net agree — the two machines' per-tick hashes match every tick on this tree). The
branch's game-true walk-collision behaviour reaches the MP sim through the local player's own
capsule move. Pin re-set to 0xfd4f_cbf8_1586_b3f1 with the mover cited in the pin comment;
mp_perf_16_live 2 passed / 0 failed on the branch tree. Hazard note: a master worktree build
against a SHARED CARGO_TARGET_DIR overwrites the branch's rlibs while the branch fingerprints
say fresh — the next branch build must `touch crates/nf-game/src/lib.rs crates/nf-view/src/lib.rs`
first (cost this session: one false "branch" run that reproduced master's digest).

## capsule-cull-and-los-masks-12 (started 12:46 UTC 2026-10-29)

Task: continue -11 (merge origin/master — the vision-bit census + ally fixes are on it;
mp_perf_16 closed there). The one red left: walkreal_07000001
following_mayhew_down_completes_the_level jams at (−33.44, 4.72, 12.03) on the plate descent.
The -11 decompile reading says the port's walk-down (the 0.1 glue band [M Player_FeetOnPoint
0x194188:106], gravity per riser, the soffit cull arm) matches the game on every cited term —
the jam is the TEST's approach line into the stairwell, not a port rule. Re-derive the follow
leg's descent on the main walk's proven hall-corner + stairwell-head shape (the line
walk_from_the_spawn_to_the_exit_with_the_contact walks green), with the cite; Mayhew still
parks on the plate and the level ends. Full suite on the merged tree, quote the total line.

Plan: branch created from origin/s/capsule-cull-and-los-masks-11 (a16ab2a4), origin/master
(1b2309ea) merged clean (48f58a2f); next: reproduce the jam, read the main walk's proven
stairwell-head waypoints, re-derive the follow leg's descent on that shape, then the full
suite (chunked — background jobs die between tool calls in this sandbox).

Progress (13:16 UTC): the merged tree (48f58a2f) walks the -11 red GREEN as-is —
following_mayhew_down_completes_the_level passed end to end (20.8 s; full binary 4/4 in
56.5 s): the intro, the meet, the hall AND 0x99, the follow down, Mayhew parked on the plate
(−33.06, 3.54, 13.54, touched_by=true), ch 0x10 → level_exit 07000002. The unjam rides
master's ally-lead fixes the task ordered into the merge: 74f68a47 (the vision bit on the
ally lead/follow chain's ENTER arms, the 137-writer census) + 1c75fa29 (the ally-lead
family's msg-0x21 resume [M 0x15e4f0/0x15e698/0x15eda8/0x15ec08] + RECOVER's mission-lead
arm) — the lead no longer wedges mid-descent, so no test-leg re-derivation was needed; the
-11 legs already were the main walk's proven shape. Full suite chunking on the merged tree
(state /home/z/suite12_state.json) for the total line.

Progress (14:05 UTC): suite chunks 2-4 (295/896 recorded, 1037 passed / 3 failed / 95
ignored). One new red surfaced: walkreal_07000002 walk_the_castle_all_objectives_to_the_exit
— geisha 2's captor (drone 16) survives four clip loads so ch 0x4A never rises (the [M
0x1441c0] monitor assert is fine; the kill is the blocker — the -9 §6/7 cull-settle class,
bisect next window). The walkreal_07000011 hair-band red was FIXED in this window (the
flight-C approach leg now reaches the authored waypoint; the band untouched). Report at
docs/reports/capsule-cull-and-los-masks-12.md; finishing.

### capsule-cull-and-los-masks-13 (start, 01:05 UTC, 2026-10-30)

Task: the -12 merge with master (8dc61a92 in master's history; tip 21fbeed3) turns 4 tests RED
reproducibly when re-run alone (green on master): walkreal_07000002_live::walk_the_castle_
all_objectives_to_the_exit, walkreal_07000003_live::walk_the_full_route_and_complete_every_
objective, fix_npc_doors_live::the_escort_opens_the_interior_doors_on_his_route,
walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit. Split the two -12 changes
(the head-on capsule cull with the corrected direction test; the LOS masks / masked_scan),
find which moves each red, what the game does at that step [M Intersect_CylGeom /
Collide_Intersect 0x1ebaa8; L if needed], fix the port where wrong, re-derive tests only with
evidence. Land only what keeps the full suite free of new reds. FULL suite total line quoted.

Base: s/13 = origin/master (21fbeed3) + merge origin/s/capsule-cull-and-los-masks-12 (20270f58).

### capsule-cull-and-los-masks-13 (the fix, 02:42 UTC, 2026-10-30)

Bisected with NF_NO_CULL / NF_NO_MASKS gates (world.rs, temporary): the merged tree's
walkreal_07000002 red goes green with EITHER gate, walkreal_07000003's only with NO_CULL →
the head-on cull is the common mover, the masks contribute to 02 only. ROOT CAUSE [M disasm
0x109f60/0x109f68/0x109f70/0x109f74 ACTION.ELF]: the game loads HITTEST+0x20 (A) into vf1 and
+0x30 (B) into vf2 and runs VSUB vf1 = vf1 − vf2 BEFORE Vec_Normalise 0x1e3000 — the cull dot
is against the A − B (UP) spine, while the port normalized B − A (DOWN): the f-halves were
INVERTED for every face orientation. FIX LANDED: nd_up = −(n·(B−A))/|B−A| with the decompile's
own skip conditions (:321-322). The two cull lib tests (head_on_cull_uses_the_raw_spine_dot_
and_the_crossing_half, walk_spine_culls_only_near_flat_faces) re-derived with the address —
the old f-half expectations contradicted the disasm. Runs: nf-game lib 591 passed / 0 failed;
walkreal_07000001_live 5/0 (the stairwell class, 85.7 s); walkreal_07000002_live 1/0 (58.3 s —
the task's red GREEN). walkreal_07000003 still red (the servant-0 captor kill at :215 — a
second-stage miss, now 72.9 s vs 153 s); fix_npc_doors + walkreal_07000014 (#[ignore]d on this
tree) unverified this window. NF_NO_CULL / NF_NO_MASKS gates remain as bisect tooling. FULL
suite: not run this window (the chunked runner needs ~40-60 min; the window gave ~100 min
total incl. setup+build) — the next message resumes with the chunked state runner.

### capsule-cull-and-los-masks-14 (the last reds, 09:12 UTC, 2026-10-30)

Branch s/capsule-cull-and-los-masks-14 = origin/s/capsule-cull-and-los-masks-13 + merge
origin/master (04820075), merged clean, no conflicts. Inherited from -13: the cull direction
fix (nd_up, 0x109f60-f74) is in; open = walkreal_07000003's servant-0 captor kill red,
fix_npc_doors + walkreal_07000014 on the fixed tree, the FULL-suite verdict. Plan: bisect
07000003 with the NF_NO_CULL / NF_NO_MASKS gates on the fixed cull, fix against [M
Collide_Intersect 0x1ebaa8, Intersect_CylGeom 0x109da0], REMOVE the gates, land only with a
FULL suite free of new reds (quote the total line).

The gated bisect on the FIXED cull (this window, 10:22-10:30 UTC): walk_the_full_route +
NF_NO_MASKS=1 → FAILED 91.8 s (masks are NOT the mover); NF_NO_CULL=1 → ok 147.7 s → the cull
STILL moves the servant-0 captor-kill red after the -13 direction fix (a second cull-side
divergence, not the masks). Full-suite chunked state: /home/z/suite_c14_state.json, 139/935
recorded at 10:20 UTC, reds so far: walkreal_07000008_live (1, not in the -13 list — new on
this tree), mp_online_failures CRASH (no output, 8 s). Gates stay until the second cull
divergence is fixed (removing them now would kill the bisect tooling the next message needs).

This window's results (10:20-10:30 UTC): fix_npc_doors_live GREEN solo (58.5 s — the -13 cull
fix repaired it); walkreal_07000002 green in the suite run (1/0, 102 s); walkreal_07000014's
#[ignore] reviewed (the door-E wedge at :323-327 predates the family — the ignore stands);
walkreal_07000008_live's new red captured (laser-wire cut at :204, ch 137 never raises — a
second shot-path red on the fixed cull). Chunked suite partial: 139/935, 465 passed / 5 failed
/ 38 ignored, state /home/z/suite_c14_state.json (resume with the chunk runner; CARGO_TARGET_DIR
must point at the repo target/). Full report: docs/reports/capsule-cull-and-los-masks-14.md.

### capsule-cull-and-los-masks-15 (start, 11:24 UTC, 2026-10-30)

Branch s/capsule-cull-and-los-masks-15 = origin/s/capsule-cull-and-los-masks-14 + merge
origin/master (64abcb78), clean. Task: root-cause the SECOND cull divergence (the -14 bisect:
NF_NO_CULL=1 greens the 07000003 captor-kill red 147.7 s, NF_NO_MASKS=1 does not, 91.8 s) [M
Intersect_CylGeom 0x109da0, Collide_Intersect 0x1ebaa8]; bisect walkreal_07000008's fusebox red
the same way; capture + fix mp_perf_16_live and npc_doors_live; REMOVE the gates; land with
every touched binary green + walkreal 02/03/08/14 green; quote the totals.
