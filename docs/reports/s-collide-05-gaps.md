# s/collide-05-gaps: the game's one-sided cylinder push

**This branch moves the shared random stream and turns 16 walkreal binaries red. Keep it on the branch.** It changes Bond's wall and edge response on every level. Every scripted walk's trajectory shifts, so the guard phases and the committed recordings shift with it.

The two gaps the ignored-walks census named are both on 0700000a (rows 9-10 of docs/reports/s-ignored-walks-1121.md), not 07000005: the west hall glide height and the west office corner.

## The finding: the push tail is not dead

The tests and comments around `capsule_tri_contact` say the game's player push tail almost never fires. The claim is that its reference HITTEST+0x50 is a lifetime DeltaP sum that sits far from Bond. That reading misses an argument that Ghidra drops:

- Both `ASH_vecutil_Dist2Tri` 0x109770 calls in `Intersect_CylGeom` 0x109da0 pass `t2 = HITTEST + 0x50`. The disasm at 0x10a2fc and 0x10a4f0 is `addiu t2,s1,0x50`, where s1 is the HITTEST argument (0x109dc4 `move s1,a0`). [M]
- Dist2Tri stores its closest triangle point through t2: 0x109aa0-0x109aa4 `bnel t2,zero` / `sqc2 vf3,0(t2)`. vf3 is the chosen vertex, edge or interior point (the `vmove vf3,vf4/vf5` arms). The return value is `|vf3 - query|`. [M]
- The push arm reads it straight back. At 0x10a53c-0x10a548 the code loads `lqc2 vf3,0x60(sp)` (the spine query point), `lqc2 vf1,0x50(s1)` and `lqc2 vf2,0(s1)` (the face normal), then `vsub vf3,vf3,vf1`. So v = p - closest point. The push follows only when `dot(v, n) > 0` (0x10a568) and `|v| < radius` (0x10a5c8). It runs along v-hat by `radius - |v|`. [M]
- Dist2Tri returns flags 99 when the query point sits behind the face plane (0x1097a8-0x1097c4). The caller then skips the triangle (0x10a33c and 0x10a528). [M]
- An edge or vertex closest point projects p back onto the spine with no second Dist2Tri call: the 0x10a344-0x10a3e8 arm ends `b 0x10a508`. Only the parallel-spine arm makes the second call (0x10a4fc). [M]

The real BL+0x790 is still the lifetime sum, so the [L] reads stay true: it boots at zero and drifts by DeltaP. `Collide_Intersect` 0x1ebaa8 runs CylGeom on a 0xa0-byte stack copy of the HITTEST (`amemcpy`). Dist2Tri overwrites the copy's +0x50 for every triangle before the push reads it. The real field only receives the `+= DeltaP` add-back.

In short, the game's push is the plain one-sided capsule push. The port's old shape differed from it in two ways. It pushed from behind faces (two-sided), and it had an accumulator-relative "radial" arm that pushed in an unrelated direction whenever it fired.

## The change

- `crates/nf-game/src/world.rs`: `cyl_game_contact` is the game's per-triangle body as listed above. That covers the raw-denominator head-on cull, the flags-99 skip, the edge re-projection, the `dot(v, n) > 0` gate and the zero vector at `|v| = 0`. It is the default for every `capsule_tri_contact` caller. `NF_CYL_OLD=1` restores the old shape for A/B runs. The walkable and pick signals keep their old rules. They are set only when a push lands.
- New unit test `world::tests::the_cylinder_push_is_one_sided_like_the_game`. A spine 0.3 in front of a wall is pushed out to r. A spine 0.3 behind it gets no push. It passes on the new code. It fails on the old shape (NF_CYL_OLD=1: "a spine behind the face plane is never pushed").
- `walkreal_0700000a_live`:
  - `the_west_hall_glide_height_matches_the_game` is **un-ignored and green**. Bond now settles at the game's 1.1285 ± 0.003 [L] in the west hall. The old code gave 1.0905. The jamb base's top edge lifts the capsule and the feet band holds the climb.
  - `the_pick_ref_accumulator_tracks_the_game_ref`: the return leg now walks to (-8.0, 16.9) on the game's z 16.83 hall line. The one-sided push holds the capsule off the jamb column's west face, so the old (-8.0, 16.29) spot can only be reached by placing Bond there, which is how the game's probe got there. Green.
  - `the_corner_crosses_like_the_game` stays ignored with a new reason. The route now wedges one leg earlier, at the west office door (-20.22, 1.04, 3.93). Bond sits between the jamb face at x -19.683 and the open door's small parts at (-20.39, 1.10, 4.47). The gap is 0.72 and the capsule needs 1.1. The corner line itself is never reached, so the corner gap is **not settled** on this branch.

## What a player would notice

On this branch walls only push Bond from their front side, edges lift him the way the game's do, and the 0a west hall walk height matches the game. Nothing here is visual.

## Tests run (NF_SILENT=1, -j 3, the 109 walkreal_* binaries)

The full sweep ran with the game push on (`--test-threads 3`, two binaries at a time). Every binary that went red was re-run on the same executables with the old push (A/B).

- **Green with the game push:** 04, 05, 0c_barrelrow, 0c_eastroute, 0c_ventexit, 11_route, 14, 41, 4a, 4a_satchel, celprobe, dump and every probe binary.
- **Red with the game push, green with the old push (16 binaries):**
  - 02: castle 00agent, stuck at the ch-0x17 plate with head_bump
  - 06: 4 interior walks, stuck "through the sill gap" and real-damage deaths
  - 07: 00agent death
  - 08: 4 whole-mission walks, stair or walkway wedges
  - 09: `the_alarm_switch_fails_objective_1`, the switch reach is missed
  - 0a: the 4 keyboard-hatch walks, the mission fails at door #725 and the committed recording no longer matches
  - 0b: 3 real-damage walks
  - 0c: the real-damage walk
  - 0d: 3 c5-door walks
  - 11: 3 real-damage walks
  - 12: the Agent walk
  - 15: 4 omega-exit walks
  - 16: 14 tests
  - 1b: 3 Drake walks
  - 41_wall: `the_one_sided_wall_blocks_from_the_north`
- **Red in both runs (not attributable):** 13, 3 warm-chain walks. This is the census's known nondeterministic red. It fails under `--test-threads 3` with the old push as well, and the plain walk passes alone with the game push.
- **Hit the 595 s cap, not measured:** 01 and 03. The census times 01 at 720 s. 01's finished tests were all green.
- nf-game lib: 1011 passed, 0 failed.

## Unproven, and the live checks that would settle it

1. Many of the reds are stair descents and squeezes. Examples: 08 "the stair head" and "the walkway head", 15 and 16 stair legs, 06 "the sill gap". The scripted routes may lean on the old two-sided push, but that is unproven. Whether the game's capsule passes those exact spots needs a live walk of each.
2. 41_wall: the file's header says the 2-tri wall collides "from both sides". With the flags-99 rule a single-winding quad cannot stop a capsule coming from behind its plane. A live walk at the wall's north face would settle which is right.
3. 0a corner: does the game's player collide with the open door's small parts at (-20.4, 1.1, 4.47)? If it does not, the port's dynamic door geometry carries parts the game's coll data lacks.
4. The VU clip-flag `break` inside the triangle loop (0x10a1xx, `cfc2` flag & 2) is not ported. It may end a box's triangle scan early.
