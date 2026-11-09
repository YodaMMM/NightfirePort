# 0700000b: which roof-scene objects hide from the scene camera

The roof cutscene creates its own lift and lift_glass objects (Script_CreateEntity -> Control_SetGList). Control_SetGList 0x134960:14-15 hides an object from the scene camera when its glist flags carry 0x40000. Read the disc's glist flags for every object the 0b scenes create (lift 0x02000367, lift_glass 0x02000421, the NIS door, any others) and report which carry 0x40000; if any do and the port still draws them in the scene, name it in the report (draw change is the main agent's). No draw changes on this branch; a census test is welcome.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-0b-scene-entity-flags`, report in docs/reports/level-0b-scene-entity-flags.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox3 at 2026-11-09T06:30Z

---
## Report (done, 2026-11-09T08:10Z)

# level-0b-scene-entity-flags — report

Branch `s/level-0b-scene-entity-flags` (cut from origin/master 63b05cca6). Task: the 0b roof
scene's created objects (lift 0x02000367, lift_glass 0x02000421, the NIS door, any others) —
read the disc's glist flags for every object the 0b scenes create, report which carry the
0x40000 scene-hide bit, and name the port gap if any still draws in the scene. No draw changes
on this branch; a census test welcome.

## The law, first-hand (all [M] unless noted)

`Script_CreateEntity` 0x1d8968 attaches the command hash's glist to every created object: the
typed arms through their creators, the 0x31/0xDB fallthrough through
`hashtable_set_object_to_entity_gfx` 0x1ca6d8:38-40 → `Control_SetGList` (the decompile drops
the celglist argument; the hashtable value is it). `Control_SetGList` 0x134960:11-16 copies
glist+0x44 into obj+0xF0 and, when the word carries 0x40000, clears obj+0xF8's display-mask bit
0x10 (the u16 inits to 0x1F, control_init_object 0x134630:11). Both dynamic draw walks skip an
object when `(obj+0xF8 & object_display_mask) == 0` [M View_AddObjects 0x1e62d8:30;
View_AddForcedObjects 0x1e5fb8:79]. `View_CaptureScene` 0x1e6df0:15 sets
`object_display_mask = 1 << (viewer+0x28 & 0x1f)` per capture, and the layer byte is the viewer
index (build_alloc_viewer 0x1c9378:6-7; `glb_viewer` 0x2d88b0, 11 slots, `object_display_mask`
0x30d87c — ELF symtab). The cut-scene camera is viewer 4: `Script_CameraStart` 0x1d9140 stores
literal 4 into the stream's camera slot. `View_AddCels` 0x1e6ba0:57 is the STATIC twin: a
placement record whose +0x94 word carries 0x40000 is skipped only when the mask is 0x10.

**So glist-flag 0x40000 = "hide this object from the scene camera's pass (viewer 4, mask
0x10) only" — the game views (viewers 0..3, masks 0x1..0x8) keep drawing it.**

## The 0b census (the disc answer [D])

`crates/nf-formats/tests/scene_entity_glist_census_0b.rs` (opt-in NF_ISO) walks every kind-7
script of `0700000b.bin` and its kind-0xC packs — 23 scripts (0780000b/0790000b/07a0000b hold
060004f9 arrival, 060004fa roof, 060004fb), 221 `Script_CreateEntity` commands, 74 distinct
hashes, sub-scripts followed. The 0x40000 set is **exactly two objects**:

| hash | name | glist+0x44 | created by |
|---|---|---|---|
| 0x02000367 | lift (the paternoster cab) | **0x40080** | lift SP #232's pair: 06000a09/s0 + 06000048/s0, type 0x31 |
| 0x02000421 | lift_glass | **0x40081** | lift SP #232's pair: 06000a09/s1 + 06000048/s1, type 0x31 |

The rest draw in the scene pass. Named ones: the roof cut-scene 060004fa's **NIS door
0x02000668 'NIS_towerC_door' (type 0xDB) is glist 0x0** — the game draws it in the scene, so
the port drawing it is CORRECT. The arrival 060004f9 sub-scripts 06000046 (the helicopter set:
OBJECT_Cabin, Helicopter-Body/Rotor/TailRotor/searchlight/beam/glass, coronas — all scene-visible
words). The Bond re-place is entity-hash 0 (not in any hash table). No static visual placements
of the pair exist: the map-50 kind-0x31 rows (flags 0x2031) are the inert parking spots the
audit already lists, and map 16 #0 (NIS_towerC_door, flags 0xdb) is the load-point stub. So the
cab exists visually only as script entities.

## The port's state (the named gap — NOT fixed here)

nf-view's script-entity draw arm (scene.rs, the `Cmd::Entity` match arm) resolves the hash and
adds the draw slots with a glist bit-0x8 billboard check and an `owned_only`-gated bit-0x10
check — but **no 0x40000 check**. The static twins exist: `scene_hidden_cels`
(cutscene-cel-dup, [M View_AddCels 0x1e6ba0:57]) and `door_scene_hidden` (cutscene-door-dup),
both gated per frame on `sim.script_camera().is_some()`. The script-entity twin does not exist,
so **while any 0b scene camera renders, the port draws the lift cab and its glass where the
game's display mask hides them** (and would do the same on any other level's 0x40000 script
entities — the census shows 0b has exactly this pair).

Pinned red-to-be by `crates/nf-view/tests/scene_entity_flags_0b_live.rs` (real input, no
teleports): fresh boot, walked short of the arrival plates, per-tick real-input approach catches
the intro scene's camera the tick it engages (event-4 disable at the scene's tick 0; the scene
runs 690 ticks then the SP finishes into state 5 and deletes — a walk that overshoots misses
it). Under the camera, the cab's and glass's palette slots stay posed: one slot per
(script, stream) — the non-playing script of the SP #232 pair sits FAR, the live script's slots
hold y 11.0 (the cab at the shaft bottom). The test's assert expects the UNFIXED state; when
the draw change lands it goes red and should be flipped to require the collapse.

## The draw change (for the main agent)

In scene.rs's `Cmd::Entity` arm, mirror `door_scene_hidden`: an entity whose object's glist
flags (`level.maps[m].objects[object].flags`) carry 0x40000 collapses its slot(s) to FAR while
`sim.script_camera().is_some()` — the same per-frame shape as scene.rs:13137's door gate (the
palette `p[...] = FAR` write) or lib.rs:2171's `scene_hidden_world_centres` feed. Scope: the
slot is per (script, stream, hash, k) — all of an entity's slots collapse (the game's obj+0xF8
bit is per object, not per script).

## How this was verified

- `scene_entity_glist_census_0b` (nf-formats): PASS — pins the exact 0x40000 set (the lift
  pair), the NIS door's 0x0 word, and asserts the set's exactness so it cannot drift.
- `scene_entity_flags_0b_live` (nf-view): PASS — the real-input camera catch + the posed-slot
  pin (cab and glass).
- `walkreal_0700000b_live`: 7 passed, 0 failed (the touched level's walk, unchanged).
- `cutscene_cel_0700000b_live`: 2 passed, 0 failed (the static twin's own test, unchanged).
- **Full four-crate suite: NOT completed in this sandbox.** The sandbox disk (9.9 GB) filled at
  100 % — the linker died with SIGBUS mid-relink (the four-crate `--no-run` never reached
  `Finished`), background jobs are killed between tool calls, and every `--no-run` pass that
  did relink binaries refilled the disk. What ran, ran green (the four lines above). This
  branch changes NO `crates/*/src` code — only two new test files — so the exposure of the
  missing full-suite run is limited to the two new tests themselves (both green) and their
  `common/walker_b2.rs` include (unchanged; `walkreal_0700000b_live` green proves the include
  compiles and walks).

## What's left

- The draw change itself (main agent's, above) — nothing on this branch.
- Nothing else: the census answers the task's question in full; no port behaviour outside the
  scene-camera pass is affected by the finding.

## Live checks for the main agent

1. `object_display_mask` (0x30d87c, u16): watch during a 0b cut-scene — expect 0x10 (viewer 4
   capturing); during normal play — expect 0x01 (viewer 0).
2. The cab entity's obj+0xF8 during any 0b scene: expect 0x0F (bit 4 cleared by Control_SetGList
   for glist 0x40080); a scene-visible control, e.g. the NIS door 0x02000668's entity: 0x1F.
3. Render: during the roof ride cut-scene (060004fa) the shaft shows NO cab and NO glass; in
   normal play both draw (the paternoster ride). The port today draws them in both — after the
   draw change only the play views should.
4. Poke cross-check (optional): clear glist+0x44's 0x40000 on the cab's glist before the scene —
   the cab should then APPEAR in the scene pass (the bit is the whole mechanism).
