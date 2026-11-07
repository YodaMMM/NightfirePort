# homing-copter-2

Branch `s/homing-copter-2` from `origin/s/port-map-bullets` (70d0fbf04). Task: finish the
`Bullet_heatseekable` 0x12f248 class-0 arm — the dormant-copter gate, the un-ignored live
harness, and the `Bullet_homing` 0x12f358 audit past the target pick.

## Done

### 1. The `+0xFC` gate on the copter candidates (the task's gap 1)

The game's class-0 arm requires `obj+0xfc != 0` before the `CopterList` walk
[M Bullet_heatseekable 0x12f248:29-40]. The counter's full writer set, traced:

- `control_init_object` 0x134630:16 sets `+0xfc = 2` at the obj's creation [M];
- `control_movement_object_handler` 0x1336f8:242-246 decrements it once per tick over the
  whole object list [M];
- `View_AddObjects` 0x1e62d8:71 re-sets it to 2 every drawn frame (and
  `View_AddForcedObjects` 0x1e5fb8:174 for forced objects) [M];
- the copter's SP is `SP_HideObj`'d at create while its gate channel is off
  [M Copter_Create 0x17d340:73-76] and unhidden on the gate's first ON tick
  [M Copter_Update 0x17dbb8:91-97]; `Script_HideObj` 0x1db808:27-32 sets obj+0xF0 bit 0x10
  per part — the hidden body is never view-added, so its `+0xfc` decays to 0 within three
  ticks and stays 0 until the gate opens. `SP_HideObj` itself never writes `+0xfc`.

Port changes (commit a57375669):

- `Copter::body_shown()` (copter.rs) — the port's shown/hidden gate (`gate == 0`; the port
  clears `self.gate` on the fire tick), with the full trace in the doc comment. The port
  has no per-frame SP view cull, so shown/hidden is the whole gate — documented.
- The homing acquisition loop (drone.rs) skips copters with `!body_shown()`.
- The homing re-validation drops a copter target on `c.landed` — the port's obj+0xFE bit 0
  (state 5 arrival / state 6 death set it [M 0x17dbb8:274]; the retain that removes the
  copter runs later in the tick). This ports `Bullet_homing`'s own re-validation
  `(target+0xfe) & 1` [M 0x12f358:107-110], which the branch read only as list membership.

### 2. The (0,0,0) body read (found under gap 1's symptom)

`copter_body_pos` read `matrix[0..2][3]` — column 3 of rows 0-2, the projection row, which
is (0,0,0) for every rigid matrix. The port's Mat4 is row-major with the translation in row
3 (`view_rot_trans_matrix` writes `[trans, 1]` there [M placement.rs:220]; `Copter::pos()`
reads it). Every copter lock — shown or dormant — steered at the world origin. Fixed to
`c.pos()` (row 3), both in the acquisition and in the steer site. This is also why the old
harness concluded "the copter never poses": its pose check summed the same wrong elements.

### 3. The live harness on the level's own path (the task's gap 2 — mostly done)

`homing_copter_07000012_live.rs` rewritten and un-ignored:

- **The equip goes through the level's own grant path**: the placed 0x2e pickup
  (#2aa/#2f0 [D the placement sweep]) is taken through the real `Pickup_Update` take test
  (the teleport is the documented stand-in transit; the take itself is
  `Pickup_Handler` 0x18e650 → `Player_EquipWeapon` with the loaded-model gate), verified by
  `inventory.owned[base(0x2e)]`, then the real switch request draws it. This WORKS — the
  green run logs "scorpion taken through the level's pickup path" and the draw completes.
  The old give-path refusal is not on this path.
- The gate channel is raised (`channels.set(37, 1, f)`) and `body_shown()` must read true —
  the body then flies its route (verified: the pose tracks the spline keys).
- The dormant half is a second, GREEN test: with the gate never raised, Bond equips through
  the pickup, stands in the open pocket by the pickup aimed straight at the world origin —
  the first cut's candidate position — and fires ~100 homing rounds; no round may carry a
  copter target [M 0x12f248:29]. Green: 99 rounds, none locked, the gate stayed off.

## What's left (one slice, precisely diagnosed)

The lock test (still `#[ignore]`d, new reason) fires real rounds at the flying body from a
standable, bullet-LOS-verified shelf and asserts lock + convergence as written — but the
lock never engages, because **the arm's own LOS self-blocks on the port's dynamic set**:

- The game's `Collide_LineOfSight` carries the ray's SUBJECT objects
  (`Collide_RayIntersect` args 4/5 — `Bullet_homing` 0x12f358:92-93 passes the bullet obj
  and the CANDIDATE obj; the copter's own spot check passes the copter and Bond [M
  0x17dbb8:171-172]), and the ray cannot report the candidate's own collision nodes — a ray
  whose endpoint sits inside the target's own glist would never return clear, so no heat-seek
  lock could ever exist, for guards or copters.
- The port's `segment_bullet_clear` has no exclusion, and the copter's script entities pose
  into `set_dynamic` (drone.rs `script_parts`; the fusebox precedent) — measured: after the
  opening ticks the segment from the (20, 4.4, 15) shelf to the body's rest pose hits at
  t = 0.943, at (61.9, 4.38, 19.3), 2.5 units off the body (64.44, 4.30, 19.53) — the body
  cluster's own mesh (fresh-load LOS to the same point is clear, so this is registered
  script-entity collision, not static geometry).
- The fix (next round, ~40 lines): an exclude-by-shape-hash bullet-clear in world.rs (the
  dynamic set's `last_objects` carry each object's hash, index-aligned with `geo.objects`),
  fed the candidate copter's SP stream hashes (the `hidden_stream_objects` precedent in
  nis.rs), called from the homing's LOS for copter candidates. Then the test runs
  red-then-green exactly as written (everything up to the fire loop already runs green).

## The `Bullet_homing` 0x12f358 audit past the target pick

- **Re-validation** [M 0x12f358:107-110]: the target drops when `target+0xfe & 1` is set,
  then the rescan runs the same tick. The port's drone arm re-validates through `seekable`
  (drawn / health > 0 / state ≠ WAIT_SWITCH) — a stricter per-class stand-in documented on
  the branch; the copter arm now drops on `landed` (the same bit). The ≤1-tick skew (the
  port's bullet pass runs before the copter walk in the tick; the game's order is
  list-order within the control pass) is the documented drone-arm pattern.
- **Steering** [M 0x12f358:116-167]: the port matches the game's math term for term —
  `dir += clamp((to−dir)·0.0333, ±0.01)` per component, normalised. The trailing
  `Mat_Align2Dir(obj+0x90, dir, 0x2d6210, 0x2d6220)` [M 0x1df2a8] is an orthonormal
  facing-matrix build from the already-normalised dir with two up-hint vectors (the two
  constants are 16 bytes apart — vector constants, not turn-rate limits); the port's
  facing IS its dir, so nothing further to port.
- **The steer skip** [M 0x12f358:116-118]: `Bullet_homing` returns when the BULLET obj's
  +0xf6 short is non-zero. That state is the smoking-missile state `Bullet_DoTrails`
  0x12d890:399-407 sets (for a `weap+0x6c & 0x20000` bullet reaching its hit event: obj+0xf4
  = 1, obj+0xf6 = 1, the smoke puff + whoosh — the bullet keeps flying and stops homing);
  `Bullet_update` 0x12e7b0:473-475 also skips the homing call entirely while obj+0xf4 == 3.
  The port has no smoking-flight slice (port-map-bullets' documented remaining work), so
  there is no state to gate on — nothing to change here yet; the addresses above are the
  warrant for that slice.
- The acquisition scan's ordering (the port's two-list fold, ties to the earlier candidate)
  stands as documented on the branch.

## How this was verified

- `a_gated_copter_body_is_never_a_homing_candidate_on_07000012` (new, green): the level's
  own pickup grants the 0x2e, the gate stays off, ~99 real rounds fire and none ever carries
  a copter target. Red on the branch's old code: the old scan accepted the copter
  unconditionally at position (0,0,0) — the origin is open ground by the pickup, so the old
  code's LOS+cone+dist all pass and the lock would set `target_copter`.
- `the_scorpions_homing_round_locks_the_copter_body_on_07000012` (un-ignored, `#[ignore]`d
  on the measured self-block above — the old wrong reason is gone): the pickup take, the
  draw, the gate raise and the body's flight all run green up to the fire loop.
- Terrain facts behind the harness (probe, since deleted): the copter's rest pose is route
  key 0 (64.44, 4.30, 19.53); the patrol loop (keys 0-250) is bullet-LOS-clear from the
  (20, 4.4, 15) shelf at 42-79 units; the loop's later stretch is sealed from every
  standable floor.
- Test binaries run whole after the change: `nf-game --lib` 686 passed / 0 failed;
  `scorpion_modes_live` 6/0; `fix_weapon_modes_live` 1/0; `oddjob_hat_live` 2/0;
  `homing_copter_07000012_live` 1 passed (the dormant test), 1 ignored.
- The full four-crate suite was started in the background (tools/test/suite.py); see the
  suite result line below — it did not fit in the session's remaining window.

## Found on the way (not this task's slice)

`ss.rs:209` reads the sweep objects' sound position as
`[s.matrix[0][3], s.matrix[1][3], s.matrix[2][3]]` — column 3 of rows 0-2, which is (0,0,0)
for the door/spline matrices (translation in row 3 [L door.rs:105 `quat_matrix`]). Same bug
class as the homing body read; the sweep sounds play at the world origin. Needs its own red
test through the real path (a sweep sound test), so it is reported, not patched here.

## Live checks for the main agent

Optional, on a real PCSX2 boot of 07000012 with the Scorpion from pickup #2aa:

1. Watch the body obj's `+0xfc` (address = the body obj from `Copter_Update`'s
   `*(rec+0x48)`, read at `Copter_Update` 0x17dbb8's head) before the gate fires: expect 0
   (decayed from the init 2 within ~3 ticks). After the gate's first ON tick
   (`SP_HideObj(…,0)` at :95): expect 2, refreshing each drawn frame.
2. Fire a homing round at the flying body and watch `Bullet_homing` 0x12f358's
   `Collide_LineOfSight` (0x1edf50) return value with the body in the cone: expect 0 (clear)
   with the body's own mesh in the ray's path — confirming the game excludes the candidate
   obj from its own LOS (the port's missing half).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for
the session (no input/output token counts, no cost display, no context meter), so there is
nothing to copy.
