# level-11-stairwell-bullet-holes-5 — 07000011 stairwell bullet impacts vs render geometry

Task: the 10-25 report (holes hanging mid-air along the railing level, on a plane in the well
when looking up, invisible walls eating shots; 10-26 "maybe progress?" unresolved). FIRST
measure the current state on master AND on `s/capsule-cull-and-los-masks-11`; only if mid-air
impacts remain, find which collision faces stop the port's bullet that the game's bullet passes
and fix the bullet path. Report the measurement either way.

## What's done

1. **The measurement exists and ran on both trees.** New test
   `crates/nf-view/tests/stairwell_impacts_render_07000011_live.rs`:
   - `dump_the_stairwell_impacts_vs_render` (`#[ignore]`, run with `--ignored --nocapture`):
     fires a real fan through the real input path (`step_bond` trigger press → gun →
     `fire_player` → the flight → `flush_world_impacts`) from the four reported stairwell spots
     [D the 10-25 report and the pinned tests: repro (15.33, 26.69, 1.33), frame-2944
     (22.40, 30.09, 1.42), the wake-plate spot (22.34, 29.37, 3.52), the upper landing
     (15.35, 35.42, 1.61)] — 8 yaws × 5 pitches (up/across/down the well) each, 160 rounds.
     Every world impact and every bullet-hole decal is measured against the render mesh: the
     distance from the point to the nearest drawn World-layer triangle of the loaded Scene
     (spatial-hash + closest-point-on-triangle over `Scene.vertices/indices/materials`).
     Impacts get their collision face attributed by a short re-cast through the point
     (`segment_surface_flags` → surface byte + node pass flag; the `impacts_live.rs` idiom).
   - `every_stairwell_impact_sits_on_drawn_geometry` — the pin; runs **ignored-RED today**
     (see below), the reason and the numbers are in its attribute.
   - Harness notes: the clip lives in Bond's inventory and re-syncs into `gun.clip` every tick
     (the BLData `+0x1B8`/`+0x170` slots), so the fan refills `inventory.clip`/`carried` —
     harness ammo, not a sim rule; 25 settle ticks per shot clear the gun's fire interval.

2. **The numbers (identical on both trees — the capsule masks work does NOT change this):**

   | tree | world impacts | mid-air impact records | decals measured | mid-air decals (visible holes) |
   |---|---|---|---|---|
   | master (c0d3e92e) | 101 | 5 | 200 (the 200-decal eviction cap) | **1** |
   | s/capsule-cull-and-los-masks-11 (a16ab2a4) | 101 | 5 | 200 | **1** |

   - The **5 mid-air impact records** (all trees): every one lands on a face with **surface byte
     0x40** (the bullet-pass byte), node_pass false — the undrawn plane family around
     x≈15.95 and x≈21.78 (the fudge, map43#126, whose own disc name says "stairwell_blocking_
     ploys … a fudge!!!!" [D]; surface byte 0x40 [D 07000011.bin]). These records are
     INVISIBLE to the player: `mat & 0x3f == 0` is the Effect_Create guard [M 0x1ce6f8], so no
     decal, no gas, no debris spawns there; `player_impacts` has no render consumer. They only
     touch the accuracy stat (`PlrStat_LogShotHitScenery`, pushed at the flush's first world
     hit — drone.rs `flush_world_impacts`).
   - The **1 visible mid-air hole**: decal at **(23.20, 28.22, 5.77), 0.40 from the nearest
     drawn triangle** — from the frame2944 spot, yaw +0.79, pitch −0.50. That shot's ricochet
     chain produced 4 decals; this one is a post-ricochet tick's stop on a collision face that
     has a REAL material (a decal spawned) but no drawn surface within 0.40 — a hole hanging
     0.40 in front of the wall. This is the 10-25 "holes hanging in mid-air" symptom,
     reproduced through the real input path [L].

3. **The face hunt and the attempted fix (attempted, measured, then reverted — see "Left").**
   The flush's re-query (`drone.rs flush_world_impacts` → `segment_surface(cur, end, false)`)
   is FLAT, while the game's bullet pick takes its candidates ONLY from `Collide_StraddleCels`
   plus the ForcedList [M Collide_Pick 0x1eb320:125; the walls4 test's dump]. I implemented
   `segment_surface_flags_scoped(cell, from, to)` (the `segment_bullet_scoped` straddle BFS +
   the same ray-tri test as `segment_surface_flags`, pass faces kept — the handler's walk
   decides the pierce itself [M 0x12cee0..0x12cfb8]) and switched the flush onto it
   (`find_cel(cur)` per hop, mirroring the flight's per-tick re-feed [M 0x12f0cc-0x12f0d8]).
   **Measured result: the numbers did not move** (104 impacts / 6 mid-air records / 1 mid-air
   decal at the same point) — the faces involved are IN the straddle set. The flat-vs-scoped
   mismatch is real [M] but it is not what produces this stairwell's visible hole.

4. **What the surviving hole means.** The hole's face is in-scope, has a real material and no
   drawn surface within 0.40. Under the ported rules the game's bullet would hit the same face
   the same way (the pick is scoped; the face is in the set) and `Effect_Bullet` would decal
   it [M 0x1ce8a8] — i.e. the mid-air hole may be GAME-TRUE behaviour of this fudge-laden
   stairwell (undrawn collision planes with real material bytes). Deciding that needs the
   game render, which I cannot run — see "Live checks for the main agent".

## How it was verified

- Both trees measured with the same binary (the test cherry-picked onto a throwaway branch of
  `s/capsule-cull-and-los-masks-11` in a worktree; worktree removed after). Raw dumps were
  saved in the sandbox as `/tmp/dump_master.txt` and `/tmp/dump_caps11.txt` (not committed —
  scratch), the shape is quoted above and reproducible via the `#[ignore]` dump.
- The pin test `every_stairwell_impact_sits_on_drawn_geometry` reproduces the symptom RED on
  master (0 passed / 1 failed before the ignore mark). It runs `#[ignore]` now so the suite
  stays green; the numbers live in its attribute and here.
- `stairwell_bullets_07000011_live`'s two tests (the 10-26 state) still hold on this tree:
  the round from the reported spot passes the fudge's 0x40 plane and damages the rope guard;
  the downward round stops on the stairwell's own geometry (ran in this sandbox, green).

## What I ran

- `cargo test --offline --release -p nf-view --test stairwell_impacts_render_07000011_live`
  on the final tree (master c0d3e92e + the new test file): `0 passed; 0 failed; 2 ignored`
  (the dump and the ignored-RED pin), green.
- The full four-crate suite: **blocked by sandbox disk, not run to completion.** This sandbox
  has ~10 GB total; the shared warm target cache (2.4 G) plus the ISO, work/, and the
  nf-game/nf-view rebuild churn (branch switches + the two scoped builds) filled it to 100%
  twice — the linker died with `collect2: fatal error: ld terminated with signal 7 [Bus
  error]` (disk exhaustion). I freed 3.9 GB by pruning stale test executables and the cargo
  registry cache, but the remaining window did not fit the full relink (~150 nf-view/nf-game
  test binaries) plus the run. `tools/test/suite.py` could not produce its total line. The
  binaries that did build and run on the final tree: the new stairwell measurement binary and
  `stairwell_bullets_07000011_live` — green. **The next session must run the full suite
  before merging anything.**

## Left (next session)

1. **Decide whether the surviving mid-air decal is game-true** (the live check below). If the
   game does NOT show a hole there, the next candidate rule is in the flush walk's per-hit
   processing (compare `Bullet_CollisionHandler` 0x12ce80's HITDATA walk line by line against
   `flush_world_impacts` — especially whether the game's decal for a post-ricochet tick uses
   the reflected direction's own scoped pick, and whether undrawn-cel faces are excluded
   somewhere else, e.g. the glist-0x10 node rule inside `Collide_Intersect` 0x1ebaa8 rather
   than the pick's candidate set).
2. **The scoped flush re-query is still [M]-correct and worth landing on its own merits** —
   the flat re-query can manufacture contacts the game's pick never had (a real defect even
   though it does not change THIS stairwell's numbers). My implementation (this sandbox, not
   committed): `CollisionWorld::segment_surface_flags_scoped(cell, from, to)` mirroring
   `segment_bullet_scoped` (world.rs:1036) with the `segment_surface_flags` ray-tri body, pass
   faces kept; the flush calls it with `find_cel(cur)`. It needs the full suite first — it
   changes which faces exist for the walk everywhere, and the walk's rolls feed the shared RNG
   (the fix-gas-rng4 lesson).
3. The 5 mat-0x40 impact records: confirm against the decompile whether
   `PlrStat_LogShotHitScenery` runs per HITDATA (including pass faces) or once per stop —
   a one-line accuracy-stat difference, invisible in play.

## Live checks for the main agent

1. **The surviving hole** — render the repro and look: `--bond 22.40,30.09,1.42,-1.505,-0.035`
   (the frame-2944 spot), fire toward yaw ≈ +0.79 (≈45°, toward +x/+z), pitch ≈ −0.5 and also
   the plain shot `--bond 22.40,30.09,1.42,0.79,-0.5`. Expected if the port is wrong: the
   game shows NO hole at ≈(23.20, 28.22, 5.77). Expected if game-true: the game shows the
   same floating hole 0.40 off the wall (this stairwell is full of declared fudges [D]).
2. **gdbwatch on the fudge plane's hits**: during a shot from the frame-2944 spot at the
   x≈21.78 plane, watch the HITDATA+0x50 writes ([M `Intersect_RayGeom` 0x10a8c8:326 copies
   the surface byte]) and whether `Effect_Create` 0x1ce688 is even reached for the 0x40-byte
   faces (expected: not reached, the `& 0x3f == 0` guard at 0x1ce6f8).
3. **PlrStat accuracy**: fieldprobe the accuracy stat before/after one shot that crosses the
   0x40 plane and stops on a real wall — does the game count 1 scenery hit or 2?

## Usage

not visible to me
