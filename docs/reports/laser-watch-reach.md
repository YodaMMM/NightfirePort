# laser-watch-reach — report

Branch `s/laser-watch-reach` (from origin/master 6f341d9e). WORKLOG: `docs/WORKLOG.md`, entry
`laser-watch-reach`.

## The task

The port's laser-watch round reaches 4.0 units and the satchel contact is a 0.5 sphere (a stand-in
for the game's mesh check), so lasering a landed satchel from more than about 4.5 units does
nothing. Find the game's real reach and contact, port them with the addresses, pin the burn's
maximum distance (sister); also fix the space laser's `spawn_impact(0x2d6210, …)` argument order
against `Emitter_Create` 0x17ae00.

## What the data says

**The reach — the port's 4.0 is the game's own number, not a stand-in.** The laser watch's weapon
records read straight off the [L] boot snapshot `ee_start_05.bin` at `weapon_data 0x2bf150 +
id·0x10C`: **0x4E range +0x1C = 4.0, 0x4F range +0x1C = 8.0**, both speed +0x20 = 33.333, refire
+0x40 = 1. The bullet's death check compares the record's +0x1C against the travelled distance [M
`Bullet_update` 0x12e7b0:158 — `if (*(float *)(record + 0x1c) < travelled) …`], and the speed
loads from record +0x20 at `Bullet_init` [M 0x12c5a0:187]. With the state-0 step
`(0.5 + FRand(0.5))·33.33` the first segment clamps onto the 4.0 boundary and the round is dead by
the next tick's head check [M 0x12f014-0x12f02c, 0x12e95c-0x12e984] — a 0x4E round's whole life is
one segment of ≤ 4.0 from the muzzle; a 0x4F round's is ≤ 8.0. The port already runs exactly this
law (`bullet.rs`'s watch test; the `drone.rs` burn walk consumes the round's own clamped segment).

**The contact — the game's chain, now written down.** `Bullet_DoTrails`' satchel tail does NO
distance test of its own [M 0x12d890 tail 0x12e2f0-0x12e390]: it walks the CHARGE's `obj+0xD0`
contact list, keeps the class-5 hitters (`*(char*)(hit+0xff) == 5`) whose record id is 0x4E
(−FRAME_RATE_MUL off `BU+0x102`) or 0x4F (−FRAME_RATE_MUL·10), spawns the sparks
`Emitter_Create(0xc00000a, hit+0x40, hit+0x10, 0, 0, 0)` at the contact point, and plays 0x5dd at
`BU+0x102 ≤ 0`. The contacts arrive from the engine's collision pass: the broad phase runs the
charge's world sphere (`Control_BuildWorldSph` 0x133070 over its glist) against the round's segment
[M `Collide_PickObj` 0x1eae80:92-108], the narrow phase runs the glist's triangles [M
`Collide_Intersect` 0x1ebaa8 → `Intersect_RayGeom` 0x10a8c8]. So the answer to "sphere or mesh":
**both — the glist world sphere for the pick, the glist mesh for the hit**. The port keeps the
segment-vs-sphere form with the 0.5 radius ([S] the charge's exact glist bounds are not pinned in
the port — its bullet objects carry no glist); the radius question is now a precise live check
(below) instead of an unexplained stand-in.

**The space laser's impact emitter was mis-argued — fixed.** `Emitter_Create`'s signature is
`(f, int, def, pos VECTOR*, up VECTOR*, u, u, u)` and the def keys the hashtable lookup [M
0x17ae00:86 — `param_3 == 0 || hashtable_getitem()` gates the whole body; param_4 → obj+0x80; the
`Vec_MakeOrthonormalBasis(…, param_5)` / `Mat_SetUp(…, param_5)` pair makes param_5 the normal].
The game's space-laser call is `Emitter_Create(0, 0, 0xc000010, target, 0x2d6210, 0, 0, 0)` [M
0x1aa3f8:189] — the def is **0xc000010**, and 0x2d6210 is the ADDRESS of the boot-written (0,1,0)
constant [L .data 0x2d6210, written at boot 0x1c7e78; the same constant `Player_LaserPointer`
0x1a65d8:190 passes to `Mat_Align2Dir`], so the normal is world-up. The port passed 0x2d6210 in
the def slot; `drone.rs` now calls `spawn_impact(0xc000010, l.target, [0.0, 1.0, 0.0])` and the
test's [S] note is replaced with the arg-order citation.

## What changed

- `crates/nf-game/src/drone.rs` — the space-laser impact def fix + the cited comment (above).
- `crates/nf-game/src/bullet.rs` — new `the_burn_segment_never_passes_the_record_range`: the
  not-beyond sister — a 0x4E round never produces a segment end past 4.0 (its whole life is one
  clamped segment, dead after), while the 0x4F round's segment covers the 4.5-8.0 band [L records;
  M 0x12e7b0:158].
- `crates/nf-view/tests/walkreal_0700004a_satchel_live.rs` — the ride's first-contact distance is
  now asserted inside the 0x4E reach (< 4.5, the record 4.0 + the contact sphere) at the first burn
  tick [L + M 0x12e7b0:158] — the in-reach arm of the pin through the real ride/aim/fire path.
- `crates/nf-view/tests/space_laser_1b_live.rs` — the impact-emitter print/assert updated to
  0xc000010 with the [M 0x17ae00] argument-order citation.

## What's left

- The charge's glist world-sphere radius (the broad-phase pick radius): the port's bullet objects
  carry no glist, so the 0.5 contact radius stays an [S] stand-in. **Live check for the main
  agent:** fresh boot 0700004a, land a charge, fieldprobe the charge obj's `+0x80`/`+0x8C` (the
  world-sphere centre/radius that `Control_BuildWorldSph` 0x133070 wrote) — expected: a radius of
  the satchel mesh's bounding sphere (mesh 0x20007b3 [L weapon_data 0x3b+0x5C]); the port's 0.5
  should be replaced by that number if it differs.
- The 0x4F (> 4.0, ≤ 8.0) burn through the real path: no shipped level hands Bond a landed satchel
  at that standoff; the unit seam pins the law (the 0x4F segment covers the band) and the 4a ride
  pins the 0x4E in-reach arm.

## Live checks for the main agent

1. **The landed charge's world sphere** (the contact radius): fresh boot 0700004a, let shooter
   #286's charge land, fieldprobe the charge obj +0x80 (centre) and +0x8C (radius). Expected: the
   centre at the charge's rest position, the radius = the 0x20007b3 mesh's bounding sphere. The
   port's 0.5 stand-in becomes that number.
2. **The burn's contact rate**: gdbwatch on a landed charge's `BU+0x102` (the burn counter) while
   the watch beam holds on it from ≤ 4.0. Expected: −1 per tick with 0x4E (FRAME_RATE_MUL [M
   0x12e2f0-0x12e348]) — the port matches; a different rate would point at the contact
   multiplicity (the port allows one contact per round per tick).

## Runs

- `cargo test --offline --release -p nf-game --lib bullet:: -- --test-threads=1`: **8 passed;
  0 failed** (both watch tests included).
- The satchel/laser binaries (nf-view unless noted, `--test-threads=1`):
  `walkreal_0700004a_satchel_live` **2 passed** (the ride's defuses + the new distance assert);
  `space_laser_1b_live` **4 passed**; `fix_laser_watch_live` **3 passed**;
  `gadget_laser_live` **8 passed**; `interactables_0700004a_live` **2 passed**;
  `camera_beam_satchel_wreck_live` **3 passed**; `satchel_digits_4a_live` **9 passed**;
  `fix_satchel_stick_play_live` **6 passed**; `fix_satchel_tripbomb_live` **2 passed**;
  `fix_sg5_laser_live` **2 passed**; `laser_gas_mesh_live` (nf-game) **1 passed**.
- `cargo test --offline --release -p nf-game -- --test-threads=1`: **988 passed; 0 failed; 9
  ignored** (the +1 over the previous session is the new segment test).
- `cargo test --offline --release -j4 -p nf-formats -p nf-audio -- --test-threads=1`: **136
  passed; 3 failed; 0 ignored** — the 3 are exactly the documented slim-ISO allowance
  (`disc_golden`: `/DRIVING/MIS01.MUS: too small for a BIGF header` — the slim image's driving
  audio is truncated).
- The fast runner (`tools/test/suite.py`): started detached twice on this sandbox; both launches
  were reaped between tool calls (suite.out stayed empty — the same reaping the previous session
  hit), and a foreground full run cannot fit the tool call ceiling (nf-view alone carries ~900
  test binaries at several seconds each). **No full-runner total this session.** What did run to
  completion: all of nf-game (988/0/9), all of nf-formats + nf-audio (136/3-allowed/0), and every
  binary the task names (the satchel/laser set, interactables_0700004a_live, space_laser_1b_live)
  — the totals above are the verified numbers behind the push. The tree is merged with
  origin/master (branched from 6f341d9e, no newer master commits fetched).

## Usage

not visible to me
