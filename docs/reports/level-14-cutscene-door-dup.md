# level-14-cutscene-door-dup — the control-room cut-scene door's static copy

## WORKLOG entry

level-14-cutscene-door-dup-b (2026-10-27): branch s/level-14-cutscene-door-dup-b from
origin/s/level-14-cutscene-door-dup (85157dd6, itself from origin/master ec917093). Symptom: on
Countdown Part 1 (07000014), the control-room cut-scene door (Bond at (40.87, 2.98, 40.85))
moves with the script streams (06000833 entity streams 10/11, hashes 0x02000891/0x02000892) but
a static copy stays in place behind it. Found the rule, fixed it, pinned it with a real-input
test; commit 8aeae4ea.

## What's done

**The duplicate instance [D].** The level's own placed door: the main map's kind-0xdb
placements at (27.7878, 1.2030, 49.6707) — two records (indices 117/118, both by object index,
flags exactly 0x000000db, no 0xa000 bits — verified by re-parsing 07000014.bin's block 0x1A
record by record), created as doors by `parsemap_create_dynamic_objects` 0x1d04f0 case 0xdb →
`Door_Create` 0x136100. The scene's movers are separate objects: `Script_EntityStart`
0x1d8d78 → `Script_CreateEntity` 0x1d8968's default arm (`control_create_object` +
`hashtable_set_object_to_entity_gfx` 0x1ca6d8) for obj_type 0xff. The placed leaf sits at the
placement pose, the scene keys at y 1.2965 z 49.5721 — ~0.09/−0.10 off, so the game's closed
placement reads as a "static copy behind" the moving leaf.

**The game's hide rule [M].** The glist flags (celglist+0x44, = the entity-params block word at
+8, = nf-formats' `MapObject.flags`) carry bit 0x40000 for the placed doors' packs:
`Control_SetGList` 0x134960 stores `obj+0xF8 &= 0xffef` when `glist+0x44 & 0x40000` — it clears
obj+0xF8's display-channel bit 4. `Script_CameraStart` 0x1d9140 disables cameras 0..6 and
enables ONLY camera 4 for the scene; `View_CaptureScene` 0x1e6df0 computes
`object_display_mask = 1 << (viewer+0x28)` and `build_alloc_viewer` 0x1c9378 sets
viewer+0x28 = the camera index — so during the scene the mask is 0x10, and both draw walks
(`View_AddObjects` 0x1e62d8:33, `View_AddForcedObjects` 0x1e5fb8:79) reject the doors on their
first gate. The scene's leaf entities bind their gfx through the entity-gfx hashtable
(`parsemap_block_entity_params` 0x1d1508 registered hashes 0x02000891/92 → map 010001c6's
Elevator_Door_Slide_3A/3B packs, flags 0x00000000 [D]) — no 0x40000, they keep the bit and draw.
The static-side mirror agrees: `View_AddCels` 0x1e6ba0:57 skips a cel whose flags carry 0x40000
exactly when the mask is 0x10.

**The fix.** `nf-view/src/scene.rs`: `build` now collects `door_scene_hidden` — the door
placements whose object's glist flags carry 0x40000 — and the per-frame palette pass
(`skin_palette`) collapses those door slots to the port's FAR "not drawn" matrix while
`Sim::script_camera()` is Some (the scene's camera 4). Normal play (camera 0, mask 0x01) is
untouched: before the scene fires the placed pair still draws closed. No sim change, no RNG, no
state constants touched.

**Verification.** `crates/nf-view/tests/cutscene_door_07000014_live.rs` gained
`the_placed_door_hides_from_the_scene_camera_while_the_leaves_move`: it pins the [D] glist
flags (0x40000 on the pair's packs Door_Slide_3A/3B, the corridor door's pack idx 103 with
0x80000 as the not-hidden control), that pre-scene the placed pair's slots draw, and that once
the scene fires on the real path (the vent-drop touch #301, bond at (40.7, 2.6, 39.0)) the
placed pair's slots collapse while the scene's leaf movers keep animating (max leaf gap
> 0.5 against the [D] open gap 1.6118). All 3 tests of the file pass. Locally also green:
cutscene_camera_07000014_live (1), door_aim_ray_live (2), door_glist0_e_dead_live (2),
ride_cutscene_objects_live (1).

**Suite.** Started in the background at 02:29 UTC on the pre-fix tree (845 binaries, build
1150 s); it had not finished by the deadline window. Its failures up to 03:54 UTC are all
environmental, none in code I touched: `identity_disc` + 3 `disc_golden` (the slim ISO's
golden-table drift), `fix_0c_pylon_live` ×2 + `probe_pylon_render` (no GPU/wgpu in the
sandbox — the prior task's worklog records the same two), and CRASH exit -9 for
`mp_bot_objectives_live` / `zerog_0700001b_live` (killed — sandbox memory). The door/cutscene
binaries affected by this change pass locally as listed above.

## What's left

- The full-suite result line once it finishes (the finish steps ran with the suite still in
  progress; re-run `tools/test/suite.py` on the merged tree before merging).
- The static-side mirror (`View_AddCels` 0x1e6ba0:57's 0x40000 gate for baked statics) is NOT
  ported — statics bake into vertex buffers at build time and cannot collapse per frame. No
  0x40000-flagged 0x8000-cel placement exists on 07000014's main map (all 0x40000 packs belong
  to door objects, which ride slots), so nothing visible is missing on this level.
- `Scene::script_camera()` is Some only while a camera stream runs [S unverified edge: a scene
  with no camera op would leave camera 0 enabled in the game — the game switches only on the
  camera op; 06000833 carries one].

## Live checks for the main agent

1. On a FRESH boot of 07000014, walk into the vent-drop trigger (Bond ≈ (40.7, 2.6, 39.0)),
   let the control-room scene play, and watch the control-room door: exactly one door should
   slide open (no closed copy behind it). Then after the scene, the same door closed again.
2. Same boot, before the trigger: the corridor door at (27.788, 1.203, −25.975) (pack idx 103)
   must open for Bond normally (my fix must not hide it — its pack lacks 0x40000).

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures to the
session. (The research subagent self-reported prompt 223716 / completion 4851 /
total 228567 tokens, read from its return message.)
