# s/drive-view-reflection: the car reflection map in the driving view

Draw change (driving view only). No sim change, no change to the shared random stream.

## What a player sees
The player car's paint, chrome and windows now pick up a reflection each frame: the track's sky
(`cref`) and whatever stands over the car (trees, rock faces, buildings, poles, overhangs, nearby
traffic), added over the base colour like the existing specular shine. It is most visible on the
windows, which were flat black before; the light silver paint was already close to white, so the
body changes little in the shots.

## How (docs/formats/car-model.md §10.5 pass 3, §10.6.3, §10.6.4)
- `crates/nf-view/src/drive_reflection.rs` (new):
  - `DriveReflection::build`: the track's `Map in#0` items with their drawn triangles (the flags
    test drops some items for good, so their geometry is left out), the AI vehicles' triangles
    grouped by texture, `cref` (track sheet, else `ext.ssh`), `rmsk` (`common.ssh`, only when
    `Desat ?` is on), and the car strips whose material has a reflection pass and Reflect > 0.
  - `DriveReflection::fill`: the camera from `CarRender::reflection_camera` and the documented
    R/U cross products, the projection of `ReflectionProjection::clip` turned into a wgpu matrix
    (texel space of `map_texel`, mirror kept, `Scene::mirror_x` not applied), the items
    `ReflectionItemQuery::keeps` keeps in table order, then up to 4 vehicles.
  - `DrivingState::reflection_frame`: fills the frame and adds the car's pass-3 group
    (`carmesh::reflection_vertex`: ST = (0.5 + v.x, 0.5 + v.z), RGB = min(Reflect·C_w,
    100000·max(0, v.y)), vertex alpha 127, additive) right after the shine group.
  - `ReflGpu`: one 256 × 256 Rgba8Unorm target, drawn at the start of `Renderer::render` with the
    main shader (its own uniform buffer and bind group, pipelines made once, fog off, gain 1.0),
    the steps in order: `cref` (s, s, s, 0x40) ALPHA 0x44, the scene, `cref` (s, s, s, Sky blend),
    `rmsk` as a reverse subtract. The map is a scene texture, so the car group samples it through
    the normal texture table; on the bindless path the map pass uses a copy of the array with the
    map's own slot set to white.
- `gpu.rs`: the pipeline builder takes a target format; the map texture is created as a render
  target; `set_reflection` / `reflection_target`. `driving.rs`: the per-strip item record, the
  build call and the `reflection` field. `lib.rs`: the frame and the upload, in the window and the
  headless shot paths. `scene.rs`: `Scene::drive_reflection`.
- Frame cost: no per-frame buffers or pipelines; staging vectors are reused. CPU side measured at
  0.15 ms median per frame on MIS4 (3,204 items tested, the car overlay included); the MIS4 test
  pose draws 21 draws into the map (7 items, 4 vehicles, 2 quads).

## Tests
- New `driving_reflection_live` (MIS4): the steps (two `cref` quads round the scene, no `rmsk` on
  the shipped tuning); at a known pose beside a tall item, the eye, forward, right, up and
  projection equal `CarRender::reflection_view` and the formulas eye = p + (0, HEIGHT, 0),
  forward = normalize(0.8 f + up), R × U = F; the GPU matrix lands world points on `map_texel`'s
  texel; the kept items equal an independent `keeps` pass over the decoded Map items with
  geometry; the draw list is the kept items' then the vehicles'; the player's car is not among the
  vehicles; the car group binds the map, is additive, and comes from exactly the Texture /
  TexturePacked / Chrome / GlassPacked strips with Reflect > 0 (no Dash); its first vertex matches
  `reflection_vertex`; the per-frame CPU cost is under 0.5 ms. On a GPU: the map without the scene
  matches the two `cref` layers texel by texel (over 90 % within 6 levels), and the scene step
  changes 46,135 texels, including round the tall item's projected texel. Passes on the bindless
  and the per-texture (`NF_NOBINDLESS=1`) paths. It does not build on the old code (no
  `drive_reflection` module).
- All 17 `driving_*` / `drive_cam_collide_disc` binaries green (19 tests), plus
  `hud_blend_gpu_live`, `split_runs_identity_live` and the nf-view lib `driving::` / `gpu::` unit
  tests. `frame_cost_live` loads the on-foot archives only (not the driving view), so it does not
  cover this path; it was not run.

## Unproven / port choices
- The map's clear value (black, alpha 1.0 here) and the GS viewport mapping (the texel formula is
  the documented inference).
- Whether the player's own car is among the 4 vehicles: left out here. The vehicles are ranked by
  distance to the main eye; `FUN_001ad0b8`'s LOD measure is not ported.
- The cell-tree gather before `keeps` is not ported: every item is tested with `keeps` alone.
  Items are drawn at LOD 0 (the port's track geometry), not `FUN_001ad2e0`'s pick.
- The texture functions (TFX/TCC), the clamp mode (the sampler repeats) and whether the map gets
  any display gain (none here) are not traced.
- `DAT_003349e0 + 0x30` (the flat-quad swap) is not handled.
- The car strips are baked at their rest part matrices in car space (as the shine), so `p` in
  the pass-3 formula is the car-space position, not the part's model-space position.
- The `rmsk` step is coded but never runs on the disc data (`Desat ?` is 0 in every archive).
- No PCSX2 comparison was made; the brightness of the window reflection (notably at night in
  Paris) needs a look against the real game.
