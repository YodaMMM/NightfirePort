# capsule-cull-and-los-masks-4 — the walkreal_07000008 stall is NAILED (the open door leaf), the mp digest pin method started

## State: the walkreal_07000008_live `walk_the_gondola_to_the_briefcase_and_back` red is
## fully diagnosed with instrumented runs — it is a stance stall against the OPEN secure-room
## door leaf's collision, with a precise one-line fix recipe and the evidence chain below.
## The fix itself is NOT landed (shared collision surface, no window to run the door battery).
## The mp digest pin (item 2) is untouched beyond the method setup. Both reds stay red.

## What's done

1. **Three env-gated additive traces landed** (all `OnceLock`/per-60-tick, trace only, no
   behaviour change — the NF_WALK_TRACE pattern):
   - `NF_BURN_TRACE` (crates/nf-game/src/drone.rs, the lock burn block under `update_player_bullets`):
     per held laser tick, every unburnt lock within 8 units prints the muzzle, the segment end,
     the lock pos/radius and the closest segment→centre distance.
   - `NF_PUSH_TRACE` (crates/nf-game/src/world.rs, `capsule_tri_contact`): every contributing
     face whose push is deeper than 0.05 prints its vertices, mat, normal, distance and push.
   - `NF_DOOR_TRACE` (crates/nf-view/tests/common/walker.rs): the doors within 4 units of Bond
     every 60 ticks (index, state, live sphere).
2. **The red reproduced deterministically** (not thread-flaky on this tree): the test fails at
   :164 with default threads AND `--test-threads=1`, 45.7 s. Round 3's "thread-order flaky"
   reading does not reproduce on the round-4 tree; both modes are red.
3. **The stall is separated from the burn** (the round-3 ask): Bond walks to the fusebox, cuts
   the wires (ch 137 up), and then NEVER reaches "before the safe" — he freezes at
   (154.50, 43.10..43.42, −120.64) for 1600+ ticks. The burn trace shows the muzzle's segments
   end 4.176 from bolt lock0 (r 0.208) — outside the laser's 4.0 reach [L ee_start_05
   weapon_data +0x1C = 4.0] — so ch 134 never writes. It is the WALK, not the burn.
4. **The blocker is named**: `NF_PUSH_TRACE` in the stall window shows ONE face, 120 ticks
   identical: tri (155.003,36.751,−117.997)(155.003,42.747,−122.997)(155.003,42.747,−117.997)
   mat 0x0a n=(1,0,0), dist 0.460, push (−0.082,+0.038,0) — Bond pressed against a vertical
   face at x=155.003 whose top edge (y 42.747) sits inside his capsule axis span [B = root−0.55,
   A = root+0.275; RADIUS 0.55 [M player.rs 0x194df0 slot +0x7cc]]. In that configuration the
   closest capsule-axis point shares the edge's y, so the push is purely horizontal — the
   equilibrium can never lift B above the lip, and the walk velocity cancels against it
   (measured oscillation 154.45 ↔ 154.49 = the push-out + the walk step).
5. **The face is the door leaf, not the wall** — the chain:
   - Flat `sim.world.segment` rays (STATIC set only) at y 42.55..43.00 through the doorway z
     hit x≈157.33-157.37, never 155.0 — the blocker is NOT static geometry.
   - `NF_DOOR_TRACE`: door #10 (the Secure Room door, lock ch 137 [D #122]) opens on the ch 137
     rising edge at t14400 (state 0x2 → 0x4, matching the doc: p3=137 opens on the channel's
     rising edge [M 0x136550-0x136608; D docs/levels/07000008.md]) — the E leg in the test is
     superfluous but harmless — and its live sphere moves (154.49,43.12,−120.70) →
     (155.14,43.12,−121.30), r 1.29, covering Bond's stall point.
   - **The door's keyframes [D placement payload, `Door::keys`]**: frames=2, keys=3 —
     key0 pos (154.514,41.999,−121.329) quat (0,1,0,0); key1 (154.516,…) quat (0,0.7071,0,−0.7071);
     key2 (154.518,…) quat (0,0,0,−1). The position barely moves (0.004); the swing is the
     quaternion: closed = 180°-about-Y pose, open = the mesh's NATURAL pose (identity).
     The leaf's natural pose IS the 5×6 panel at x=155.003, z −118..−123, y 36.75..42.75 —
     exactly the blocking face. In BOTH key poses the leaf lies across the wall/walkway area.
   - The port poses every glist door into the collision set unconditionally (drone.rs
     `set_dynamic` parts: `doors.list.iter().filter(|(_, x)| x.glist != 0)` — "the pose this
     tick's Door_Update writes lands in the same set_dynamic" [M 0x1336f8:238]), so the OPEN
     leaf's mesh blocks the walk.
6. **Why the game does not stall there** [S→M-checkable, the fix's basis]: the game's door
   solidity rides the door's own state machine — the port's already-ported zone rule is
   "obj+0xF4 == CLOSED exactly — OPENING/CLOSING leaves do not block" [M door.rs
   `Doors::update` blocking zone, the game's 0x135738/0x1336f8 zone machinery], and the OPEN
   leaf is likewise not the walk's blocker — the game's player walks through the open doorway
   (the level's own geography requires it: the fusebox→door→safe→briefcase chain). The port's
   unconditional door→set_dynamic membership is the divergence: the game's collide walk reads
   cel-linked objects, and the door object's mesh is not what stops the player at a doorway —
   the state-gated zone is. **The fix: gate the door parts out of the collision set by state
   (`x.state == CLOSED` keeps closed leaves solid; open/closing leaves stop blocking), one
   filter change at drone.rs's doors.extend, with the SS parts (the safe door #205, the bolt
   #217) reviewed the same way.** NOT landed here: it touches every level's door collision and
   the door-family tests (fix_npc_doors_live, the doors family) must run — the window closed
   first. A live check below settles the [M] side (whether the game's open leaf meshes block).
7. **Two port-mechanics cross-checks ran clean** (the stall is NOT these):
   - `NF_WALK_GHOST=1` (drop the push displacement): Bond falls out of the world on 07000008
     (y −489 by t1920) — the push is load-bearing for the floor band; the unguarded through-push
     stays (the known escort_follow divergence note in player.rs stands).
   - The head-on cull/guard math: the lip face's n·d̂ ≈ 0 (a vertical face vs a vertical spine)
     is not culled; the push-tail guard `dot(q−A, B−A) > 0` passes for a lip below the head —
     the game's Intersect_CylGeom 0x109da0 pushes the same contact [M, the port's
     capsule_tri_contact annotations with the disasm addresses; the VU decomp's Dist2Tri call
     with the edge-region axis projection confirms the game tests edges]. The mechanics match;
     the geometry (the open leaf) is the difference.
8. **mp_net_slots_post_move_live `four_human_session_digest_is_pinned` (:285)**: no fix this
   window. The method setup only: the pin's commit a7afd219 is fetched; the hash components are
   the 7-fold `net_component_hashes` (frame_rng/bodies/vitals/combat/match/bots/world,
   crates/nf-view/src/scene.rs :12206) and `NF_NET_HASHDBG=a-b` dumps the per-tick raw state
   (rng x/y/draws, per-slot bond pos/yaw/pitch/substate/crouched, the match slots, hp) with NO
   code change — so the baseline-vs-current diff is a plain log diff on both trees (the
   baseline needs its own CARGO_TARGET_DIR build of nf-view at a7afd219, ~10 min on this
   2-core box, which did not fit). The first divergent frame's `rng`/bond fields name the
   component; round 3's shared-RNG-stream suspect (the cull/masks change guard sight/aim
   timings → draw counts) is still the prime.

## How it was verified

- The instrumented red run: `cargo test --offline --release -p nf-view --test
  walkreal_07000008_live walk_the_gondola_to_the_briefcase_and_back -- --test-threads=1` with
  NF_BURN_TRACE/NF_WALK_TRACE (482 burn lines, 497 walk lines), then NF_PUSH_TRACE (57638
  push lines; the stall window's unique face), then NF_DOOR_TRACE (583 door lines; the d10
  transition at t14400).
- The doorway geometry probe (a scratch test, deleted before commit): down-ray floor grids and
  east-west ray fans at the stance bands, on a FRESH scene (door closed) and after raising
  ch 137 (door open) — the passage map (floor 42.0 both sides, the slot z −121.2..−120.2, the
  opening bottom ≈43.0) and the fresh-vs-open blocker split (static ≈157.36 features vs the
  moving x=155.003 leaf face).
- A jump experiment (scratch test, deleted): forward+jump hops reach apex root 43.42 (+0.32,
  jump vel 3.76 [M the port's Player_HandleJump arms]) — below the +0.75..1.0 the lips need;
  the doorway is not traversable by the port's jump either, consistent with the leaf being the
  blocker (not a step-height problem).
- Build: `cargo test --offline --release -p nf-view --test walkreal_07000008_live --no-run`
  and the probe builds, release, on the merged tree (base ca2e7389 + origin/master at 4a155ddf).
- **The full suite did NOT fit this window** (the background runner died at the tool-call
  boundary again — the fourth sandbox to measure it; the foreground budget went to the
  diagnosis). The touched crates' lib/test compile is clean (the trace builds above). The next
  window must run: the full four-crate suite, the walkreal battery, and the door family after
  the item-6 fix.

## What's left

1. **Land the item-6 fix** (the door-state gate on set_dynamic's door parts) and run the full
   battery: walkreal_07000008_live should go 4/4 (the stall gone → Bond reaches "before the
   safe" → the bolts burn within the 4.0 reach — the burn side was never broken), the whole
   walkreal family, fix_npc_doors_live, the door/SS tests, then the full suite.
2. **The mp digest pin** per the item-8 method (baseline build at a7afd219, NF_NET_HASHDBG log
   diff, first divergent component+tick, fix or conscious re-pin).
3. The full-suite total line on the final tree.

## Live checks for the main agent

1. **Does the game's OPEN door leaf block the player?** [settles the item-6 fix's basis]
   Fresh boot 07000008, warp Bond just outside the Secure Room door (≈154.0, 43.1, −120.7),
   shoot the fusebox wires (or set ch 137), wait for the door's state 4, then walk east into
   the doorway. Expected per the fix: Bond walks through the swung leaf's panel plane
   (x≈155.0) unobstructed; the panel visibly lies across the walkway (the leaf's natural pose
   [D keys]). If the game instead stops Bond at x≈154.45, the leaf IS solid when open and the
   fix needs the game's real path (the leaf's swing must clear the passage in the game's
   mesh/pose — re-check the glist 0x8033004c mesh vs the port's shapes entry).
2. **The doorway's static slot** [completes the passage map]: the same boot, place Bond INSIDE
   the room (≈158.5, 43.1, −119.5) and walk west through the slot — does the game pass the
   ≈157.36 static lip and the ≈155.0 plane with the door open? Read obj+0x30 at the slot
   centre (156.2, −120.7): expected ≈43.1 if the passage floor is the walkway's 42.0 + 1.1.

## Usage

not visible to me — this session's driver surface shows no token, cost or context-window
figures (the IM gateway metadata carries only session/chat/trace ids).
