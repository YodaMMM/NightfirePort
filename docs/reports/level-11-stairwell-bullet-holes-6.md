# level-11-stairwell-bullet-holes-6 — the scoped flush re-query landed; the last mid-air hole proven game-true and pinned

Task: land the scoped flush re-query the -5 report describes
(`segment_surface_flags_scoped(cell, from, to)` mirroring `segment_bullet_scoped`, the flush
using `find_cel(cur)`; [M] `Bullet_CollisionHandler` 0x12ce80's HITDATA walk) and re-measure.
If the one visible mid-air decal at ~(23.20, 28.22, 5.77) remains, compare the flush's
per-hit processing line by line against 0x12ce80 (post-ricochet scoped pick; the glist-0x10
node rule in `Collide_Intersect` 0x1ebaa8) and fix the remaining difference, or prove it
game-true [M/D] and pin that. Un-ignore the pin when it is green. Full suite, quote the
total line.

## What's done

1. **The scoped flush re-query is landed.** `CollisionWorld::segment_surface_flags_scoped(cell,
   from, to)` in `crates/nf-game/src/world.rs` — the `segment_bullet_scoped` straddle-BFS
   candidate set ([M] `Collide_Pick` 0x1eb320:125 takes the candidates ONLY from
   `Collide_StraddleCels`) + the same ray-tri body as `segment_surface_flags`, pass faces KEPT
   (back-face cull and the Möller–Trumbore gates identical to the flat query). The missile cage
   and the dynamic set stay in (the pick's per-cel object walk runs when HITTEST+0x98 bit 0x10
   is clear — the bullet's is, [M] 0x12f040). It falls back to the flat query when the scope or
   the cell is unknown, same safety net as the scoped actor queries. The flush walk in
   `drone.rs` (`flush_world_impacts`) now re-feeds `find_cel(cur)` every hop ([M] the flight's
   per-tick re-feed, Bullet_update disasm 0x12f0cc-0x12f0d8) and carries the hit's node pass
   flag as a 4th field into `penetration_roll_node` — the glist-0x40 node rule ORs into
   HITDATA+0x51 bit 1 and the walk's base pass reads that bit ([M]
   `Collide_Intersect` 0x1ebaa8:143-145, 0x12cea0..0x12cea8).
2. **The per-hit walk matches `Bullet_CollisionHandler` 0x12ce80 line by line.** The pass
   rules 0x12cee0..0x12cfb8 (surface byte 0x40 pass; material 0x10 glass pass unless
   +0x6c & 0x800; the penetration roll before any effect), the `Effect_Create` flags 0x61/0x10
   ([M] 0x12d198/0x12cf58), the explode arms gated on !pass ([M] 0x12d068-0x12d188), the
   ricochet mask 8 (the 9 variant needs node+0x3c & 0x400000, which no parsemap static node
   kind carries — [M] parseentity_fixup_entity 0x1d00c0, parsemap 0x1d1998:102), and the
   flight trace's HITTEST+0x98 = 0 ([M] 0x12f040) which drops nothing in Collide_Intersect.
   **No remaining difference was found; nothing to fix in the walk.**
3. **Re-measured on the merged tree.** The fan (4 reported spots × 8 yaws × 5 pitches, real
   input path) now yields **104 world impacts** (was 101 — the scoped candidate set is wider
   here than the flat query), **6 mid-air impact records** (was 5 — all on the stairwell's
   declared fudge planes, map43#126 "stairwell_blocking_ploys … a fudge!!!!" [D], surface
   byte 0x40 [D 07000011.bin], x≈15.95/x≈21.78; `mat & 0x3f == 0` blocks every Effect_Create
   including the decal [M 0x1ce6f8], so they are invisible), and **exactly ONE visible
   mid-air decal stays: (23.219, 28.226, 5.772), 0.402 from the nearest drawn triangle** —
   the third stop of the frame2944 spot's yaw +0.79 pitch −0.50 double-ricochet chain.
4. **The surviving hole is proven game-true by the ported rules and pinned.** Its face is in
   the game's own scoped candidate set (the re-query just landed IS the game's pick; the hole
   did not move while the impact count did), its material byte is real (the decal passed the
   `mat & 0x3f == 0` guard [M 0x1ce6f8]), and every per-hit rule of the flush matches 0x12ce80
   (item 2). This stairwell is full of declared fudge collision planes with real material
   bytes [D map43#126]. Under the ported rules the game's bullet hits the same face the same
   way and `Effect_Bullet` decals it [M 0x1ce8a8]. The pin now asserts this shape exactly:
   every non-pass impact on drawn geometry (≤ HOLE_ON_SURFACE_MAX), the six invisible 0x40
   records, and THE one game-true decal pinned to (23.219, 28.226, 5.772) ±0.02 / 0.402 ±0.05
   (the fan is deterministic).
5. **The pin is un-ignored and GREEN on the final tree.**

## How it was verified

- Final tree = `s/level-11-stairwell-bullet-holes-6` merged with origin/master (8dc61a92,
  58 commits, merge clean, pushed as a7c504cc).
- `cargo test --offline --release -j2 -p nf-view --test stairwell_impacts_render_07000011_live`
  on the final tree: `test every_stairwell_impact_sits_on_drawn_geometry ... ok` —
  `1 passed; 0 failed; 1 ignored` (the dump test stays `#[ignore]` by design, run with
  `--ignored --nocapture`).
- The full four-crate suite via the chunked runner (below).

## What I ran

- `cargo build --offline --release -j2 -p nf-view -p nf-game` (merged tree) — clean.
- `cargo test --offline --release -j2 --no-run -p nf-game -p nf-formats -p nf-audio -p nf-view`
  — all test binaries built (8m21s).
- The stairwell pin binary, `--test-threads=1`, on the final tree — green (above).
- **The full suite: interim, not complete.** This sandbox kills background processes between
  tool calls, so the suite ran through `tools/test/chunk_suite.py` (same build + per-binary
  runs as `tools/test/suite.py`, state persisted in `/home/z/suite11_6_state.json`), six
  rounds within the session's time limit:
  - **Interim total line (411/904 binaries): `1301 passed, 2 failed, 101 ignored`.**
  - **`fix_0c_pylon_live` (2 tests) — environmental, not the diff:** panics at
    `adapter: NotFound` — the sandbox has no GPU; wgpu finds no adapter. Same failure class as
    `probe_pylon_render` documented in earlier reports. The two failures are the adapter
    panic at fix_0c_pylon_live.rs:19, no assertions evaluated.
  - **`mp_stress_explosives_live` — environmental, not the diff:** `rc: -9` (OOM SIGKILL, 4 GB
    sandbox, 0 tests run). The previous session's state file (`/home/z/suite16_state.json`,
    a different tree without this branch's changes) recorded the **identical `rc: -9`** for
    the same binary — reproducible environment limit, known since earlier suites
    (mp_online_failures is the same class when it trips).
  - The remaining 493 binaries were not reached before the deadline; their status is NOT
    verified by this session. The suite resumes from the state file in a sandbox that still
    has it.

## Left (next session)

1. **Finish the full suite on the final tree** — ~493 binaries remain (list via the state
   file; `chunk_suite.py /home/z/suite11_6_state.json` until ALL DONE). Nothing in the
   covered set suggests a regression from the merge: every covered binary is green except the
   two environmental failures above.
2. **The main agent's render check is still the last word on the game-true decal** (the pin
   asserts the port's rules produce it; only the game's picture can confirm the game does
   too) — see "Live checks".

## Live checks for the main agent

1. **The surviving hole** — render and look: `--bond 22.40,30.09,1.42,0.79,-0.5` (the
   frame-2944 spot, the pinned chain's origin) and `--bond 22.40,30.09,1.42,-1.505,-0.035`.
   Expected if the pin is right (game-true): the game shows the same floating hole at
   ≈(23.20, 28.22, 5.77), 0.40 off the wall. Expected if the port is wrong: no hole there —
   then the difference is in a rule this session could not see, and the pin's decal assertion
   should be re-opened (do NOT weaken the geometry assertion).
2. **gdbwatch on the fudge plane's hits** during the same shot: the HITDATA+0x50 writes
   ([M `Intersect_RayGeom` 0x10a8c8:326 copies the surface byte]) and whether
   `Effect_Create` 0x1ce688 is reached for the 0x40-byte faces (expected: not reached, the
   `& 0x3f == 0` guard at 0x1ce6f8).
3. **PlrStat accuracy**: fieldprobe the accuracy stat before/after one shot that crosses the
   0x40 plane and stops on a real wall — does the game count 1 scenery hit or 2
   (the flush's first-world-hit logging vs per-HITDATA)?

## Usage

not visible to me
