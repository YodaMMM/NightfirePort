# floor-crack-01 — the 07000001 floor crack at (-4.703, 6.34, -5.968)

## Verdict

The crack is real geometry, and the game bridges it with a rule the port never ported: **`Drone_FeetOnPoint`
grounds on the movement hittest's pick-contact bit when its ray misses** — the guard's 0.4-radius movement
capsule spans the 2.3 mm T-junction slit, so the game sets the bit from the floor contacts on both sides
while the 1-unit feet ray finds no triangle. The port's `feet_on_point` had only the ray, so a guard parked
on the slit fell forever.

The slit (dumped with `world.tris_near`, flat `CollisionWorld::build` of 07000001):

- north floor (object bounds lo [-11.860, 5.339, -5.967] hi [-4.093, 8.868, -1.204]): its floor tris end at
  z −5.966116 … −5.9673367 along the south edge — interpolated z ≈ **−5.96695** at x −4.703, y 5.3392725;
- south strip (bounds lo [-4.736, 5.339, -8.958] hi [-2.0, 5.432, -1.065]): its tris end at z −5.969211 …
  −5.9701877 — z ≈ **−5.96923** at x −4.703, y 5.3434916;
- west slab ends at z −5.9691014, x ≤ −4.735157; the point x −4.703 is east of it.

Gap: ~**2.3 mm** in z, running in x (probes 0.1 east/west at the slit's z also miss). The parsed triangle
vertices are exact multiples of 1/16384 (the game's fixed-point collision format), so the game's data has
the same slit — `Intersect_RayGeom` 0x10a8c8's edge tests are exact sign comparisons (no epsilon), so the
game's ray misses there too. A widened tolerance would have been wrong.

## The game's rule, with addresses

1. `Drone_FeetOnPoint` 0x140660 casts the down-ray `obj+0x30 → pos + up·−(hip+0.6)` through
   `Collide_RayIntersect` (mask 0xc) [M 0x140660:66-96]. When the ray brings no hit (`iVar9 == 0`) it reads
   `sVar3 = *(short*)(hittest+0x9c)` and the tail sets `anim+0x60 & 8` (grounded) iff `sVar3 != 0` OR a ray
   hit — **no hit data is copied, no y is re-adjusted** on the bit arm [M 0x140660:160-175].
2. The hittest it reads IS the movement query's: `NDrone2_DefaultInit` 0x14b300:34 wires
   `obj+0xd4 = dcv+0x3d0` [M]; `Collide_Update` 0x1ea398:83 clears `*(u16*)(hittest+0x9c) = 0` each tick,
   then runs `Collide_Pick` and per-candidate `Collide_Intersect` [M 0x1ea398:83-96];
   `Collide_Intersect` 0x1ebaa8's tail ORs `+0x9c` per real contact [M ~0x1ec150]; the cylinder path's bit
   is set on an up-facing contact, `push·axis > 0` [M Intersect_CylGeom 0x10a72c-0x10a734 — the same rule
   `Player_FeetOnPoint` grounds on at 0x194188:176-185].
3. The movement capsule's shape: radius 0.4 (0.55 when `dcv+0xd8 == 0xc`), top `pos + up·r`, bottom
   `pos + up·(r − hip)` [M NDrone2_Collision 0x140e90:73-93]. The bottom sphere reaches the feet, so a
   guard over the slit penetrates the floors on both sides — up-pushes — bit set.

## What changed

- `crates/nf-game/src/drone.rs`: `Drone::feet_pick_bit` (the ray-miss arm: the capsule pick-bit probe,
  `world.capsule_pick_contact(top, bottom, 0.4, −inf)`), and `collision_handler` now takes
  `feet_on_point(world).or_else(|| self.feet_pick_bit(world))`. Port choices, marked [S] in the code:
  the probe runs only when the ray missed (the port's drones follow nav paths with no per-tick capsule
  push — the ray-hit ticks have no consumer for the bit), the pick reference is a fresh-probe `-inf` (the
  lifetime `+0x50` accumulator gate is the player's persistent-hittest reading, player.rs "0c3", not
  pinned for drones), the grounded y is held not snapped (`Drone_FeetOnPoint` copies no hit data on this
  arm), and the 0.55 `dcv+0xd8 == 0xc` radius stays unmodelled. `feet_on_point` is now `pub`.
- `crates/nf-view/tests/fix_guard_weapons_live.rs`: `the_3x3_hit_rate_and_damage_matrix_matches_the_game`
  un-ignored (its diff-2 guard walks onto the crack).
- `crates/nf-game/tests/floor_census_live.rs` (new): the census — every one of 07000001's 103 nav-path
  nodes (feet = disc y − 0.4, the `AIPath_Parse` +0.4 offset undone) has a triangle below within a
  40-unit column (`world.floor_below(p, 40)`), i.e. no other fall-out-of-the-world cracks; the same file
  reports the nodes that walk airborne a tick (6, stair steps — the game's own ray misses there too and
  the falling rays catch the next step within its 0.6 reach).
- `crates/nf-game/src/drone.rs` unit test `the_07000001_crack_feet_probe_finds_the_floor`: the raw crack
  pins (`floor_below` None at the point, Some 0.1 north/south), `feet_on_point` None at the parked guard,
  `feet_pick_bit` Some — the fix, through the drone's own code.

## How it was verified

Ran (`NF_ISO` set, `cargo test --offline --release`):

- `cargo test -p nf-game --lib the_07000001_crack` — **1 passed**.
- `cargo test -p nf-game --test floor_census_live` — **1 passed** (103 nav nodes: no voids; 6
  airborne-tick step nodes land).
- `cargo test -p nf-view --test fix_guard_weapons_live the_3x3` — **not completed in the window**. The
  test is un-ignored on the branch; its build (the nf-view dependency stack, from a `cargo clean`-ed
  target) was started twice and the sandbox killed both background runs between tool calls; a
  foreground rebuild needs more than the remaining minutes. It is the first thing to run on this
  branch: `NF_ISO=<iso> cargo test --offline --release -p nf-view --test fix_guard_weapons_live
  the_3x3 -- --nocapture`.

Not run: the full fast-runner suite (`tools/test/suite.py`). The sandbox disk filled (9.9 GB, 0 free —
both the runner's build and this task's first build died), `cargo clean` freed 6.8 GB but a clean-tree
full rebuild did not fit the window; the crates this task touches (nf-game, nf-view) are covered above.

## Live checks for the main agent (optional, to pin the [S] items)

1. The pick-bit grounding at the crack: fresh boot on 07000001, park a guard at (−4.703, −5.968)
   (STEP_AIM_RIGHT 0x74), watch `anim+0x60` (bit 0x8) across ~10 ticks — expected: set every tick with no
   ray hit data copied into `drone+0xdc` (`HITDATA+0x58` stays 0).
2. The movement capsule's radius: read `dcv+0x45c` (hittest+0x8c) for a guard in substate 0xc vs a normal
   standing guard — expected 0.55 vs 0.4 [M 0x140e90:73-75] (pins the unmodelled 0.55 arm).
3. The fresh-probe pick reference: watch `hittest+0x50` (`dcv+0x420`) across the crack crossing — if it
   carries a per-tick DeltaP accumulation (not zeroed), the lifetime gate applies to drones too and
   `feet_pick_bit`'s `-inf` should become the accumulator's y.

## What's left

- The un-ignored 3x3 run (see above — first thing on this branch).
- The full suite on the final tree (see above).
- The [S] items above if a live check contradicts them.

## Usage

Token, context-window, cost and credit figures for this session: not visible to me (no such counters are
exposed in this environment; nothing to copy exactly).
