# level-1b-zero-g-look — zero-g look that pitches at every heading AND keeps the aim with the view

Branch `s/level-1b-zero-g-look` (from origin/master `babf9247`).

## What was wrong

Two port-side gaps, on top of the composition disagreement that the revert (facea34c, reverting
c3ed2d764) papered over:

1. **The composition.** The game's `Player_ZeroG` 0x192ab0:160-162 is `RotMatrix(BL+0x30, R)`,
   `Concat(obj+0x90, R)`, `Mat_CopyRot(R, obj+0x90)` — the tick's look rotation composed into the
   object matrix. `Concat(a, ab)` 0x1e4a90 writes each output quadword row as a mix of a's rows
   weighted by the b operand's broadcast lanes — in the game's row-major storage (the same
   row-vector transform `v·M` the port already pins: sphere_centre [M 0x133070:33-44], the walk's
   `pos += local·rows` [M 0x193600:284-295]) that is `ab := ab·a`, so the copy-back is
   **obj := R·obj — the BODY frame** [M 0x1e4a90:13-49; Mat_CopyRot 0x1e3370 = copy the rot into
   arg 2]. Master's `m·R` (the world frame) is the wrong reading: facing east/west a look-up
   barely pitches (the pitch lane rotates about the world x axis, which the forward row's z lane
   only).
2. **The camera.** The port's play camera (nf-view lib.rs) was built from Bond's scalar
   `yaw`/`pitch` — the walk state. In zero-g the game zeroes the scalar view pitch BL+0x8a8 every
   tick [M Player_ZeroG 0x192ab0:74] and the view IS the object matrix:
   `Camera_SetToPlayer` 0x1bc048:24-26 builds `MulMatrix0(obj+0x90, RotX(−BL+0x8a8·π/2))` — with
   +0x8a8 = 0 that is obj+0x90 itself. With the scalar arm the zero-g camera froze at the entry
   pitch while the sim's matrix (and the shots) pitched — **this is the play symptom "the aim did
   not follow the camera"**: the view stayed level, the bullets flew where the matrix pointed.
3. **The crosshair offset basis.** The port's crosshair→shot direction mixed the offset lanes
   against a right/up pair rebuilt from the scalars. The game's `Player_GetAimingPoint`
   0x1a4a98:63-83 transforms the offset lanes (ox, oy, −1) through
   `Concat(obj+0x90, RotX(BL+0x8a8·π/2))` — the combined matrix's rows (on foot RotX(−p)·obj_yaw,
   which is what the scalar pair equalled; in zero-g BL+0x8a8 = 0 [M 0x192ab0:74] so the basis is
   the zero-g matrix's own right/up rows — the same rows the camera rides).

## The fix (all [M]-cited in the code)

- `crates/nf-game/src/player.rs`: the zero-g composition is `zero_g_rot := rot·zero_g_rot`
  (body frame), the Concat doc comment corrected (b := b·a in the row storage), and the port's
  `pitch` mirror zeroes every zero-g tick with BL+0x8a8 [M 0x192ab0:74]. The body-frame unit test
  restored from c3ed2d764 (`zero_g_look_pitches_about_the_body_right_at_any_heading`, with its
  world-frame sister).
- `crates/nf-view/src/lib.rs`: the play camera gains the zero-g arm — yaw/pitch off the matrix's
  forward row [M Camera_SetToPlayer 0x1bc048:24-26 + 0x192ab0:74].
- `crates/nf-view/src/scene.rs`: the crosshair offset basis reads the zero-g matrix's right/up
  rows in zero-g [M Player_GetAimingPoint 0x1a4a98:63-83, 0x192ab0:74]; the scalar pair stays for
  the walk (there it is exactly RotX(−p)·obj_yaw's rows).
- `crates/nf-view/tests/walkreal_0700001b_live.rs`: `walk_arrival_to_drakes_defeat` UN-IGNORED —
  it passes end to end (see Runs).

## What pins what

- **Facing west a look-up pitches the view** [M 0x1e4a90 + 0x192ab0:160-162]:
  `player::tests::zero_g_look_pitches_about_the_body_right_at_any_heading` — fwd.y = sin(look·π/2)
  at heading −π/2, the pitch staying in the heading plane; sister asserts the world-frame product
  barely tilts the same view (the pin can fail).
- **The shot lands on the crosshair at several headings and pitches**: the un-ignored
  `walkreal_0700001b_live::walk_arrival_to_drakes_defeat` shoots all eight console screens through
  the REAL fire path from float stations on several headings and screen heights
  (`[1b] consoles latched (console, frame): [(3, 3596), (4, 3704), (5, 4004), (6, 4322), (8, 4545),
  (7, 4856), (1, 5300), (2, 5618)]`), then Drake in the float, to the mission end
  (`to_end 0x0730001b`). On the reverted master the console shots cannot converge facing
  east/west (the pitch error never closes) — that is why the walk was ignored.

## Runs (this sandbox; NF_ISO = the slim on-foot image)

- `nf-game` lib tests (zero-g set): **6 passed** (`cargo test -p nf-game --lib -- zero_g`).
- `walkreal_0700001b_live`: **1 passed** (`walk_arrival_to_drakes_defeat`, 8.11 s).
- `zerog_0700001b_live`: **1 passed**; `census_0700001b`: 0/2 ignored (probe binary);
  `space_laser_1b_live`: **4 passed**; `walk_0700001b_live`: **1 passed**;
  `interactables_0700001b_live`: **7 passed**.
- `mouse_look_rate_live`: **3 passed**.
- Full suite (`tools/test/suite.py`, 1038 test binaries built in 805 s) on this tree: the total
  line lands below when the run completes.

## Known gaps (listed, not silently dropped)

- **Roll.** The game's zero-g view matrix carries the body's roll (a pitched yaw drifts the up —
  what the auto-stabiliser repairs [M 0x192ab0:163-200]); the port's `Camera` has no roll lane, so
  a drifted-up body renders with a level horizon. The aim/crosshair correspondence is not affected
  (the shot basis reads the same rows). [S] port-side gap, cited at the camera arm.
- The zero-g look rates: the game's pad arm is `−action·1.2566371·REC_FRAME_RATE` into BL+0x30/34
  [M 0x192ab0:76-90] — the port keeps the D-002 per-tick deltas (`look` in π/2 units), unchanged
  by this task.

## Live checks for the main agent

1. Fresh boot into 0700001b, face east (+x), hold look-up ~1 s: the horizon should stay level and
   the view pitch up (body right = world x there, so it also matches the old world-frame render).
   Then face north and repeat: the view must pitch up the same amount (the old build barely
   pitched). `--bond` spots: `32,-1.7,0,1.5708,0` east / `32,-1.7,0,0,0` north, feed look up.
2. While pitched up ~30°, yaw the mouse: the crosshair must stay on the same world point the
   camera centres (shot-vs-view check), and after a few seconds of pitched yawing the game's
   horizon would tilt with the drifted up — the port's stays level (the listed roll gap).

## WORKLOG

`docs/WORKLOG.md` no longer exists on master (dropped in batch-1023c, commit 3db74285 — the repo
holds only what the port uses); this report is the task's record, per the current layout. No
WORKLOG file was recreated (shared-file rule).

## Usage

Context window and token/credit figures for this session: not visible to me (no token, context or
cost readout is exposed in this environment; the conversation ran inside the NightfirePort worker
sandbox). The only quota-like number visible is the task clock: sandbox expiry ~07:46 UTC, worker
started 05:48 UTC.
