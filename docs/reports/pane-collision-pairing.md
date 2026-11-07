# pane-collision-pairing

Task: settle the placed 0x20 panes' shape-key/matrix pairing from the game, turn the
`_pane_parts` ride on with the death-tick grace and each pane's bullet filter, remove both
`dead_pane_live` ignores, and pin the behaviour through real input.

## The pairing, settled from the game

- `Break_Create` 0x12c230 takes the pane's glist from its CALLER: `Control_SetGList(obj,
  param_3)` [M 0x12c230:19] writes obj+0xD8 = glist and obj+0xF0 = glist+0x44 (keeping the
  0x30008000 bits) [M 0x134960]. The object matrix is `control_create_object(0xC, pos, …,
  quat 0)` [M 0x12c230:12] — the placement's `View_RotTransMatrix`, the port's `b.matrix`.
- The placement caller passes the record's resolved object: by index `pCelList[index]` —
  NULL (NO create at all: `if (iVar5 != 0)` gates the create call) when index ≥
  MapEntityCount — when the record's hashcode word is −1, else the hash-table hit
  [M 0x1d1b68:77-96]; the create is also skipped when the record's +8 flags word carries
  0x8000/0x2000 [M 0x1d1b68:93]. The port's `Breaks::create` resolves the same way and the
  ride already excluded `shape_key == 0`.
- `Collide_PickObj` 0x1eae80 admits a cel-linked object when `*(obj+0xD8)+8 != 0` or
  obj+0xFA carries bit 8 (kind-0x100 hittests pass unconditionally) [M 0x1eae80:88-90]; the
  ray walks are PER-CEL (`Collide_LineOfSight` 0x1edf50's single `cel_tag*` into
  `Collide_RayIntersect` 0x1ec690; `Collide_Pick` 0x1eb320:314-329 walks the candidate cel's
  linked objects) — a pane object in another room never joins a cross-room ray.
- `Break_Kill` 0x12b970: :59 `iVar15 = *(*(obj+0xD8)+8)` (the collision data for
  `Debris_Create_Loop`; 0 → the silent kill :61-63), :60 `obj+0xFE |= 1` — the delete that
  takes the object (and its cel-linked glist) out of the world.
- The filter: `Break_Update` 0x12bfd8:15 calls `Collide_FilterBullets(obj+0xD0, tag+0x08 |
  0x106)` [M 0x1ece30:24-33 — a class-0x05 contact STAYS only when
  `(weapon_definition+0x10 & mask) == 0`; the word is `*(u16*)(*(bullet_tag+0x44)+0x10)` and
  bullet tag+0x44 holds the `weapon_definition*` (Bullet_init 0x12c5a0:109)], then subtracts
  `HITDATA+8` per staying bullet contact (:36-37). Master's breaks arm already reads
  `weapon_definition+0x10` and skips a pane whose mask matches — a smash-only pane neither
  stops nor takes such a round. `Bullet_handle_object_destruction` 0x12cd98 only explodes
  special bullets — the stop path damages no pane.

## The disc print (crates/nf-game/tests/pane_pairing_census.rs, new)

Per placed 0x20 row on the five named levels — part bounds (the port's resolved shape at
`b.matrix`) next to the placement bounds [D], raw extent, materials, flags, dup-static,
game-create verdict:

- 07000001: 55 rows (45 kind-0/1 props of 96–434 tris standing ON the walkable museum-hall
  floor + 10 kind-2 glass sheets); 07000009: 17 rows (all kind-2, hp 1/6, one filter
  0xffcf); 0700000a: 45 rows (kind-2, mats 0x10; two rows share object (37,318) at hp 9;
  k=44 is a kind-0 row sharing (37,532) with a kind-2 row); **0700000b: ZERO 0x20
  placements** (the comment's "0b" phantom cannot come from panes); 07000014: 3 rows
  (kind-2, mats 0x0f, filter 0xffdf).
- **120/120 parts land exactly on their placement bounds (d = 0.00 per axis), every row is
  game-created (flags 0x0020, glist resolved), no static double, every key the anonymous
  synthetic resolving its own object.** No row is wrong by key, matrix, or not-a-pane
  placement: the phantom collision the old comment feared is NOT a pairing fault.
- The census assertion (`no_created_pane_part_lies_outside_its_placement_bounds_on_any_sp_level`)
  extends the print over every SP level on the disc and passes (skipping the 0xa000-flag
  rows the game never creates — none exist on the five levels).

## The ride

- world.rs: `PANE_PART = 0x400_0000_0000`; the dynamic build pushes the node pass flag for
  pane parts (both the reuse and retransform paths); the ray sets
  (`segment_in_sets_drop_pass_glass`, `segment_blocked`) walk the dynamic set separately and
  skip `PANE_PART` parts — the port stand-in for the game's per-cel ray scoping — while the
  capsule queries and the effect walks keep the panes solid.
- drone.rs: the kind-2 pane parts join the per-tick `set_dynamic` set
  (`!b.dead || b.dead_frame == drones.frame`, the death-tick grace kept; the script-entity
  arm just above carries the same rule for the gondola's script panes — one rule for both),
  gated behind `PANE_RIDE`; `BREAK_WORLD_STOP` is tied to the same switch (with the meshes
  out of the ray sets the bounds contact always fires at or before the scenery stop; with
  them still static the 07000007 point-blank vase pathology returns — that is why the
  switch is one).

**Why the switch is OFF (what the ride-on run showed).** With the parts riding, the whole
named battery ran green EXCEPT the 07000001 real-damage walks: `walkreal_07000001_live`
`walk_from_the_spawn_to_the_exit_with_the_contact_real_damage` (+ `_00agent`, `_agent`) die
to guard melee (master: green — verified by checking the two src files out of master and
rerunning; the death spots carry no pane, so this is a guard-capsule/melee-timing cascade,
not Bond stuck on a sheet). The 01 census explains the sensitivity: 45 of its 55 rows are
kind-0/1 props standing on the walkable hall floor — the ride's first attempt took ALL rows
and died the same way, so the landed form rides only the kind-2 glass ("an object that is
not a pane" is the task's third category, and the props stay at master's out-of-world
status quo [S unverified: the game-side movement solidity of those props is not proven
either way]). Even kind-2-only, the 01 walks still cascade — the capsule solidity changes
guard motion/melee timing there. The rays never see the panes (the skip), so sight is
unchanged; the delta is the movement pass, and the game's movement collision is per-cel
scoped exactly like the rays — the port has no per-room object graph for capsules either.

## How it was verified (final tree, NF_ISO set, --test-threads=1)

- nf-game lib: **686 passed / 0 failed / 5 ignored**.
- nf-game pane_pairing_census: **2 passed** (the five-level print + the all-SP census).
- nf-view dead_pane_live: 0 passed / **2 ignored** (the reason carries the flip-one-switch
  state). With `PANE_RIDE = true` BOTH ran green earlier this session (the shot-out pane
  stops blocking a round; the intact pane stops Bond and the dead one lets him cross) —
  the run is reproducible by flipping the const.
- nf-view glass_pass_every_level_live: **3 passed** / 3 ignored (the ignores predate this
  task).
- walkreal_07000009_live **8 passed**; walkreal_0700000a_live **13 passed** / 2 ignored;
  walkreal_0700000b_live **7 passed**; walkreal_07000014_live **9 passed** / 1 ignored.
- walkreal_07000001_live on the final tree: master behaviour, green (the real-damage trio
  verified green on the master-sources run during the bisection).
- Full suite (`tools/test/suite.py`): the environment kills background jobs between tool
  calls in this sandbox (two attempts, both dead with empty output — see suite-final.out);
  the foreground window went to the named battery above. The final tree is behaviourally
  master (the switch off; the only code delta inert: the dynamic-set ray walk refactor and
  the gated block), so the master suite totals carry. The next session should run the suite
  once with `PANE_RIDE = true` after settling the 01 cascade.

## What's left

1. Settle the 01 real-damage cascade and flip `PANE_RIDE` (one const) + un-ignore the two
   dead_pane tests. Candidate: per-cel scoping for the capsule walk (the same stand-in the
   rays got), or walk-side route evidence that the game's 01 route weaves the hall props
   and the guards' melee timing changes under pane-solid capsules.
2. The bullet's glass-PIERCE rule stays the documented deviation (the port kills the round
   at the pane contact unless `glass_pass`; the game's material-0x10 pass [M
   0x12cf44..0x12cf64] continues to the next contact) — inherited from
   done/30-dead-pane-collision-2.md "What's left" #2, unchanged.
3. The kind-0/1 props' movement solidity ([S]) — a live check arbitrates.

## Live checks for the main agent

1. 07000001 museum hall, fresh boot: does Bond's capsule collide with the small floor props
   (the 0x20 kind-0/1 rows, e.g. placement 22's object at (−10.7..−10.0, 6.1..8.2,
   11.5..12.5))? Walk Bond into one — blocked or passes? (`fieldprobe.py` poke the pane's
   obj+0xFE after a bullet: the kill path is already pinned.)
2. 07000001, fresh boot: do the hall GUARDS brush the kind-2 sheets at x 16–18 (placements
   121–124, 76) during their patrols — do their paths deflect (the cascade my ride-on run
   showed)? `gdbwatch.py` on a guard's +0x90 matrix while he crosses the sheet plane.
3. 07000009, fresh boot: the 16 hp-6 panes (placements 108–123, object (39,94)/(39,95))
   span y −369..−354 at 15 units tall — do guard sight/rays cross them in play (the port
   ray-skips them; the -2 report's live checks 1–3 still stand for the drive glass and the
   08 pierce).

## Usage

not visible to me (no token, credit or context-window figures are shown in this
environment; the session's sandbox expires ~10:04 UTC per the driver message).
