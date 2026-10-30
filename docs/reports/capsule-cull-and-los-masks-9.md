# capsule-cull-and-los-masks-9 — the five -8×master reds bisected to their movers; the early-out contract re-derived (green); the double read-lock collapsed; four reds remain, each with the evidence chain and the next step

## What's done

1. **The branch is the task's tree**: `s/capsule-cull-and-los-masks-9` opened from
   `origin/s/capsule-cull-and-los-masks-8` (78c435f7) with `origin/master` (480aa3db)
   merged clean. All five named reds reproduce on it (run matrix below).

2. **`line_of_sight_masked` now takes the `dynamic` read lock once.** The old body took
   `self.dynamic.read()` and then called `masked_scan`, which takes it again — a second
   acquisition on the same thread (a writer arriving between the two could deadlock the
   read-read re-entry). It now delegates straight to `masked_scan`; the dead locals went
   with it. Behaviour-identical walk: `nf_game` lib **582 passed, 0 failed, 2 ignored** on
   the final tree, and the masks digest pin `mp_net_slots_post_move_live` (the 0xbc5e
   cull/masks digest) holds **2/0**.

3. **`tools/test/run_suite_resume.sh` dropped** (environment-specific: the per-binary
   resume state this sandbox family can't use — nohup/setsid background runners die
   between tool calls here too, 9th sandbox measured).

4. **The five reds are bisected to their movers** with a full 2×2 env-gate run
   (`NF_NO_CULL` / `NF_NO_MASKS` over the four binaries, gates since removed; the tree
   under test was exactly -8+master):

   | red | both on (the tree) | cull off | masks off | mover |
   |---|---|---|---|---|
   | walkreal_07000001 `following_mayhew_down_completes_the_level` | FAIL | FAIL | FAIL | **cull** (masks alone also break the old `walk_from_the_spawn_to_the_exit_with_the_contact`, see below) |
   | fix_npc_doors `the_escort_opens_the_interior_doors_on_his_route` | FAIL | FAIL (344 s) | **ok** | **masks** |
   | mp_perf_16 `early_out_segments_match_the_full_search` | FAIL | FAIL | **ok** | **masks** |
   | mp_perf_16 `sixteen_participants_fixed_run_digest_is_pinned` | FAIL | FAIL | FAIL | **both** (moves under either feature alone) |
   | fix_searchlight_remainder `a_courtyard_head_tracks_bond_until_the_lost_limit` | FAIL | **ok** | FAIL | **cull** |

   Note the walkreal_01 binary's non-monotonic fourth test:
   `walk_from_the_spawn_to_the_exit_with_the_contact` is green with both ON (the -8
   lineage's verified state, and green on the -8 tree per its report) but fails under
   either feature alone — the two features' deltas cancel on that route.

5. **`early_out_segments_match_the_full_search` re-derived and green.** The test paired
   `line_of_sight(from, to, LOS_GUARD_VISION)` against the PLAIN `segment` — a pairing
   that was only valid while the per-caller drops were off. Under the unconditional
   drops [M Collide_Intersect 0x1ebaa8:300-319, Collide_Pick 0x1eb320:140-152] the
   0xa27 ray legitimately passes 0x40-surface / 0xd / 0xe / 0x10-glass geometry that the
   plain segment blocks, so the old assert contradicted the decompile — the TEST was
   wrong, fixed in the test file with the addresses in the comment: the guard-vision
   early-out is now paired with the full nearest-hit walk under the SAME mask
   (`line_of_sight_hit`, the `Collide_RayIntersect` 0x1ec690 distance body, same drop
   arms). The plain/bullet pairings (`segment_clear` vs `segment`,
   `segment_bullet_clear` vs `segment_surface_bullet`) are untouched. Final tree:
   **1 passed** (the digest test in that binary is still red — below).

6. **The searchlight red's root cause chain is measured** (instrumented runs, since
   removed): with the cull on, `capsule_tri_contact` skips the courtyard floor plane
   y=41.00 around (4–7, −16..−18.5) whenever Bond's spine is BELOW it
   (`[cull] SKIP n=(0,1,0) nd=-1.0000 f=0.0000`, feet y 40.115) — the game's
   jump-through-from-below arm [M Intersect_CylGeom 0x109da0:302-322; the hatch-9
   measured orientation]. The test harness's Bond stands in the under-croft at
   spawn (`Bond::stand_at` = the under-croft floor 39.015 + 1.1 [M Player_Init
   0x1910f8:156-181 → Player_StandAtNewPosition 0x190d88]) and relied on the OLD
   un-culled push-up through the floor to reach the courtyard before riding the spot;
   that lift is gone, so during the 120 tracking ticks Bond falls under the floor, the
   spot sits on the courtyard above him (y 41.010), `Vec_Dist3D(spot, player) > 2.0`
   holds on ~86 of those ticks, and the game's own lost counter
   [M 0x1cb900:251-263 — dist ≤ 2.0 resets, else +1, `FRAME_RATE·15` gives up] carries
   ~86 pre-accumulated ticks into the teleport: the head returns to the sweep at 815
   instead of the 890–940 window. The port matches the decompile; the HARNESS depends on
   removed behaviour.

7. **The walkreal_01 follow red is characterised**: Bond wedges on nav leg 4 at
   (−33.47, 4.71, 12.00) — 1.17 above the floor 3.54, `grounded=false head_bump=true`,
   never settling, target (−33.11, 2.30, 15.43) 3.45 away (the stairwell end beyond the
   open double doors, `state=0x4 pos01=1.00`). Same class as the searchlight: a
   game-true cull skip removing a below/above-spine push the old walk rode. Not fixed
   this window (below).

## How it was verified (what ran, on the final tree = 9fc3cefa)

- `nf_game` lib: **582 passed, 0 failed, 2 ignored** (the two assertion-free censuses).
- `mp_net_slots_post_move_live`: **2/0** — the 0xbc5e digest pin holds through the lock fix.
- `mp_perf_16_live`: `early_out_segments_match_the_full_search` **ok** (re-derived);
  `sixteen_participants_fixed_run_digest_is_pinned` still FAIL (digest 0xfd4fcbf81586b3f1
  vs pin 9929307665006445228).
- `fix_searchlight_remainder_live`: 2/1 — the two non-tracking tests green, the
  tracking/lost-limit test red (815 ticks; mechanism in §6).
- `fix_npc_doors_live`: 1 FAIL (escort settles at (−4.1, 6.3, 14.2) in PLAYER_IN_WAY 0x1e,
  `hit_door None`, `armed_peer_dist = f32::MAX`, masks mover).
- `walkreal_07000001_live`: 3/1 (follow red; wedge in §7).
- All runs `--test-threads=1` with `NF_ISO` set. No full-suite total line this window
  (below).

## What's left

1. **The full suite did not run** — this window's sandbox spent its first hour on
   setup (clone, the ALSA prefix rebuild, the cold build) and the rest on the bisect
   matrix (each of the four binaries takes 55–344 s per config; 12 configs) plus the
   instrumentation rebuilds. No background runner survives tool-call boundaries here,
   and a fresh 874-binary build (~20 min) plus the run did not fit. The suite must run
   on the final merged tree next window; quote its total line then.
2. **The digest re-pin** (`sixteen_participants_fixed_run_digest_is_pinned`): the brief
   allows a re-pin only with the named mover and its cite. Method for next window: run
   the digest test on master vs this tree, diff the per-tick `net_state_hash` stream to
   the first diverging tick, then name the mover there (a bot's masked LOS verdict flip
   [M 0x1ebaa8:300-319] or a cull-changed capsule push [M 0x109da0:302-322]) and re-pin.
3. **The escort red**: the escort freezes in PLAYER_IN_WAY (0x1e) with
   `armed_peer_dist = f32::MAX` — the sighted-peer scan finds nobody 1.1 units away.
   Next step: trace the `sighted[]` build at drone.rs:16689 upstream to its sight gate,
   and check which of the 0xa27 drop arms flips it on this route (the shoji glass is
   the prime suspect: the mask drops 0x10 by design, so any way-block/peer probe that
   must NOT drop glass is carrying the wrong mask constant — the game's escort opens
   the doors, so the door-adjacent probe is either a different caller mask or not a
   LOS at all).
4. **The follow red + the searchlight harness**: both are the cull interacting with
   sub-floor harness/level spawns. Decisions needed (not window-sized):
   - walkreal_01: instrument the wedge site the same way (the `[cull] SKIP` log) to
     name the skipped plane at (−33.47, 4.71, 12.00); then either the port's walk spine
     differs there (A/B ends per Player_Collision 0x194df0) or the nav walk rides a
     removed push — re-derive the harness leg if the port is right.
   - searchlight: the harness may not depend on the removed push-up. Re-derive: settle
     Bond onto the courtyard the way the game stands him (a [L]-verified spot) before
     the ride loop, keep the ride/track/lost-limit asserts untouched.

## Live checks for the main agent

1. **07000005 courtyard spawn**: fresh boot, read Bond's standing y at the level start
   (EE: player obj+0x30 y) and the courtyard floor plane under the searchlight
   housing (the y=41.00 quads near x 4–7, z −16..−18.5). Expected if the port's
   `stand_at` is right: Bond stands at the under-croft floor + 1.1 ≈ 40.115 until he
   walks out; the game must NOT push him up through the courtyard plane. This settles
   whether the searchlight harness's ride start is game-reachable.
2. **The 07000001 stairwell nav-4 spot**: with Bond walked to (−33.5, 4.7, 12.0) via
   the same route, watch whether the live game's capsule rests on the stair-step there
   (HITTEST+0x20/+0x30 A/B and the grounded flag). If the live Bond stands at floor
   3.54 + small, the port's cull arms skip a plane the game keeps — a port bug to fix;
   if the live Bond also rides high there, the harness nav leg is the wrong shape.

## Usage

not visible to me — this environment exposes no token, cost or context-window figures
for this session (no usage banner, no /cost, no token readout anywhere in the interface
this worker sees).
