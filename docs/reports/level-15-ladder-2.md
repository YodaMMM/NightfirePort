# level-15-ladder-2 — the 07000015 ladder grab (round 2)

**Done.** The ignored test's premise was wrong: the game's own pick admission makes every ladder on
07000015 ungrabbable, and the port already matches the game. The test was fixed (per the task rules —
a test that contradicts the decompile is the thing that changes), un-ignored in its corrected form,
and given a sister that can fail.

## What the game actually does at the blocker

The grab never fires because the ladder is never a pick candidate — the game's own cel scoping
excludes it. The chain, with every link's evidence:

1. `Ladder_Create` 0x1966c0 creates the ladder object and links it ONLY through
   `build_LinkToRoom(obj, 0, glb_world)` → `build_FindCel` 0x1c8cb0 [M 0x1966c0 disasm].
2. `build_FindCel` 0x1c8cb0 keeps room cels (flags 0x40000) whose **bounds box** holds the point
   (:103-119: `cel+0x40 ≤ x ≤ cel+0x50 ∧ cel+0x44 ≤ y ≤ cel+0x54 ∧ cel+0x48 ≤ z ≤ cel+0x58`), and
   returns 0 when no cel's box holds it. Those boxes are the **entity-params records**:
   `parsemap_block_entity_params` 0x1d1508 fills the celglist slot from each 0x04 block —
   slot+0x20..0x2c = the record's min, slot+0x30..0x38 = its max (the same fields the port parses as
   `MapObject.min`/`max`), copied to `cel+0x40..0x58` by `parsemap_block_map_data_static` 0x1d1998
   (`amemcpy(cel+0x40, slot+0x20, 0x20)`), and `parseentity_transform_bounding_box` 0x1d0fb0
   transforms only the sphere (slot+0x10/+0x1c → cel+0x80/+0x8c), not the box [M]. So the game's
   FindCel box and the port's `CelNode.min/max` are the same disc numbers.
3. Ladder #382's world-sphere centre is (84.529, 31.733, −2.288) — `MapObject.centre` (0,0,0)
   (radius 8.64) through the placement [M `Control_BuildWorldSph` 0x133070; D the ladder1_3 glist:
   centre (0,0,0), min/max (±0.649, ±8.615, 0), the 0x2E mesh 2 triangles, a z-degenerate plane at
   constant x 84.529 spanning y 23.12..40.35, z −2.94..−1.64]. That point is inside **no** room cel's
   box [D the 07000015 entity params]: the east room's box is x 81.54..96.36, y 39.28..45.14 (the
   centre y 31.73 is 7.5 BELOW its floor), and every shaft cel's box ends at x 81.87 (west of the
   ladder's x 84.53). `build_FindCel` misses.
4. On the miss, `build_LinkToRoom` 0x1c9680 links nothing and adds no ForcedList entry; obj+0x20
   stays 0 [M `control_create_object` 0x1344e0 memsets the object].
5. `Ladder_Create` then calls `Controls_StraddleTest` 0x134778 [M 0x1966c0]. Its straddle BFS starts
   from **obj+0x20** [M disasm 0x1347b8 `lw $a2, 0x20($s0)` / 0x1347d0 delay slot `sw $a2, 0x80($sp)`
   — the HITTEST's start-cel field], so with the link failed it starts from cel 0;
   `Collide_StraddleCels` 0x1eac98 then returns just the start cel (count 1), and the count < 2 arm
   clears the link/force state instead of adding to the ForcedList [M disasm 0x1347d8 `sltiu` /
   0x13482c path].
6. `Collide_Pick` 0x1eb320's candidates are the straddled cels' object chains (:125-134) plus the
   ForcedList loop (LAB_001eba38) — the ladder is in neither → `Collide_PickObj` never runs for it →
   `Player_DealWithObjHit` 0x194458 never dispatches class 0x2D → `Player_CollLadder` 0x196118 never
   executes for this ladder. Ladders #900 (84.53, 31.73, 69.07) and #901 (−13.34, 31.73, 73.33) miss
   FindCel the same way [D the same boxes]. **No ladder on this level can be grabbed from any spot,
   standing or otherwise.**

The "approach the game's floor allows" turned out to be irrelevant by design: there IS a real walkway
at the ladder's top (the east room's south ledge, its top strip y ≈ 40.5 over z −2.13..−1.8, then the
south walkway y ≈ 40.35, z −2.86..−2.13, x 82..90.7 [D floor tris] — the ladder's own plane crosses
it), and the ladder's mesh WOULD be in pick reach from a stander there (the sister proves it). The
admission alone blocks the grab.

The "lower deck (y 23.8)" in the old ignore message is also wrong about the geometry: there is no
floor anywhere near the ladder base (floor_below returns None across the whole shaft region [D
probes]; the level's mid cells at y 20.6..38.3 end at x 81.87).

## Port status

The port already matches the game: the load resolves `lad.cell = world.find_cel(lad.centre)`
(scene.rs, the game's create-time `build_LinkToRoom` FindCel) → None for all three ladders, and
`ladder_grab` skips ladders whose cell is not in the per-tick straddle set (the port of
`Collide_Pick`'s scoping). Nothing in `nf-game` needed a behaviour change; the only code addition is
`CollisionWorld::debug_scope_cells` (a debug introspection accessor alongside `debug_straddle` /
`debug_bucket_owners`, used to pin the box evidence from a test).

Noted, NOT ported (out of scope, no shipped case): the game's ForcedList rule
(`Controls_StraddleTest` 0x134778 adds an object whose own sphere straddles ≥ 2 cels;
`Collide_Pick` LAB_001eba38 picks those unconditionally). For ladder #382 the game's own
StraddleTest starts from cel 0 and does NOT force it, so porting the rule would not change this
level; a shipped linked-but-unstraddled ladder case would need its own evidence first.

## The test fix

`crates/nf-view/tests/interactables_07000015_live.rs`:

- `the_ladder_takes_the_climb` (ignored) asserted "the walk into the ladder engages the climb [M
  Player_CollLadder]" — that contradicts the admission chain above. **The TEST was wrong.** Replaced
  (round 2) by:
  - `the_dead_ladder_never_grabs_at_point_blank` (un-ignored): pins the premise (find_cel None for
    all three ladder centres [D boxes; M 0x1c8cb0]; all loaded ladders `cell == None` [M 0x1c9680]),
    then a real-input walk (only a reach teleport onto the walkway — the room's south wall slides the
    walker east off the direct line, the same wall round 1 reported) WEST across the ladder plane's x
    with the grab-favourable facing (yaw −π/2, |AngleDiff| = π > π/2 [M 0x196118:20]). Asserts the
    climb NEVER engages while the pre-push capsule crosses the plane's x (min x < 84.6) — Bond's
    capsule passes through the pick's reach with no grab, exactly the admission-dead behaviour.
  - `sister_the_dead_ladder_mesh_reaches_the_point_blank_stander` (can fail): at the point-blank
    stand the ladder's own pick DOES hit (`Ladder::capsule_hits_default` with the live bond
    pre-collide spot, RADIUS, cc) — pinning that the blocker is the admission chain, not the
    geometry or the pick math. It fails if anyone changes the parsed ladder mesh, the pick shape, or
    the cel boxes (a relinked ladder flips the main test too).

## Census correction

`docs/levels/07000015.md` §7 ladder row: the old "live (ladders proven on 07000006/09; one climbed
here)" was wrong — no ladder on this level is climbable in the game's own admission; the row now
carries the chain and the live pair.

## How it was verified

- `cargo test --offline --release -p nf-view --test interactables_07000015_live -- --test-threads=1`:
  **10 passed; 0 failed; 1 ignored** (the pre-existing `the_pickups_take_on_the_walk` ignore,
  untouched) — including the two new tests.
- The full suite: see the total line below (run on the final tree, merged with origin/master).
- Every assertion cites [M] decompile/disasm addresses or [D] disc data as listed above; no [S] item
  is in a pass/fail position.

## Live checks for the main agent

To confirm the [M] admission chain against the running game (fresh boot of 07000015 — a save-state
resume skips the create path):

1. Break on `Ladder_Create` 0x1966c0 (ACTION.ELF, execution) — it returns 3 times, once per ladder
   (the placements #382/#900/#901); capture `$v0` after each return (the ladder object pointer).
2. For each captured pointer P, read:
   - `P+0xFF` (u8): expect 0x2D — confirms it is the ladder object [M 0x1966c0].
   - `P+0x20` (u32): expect **0** — unlinked; the game's FindCel missed [M 0x1c9680 miss path].
   - `P+0xF0` (u32): expect bit 0x10008000 **clear** — not on the ForcedList [M 0x134778 count<2
     arm]. (Other bits may be set later by non-admission systems; only 0x10008000 matters here.)
3. Optional: watch writes to `P+0x20` for the rest of the level — expect none (nothing ever links
   the ladder; `control_handle_cel_change` 0x133288 only relinks from an existing link).

Any non-zero `P+0x20` or a set 0x10008000 on any of the three objects falsifies the chain and the
test fix — please report it.

## What's left

Nothing for the task. Follow-up candidates (not started, no evidence gaps blocking):
- The ForcedList rule port (above) if a shipped case ever shows a linked-but-unstraddled ladder.
- The other levels' ladder censuses could carry the same one-line admission check (find_cel at each
  ladder centre) — 07000006/09's ladders link and climb, so their rows stand.

## Suite

(to be filled from `tail -1 suite.out` on the final tree before finish)

## Usage

not visible to me — this session exposes no token, cost, or context-window figures.
