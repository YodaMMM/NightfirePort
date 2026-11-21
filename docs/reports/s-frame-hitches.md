# s/frame-hitches: frame work counted, the spikes' causes, the port-side fixes

The 240 Hz real-window sweep held ~241 fps on every on-foot level with single frames of 8.6-16 ms.
Wall-clock timing was unreliable on a loaded machine, so this branch counts the WORK each frame
does instead and lists the frames far above their kind's median.

## Instrumentation (off by default)

- `nf_formats::workcount` (`NF_WORKCOUNT=1`, or `force` in a probe): named per-thread event and byte
  counters. Off = one relaxed atomic load per hooked site. Hooked: every queue write
  (`write_buffer`, `write_texture`) and device creation (buffers, textures, bind groups,
  pipelines, shaders, samplers, layouts) in `gpu.rs`, through the `wb` / `wt` / `dc` helpers; the
  sound driver's first-use decodes (level bank samples, spare bank samples, streams).
- `crates/nf-view/src/hitch_probe.rs` (lib test module only): a counting global allocator for the
  lib's test binary (per-thread, off unless a probe counts; backtraces of large allocations on
  request), and a driver that runs `HeadlessApp::frame` (the window's logic half) at the 240 Hz
  cadence plus a replica of `App::draw_frame`'s draw half on a headless wgpu device.
  - `hitch_census_per_level` (ignored): each on-foot level's first 60 s, 20 s idle then a walk with a
    slow turn and a shot every 2 s. Tick frames and in-between frames are compared with their own
    medians; outliers are printed with their per-phase allocation bytes, counted events and the
    allocation sites.
  - `hitch_census_replays` (ignored): the committed 0a and 0b walk recordings, 6000 ticks each.
  - `hitch_alloc_sites_per_level` (ignored): the 64 KB-and-up allocation sites summed over a run.
  - `hitch_predecode_sizes` (ignored): the sound predecode's PCM size per level.

## Findings

Counted work is very even. After the first tick frame of a level, no frame on any of the 20 levels
allocated more than ~3 MB, and the GPU queue writes stayed within a few percent of the median
(0.35-1.2 MB a frame, the skin palette's dirty blocks). The largest outliers were these:

| Level | Frame(s) | Cause | Fixed |
|---|---|---|---|
| all 20 | any tick that first plays a sound (1-6 per frame, up to ~0.75 MB of PCM on 1b's first tick, 50-150 KB typical mid-play) | the level bank's samples decoded on first use, inside the tick | y: decoded on a worker after the level loads (`Driver::predecode_in_background`) |
| 01, 02, 03, 08, 09, 0a, 16 and more | dialogue and effect line starts (e.g. 01 t9.4 s 326 KB, 02 t12.7 s 448 KB, 03 first tick 530 KB, 08 t26 s 348 KB, 16 several 580-840 KB) | the streamed line's VAG decoded on first use, inside the tick | y: every stream the level bank's effects name is decoded on the same worker |
| 05, 06, 07, 08 (snow) | every frame | the snow quads built into a fresh ~670 KB Vec each frame | y: `snow_quads_into` refills one buffer (`App::snow_buf`) |
| 0c | every frame | the draw sort's stable-sort scratch buffer (~80 KB) | y: unstable in-place sort; the key ends in the unique draw index, so the order is identical |
| 08 | every tick | the copter path list cloned each tick (~230 KB) | y: borrowed in place |
| 1b every tick; 0b, 14, 15, 12 on ticks where something moves | `CollisionWorld::set_dynamic`'s rebuild into fresh buffers (1b: ~1.4 MB a tick) | y: the set before last's buffers are cleared and refilled (only capacity growth allocates) |
| all 20 | the first tick frame after the load (3-8 MB) | the first skin palette build, the first moving-collision set, the first snow buffer | n: one-off first-frame work at level start |
| all 20 | ~1 per second | the level music's next section decoded (`SsdDecoder::decode`, ~100 KB) | n: the music stream's own steady work |
| 05-08, 0c, 0d, 14 | every frame | the HUD overlay's build (~150-400 KB of small vectors) | n: steady, not a spike |
| any | a sound only another level's bank carries | spare bank samples still decode on first use | n: rare (the spare banks are the fallback for weapons a level lacks) |

The counted spikes are small next to the 8-16 ms worst frames of the real-window sweep. That
sweep's own breakdown put the worst frames in its "to submit" span while the measured CPU phases
in it (ticks, palette, blend + upload, encode) stayed at 1.2 ms or less. Most of that span is the
wait inside `get_current_texture`. So the worst frames are probably mostly the swap chain or the
compositor rather than port work. That is unproven here: it needs the real window on an idle machine.

## Proofs (counter tests, `hitch_probe::fixes`, release + ISO)

- `the_draw_half_reuses_its_snow_and_sort_buffers`: 05's median HUD/snow-phase allocation on draw
  frames is under 400 KB (before: ~740 KB), with no 512 KB allocation there. 0c's encode makes at
  most 4 allocations of 64 KB or more over 1200 frames (before: one every frame).
- `the_tick_reuses_its_collision_and_copter_buffers`: 08 has at most 12 logic allocations of 200 KB
  or more in 600 ticks (before: every tick). 1b has at most 12 logic allocations of 1 MB or more in
  600 ticks (before: every tick).
- `the_level_bank_decodes_before_play_and_plays_the_same`: 01's first 60 s with the walk, without
  the predecode: 39 bank + 9 stream first-use decodes (2.5 MB of PCM). With it: 0 + 0. Every tick's
  active sound list and played-sample hashes are equal, and so is the shared random stream at the end.
- Run against the four fixes reverted in place, the tests failed: 05 HUD/snow median 847 KB with
  897 allocations of 64 KB or more; 08 had 633 logic allocations of 200 KB or more in 600 ticks; 01
  still decoded 39 + 9. 0c and 1b come after those asserts in their tests, so they did not run in
  that revert. Their before-values come from the census (0c: 7112 allocations of 64 KB or more in
  the encode in 30 s, one every frame; 1b: `set_dynamic` allocated 1.4 MB every tick).

The predecode holds 8-24 MB of PCM per level (14 the most), decoded in 11-32 ms on the worker
(`hitch_predecode_sizes`, on a loaded machine). At that rate each first-use decode cost well under
a millisecond. The decodes were real per-frame work, but they cannot explain 8-16 ms frames on
their own.

## Behaviour

No game behaviour or random stream change: the caches hold the same PCM the first-use decode
builds, the sort order is identical (total key), and the reused buffers are cleared before every
fill. Kept green: load_scene_equality, frame_cost_live, frame_sampler_live, weather_draw_live,
replay_parity_0700000a/0b (whole missions, tick-exact), copter_paths_live, copter_hits_12_live,
homing_copter_07000012_live, walkreal_0700001b_live, walkreal_07000008_live, nf-audio and
nf-formats unit tests, the nf-game dynamic-collision unit tests.

## Remaining

- Real 240 Hz timing on an idle machine (not measured here).
- The first frame after a load, the music section decode and the HUD overlay build (see the table).
