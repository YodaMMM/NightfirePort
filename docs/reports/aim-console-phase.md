# aim-console-phase — the 8 walks with the corrected premise

Branch `s/aim-console-phase` (from `origin/s/aim-candidates` ceb1ddae8, merged `origin/master` d6245281a — clean). Window 2, 2026-11-06 10:44–12:00 UTC.

## What's done

1. **The traced snap (src, behavior-preserving).** `Drones::aim_snap`'s body moved into `aim_snap_inner` with an optional trace sink; `aim_snap` is unchanged; the new `aim_snap_traced` returns `(point, Vec<SnapHit>)` with one push per winning stop — stages `level` / `object` / `wire` / `door` / `drone`, `SnapHit { stage, id, pi, t }` [M crates/nf-game/src/drone.rs, the `aim_snap_traced`/`aim_snap_inner` pair]. One implementation serves both, so the planner and the report read exactly what the shot reads.

2. **The 1b probe and the evidence** (walkreal_0700001b_live.rs: `[cands]`, `[ring]`, `[gate]`, `[snap]` logs). The driver's step 1 answered for the first failing console:
   - **Console 8's shots are NOT stolen by the snap.** The per-bearing ring scan around console 8's screen (c0 (51.78, −7.51, −9.42) r 0.33 — the bank steps down along x: screens y −0.60/−2.91/−5.21/−7.51 at x 12.69/25.72/38.74/51.78) measured **120/120 bearings snap-clean** at t2207 and again at t4000: the pick's first stop is the console's own screen object (hash 2000a41, t 0.044) from every ring stance.
   - **The raised rigs are behind the screens, not between.** The rig pieces (pi30 gate191 script 0x06000928; hash 20007e6 sph (25.72, −2.90, −11.39) r 1.80; hash 20007f6 sph (25.95, −4.35, −11.37) r 2.70) sit ~2 u behind console 4's screen (z −9.42) and appear as snap winners only while the rise script streams them (they beat nothing at t1296 — the screen's t 0.044 wins by 0.001–0.002 — and are gone from the candidate set by t2207). No rig stands between Bond and any console screen from the ring stances.
   - **The real failure is a floor landing.** The float wedges at (44.7, −8.66, 1.68), ~2.2 u short of console 8's station (46, −8.12, 0): the cross-room descent overshot onto the room floor (walk-mode pins it, thrusters dead — position frozen t2700→t5700), from where `sphere_shot_ok` is FALSE (the world clips the bank) and the round's line misses 0.54 > 0.8·r. The stall never tripped because the wedged crawl still moved 0.3 every ~110 ticks, keeping `moved_at` fresh.
   - The only game-code delta on this leg is the merged master (770a74927's alert fold changes the fight); the 1b test file is byte-identical pre/post merge, and aim-candidates' tree latched 8 and 7 on the same stations.

3. **The snap-aware planner wired on 1b** (the task's step 2, kept — it is correct by the same evidence): `snap_shot_ok` (the traced twin on the stance's crosshair ray, the same candidates the snap sees) gates `level_station`'s ring, the default station, and the fire tap; the stall measures **goal progress**; the replan anchors on the console's default station when the wedged spot's straight lines are all bank-clipped; the float grew climb/creep sag guards (full climb below the goal; floor-aware creep in the last 1.5 u of descent). Consoles 3/4/5/6 still latch.

4. **The [L] 08 pins and the 0x30000000 fix kept** — untouched; the refactor preserved every gate and constant.

## How it was verified

The battery, every binary whole, every number the aim-candidates pre-merge profile (the src change is behavior-preserving; the reds are pre-existing):

| binary | result |
| --- | --- |
| fusebox_geometry_07000014_live | 1 passed / 0 failed |
| interactables_07000014_live | 10 / 0 |
| rocket_stop_plane08_live | 4 / 0 |
| rocket_glass08_live | 3 / 0 |
| rocket_origin_live | 1 / 0 |
| solar_panel_hits_live | 1 / 0 |
| walkreal_07000008_live | 5 / 1 (pre-existing) |
| walkreal_0700000c_live | 2 / 1 (pre-existing) |
| walkreal_07000014_live | 6 / 3 / 1 ignored (pre-existing; single-threaded — 2 threads OOM-killed) |
| walkreal_07000016_live | 13 / 1 / 2 ignored (pre-existing) |
| walkreal_0700001b_live | 15 / 2 / 2 ignored (this task's reds: the drakes-defeat walks) |

The full `cargo test` suite did not fit this window; the eleven binaries above plus 1b's 15 other passes are what ran.

## What's left

- **1b console 8's landing**: the zero-g volume's floor boundary sits ~0.5 u above the low stations (the bank's stepped-down end); the crossing-waypoint attempt (cross high, descend at the station) went pathologically slow and was reverted — next lever: a hysteresis-free two-phase goal in `float_to`, or raising the ring's eye band for the y −7.51 screens while keeping the shot level (the eye must stay at the screen's height for `view_level`, so the margin has to come from the approach, not the station).
- **The other six reds (08, 0c, 14 ×3, 16)**: not triaged one-by-one yet. The evidence here says their snaps may also be innocent — run the same `[ring]`/`[gate]` probe per level before touching routes or levers; say which are route levers and which are raw-aim tuning.
- **The first wave's death ledger** (what it does besides channel 19, in order, with citations) for the PCSX2 phase hunt: not compiled.
- The full suite and the `npc_doors_live`/`fix_guard_misc_live` doors/guard binaries: not run this window (no `drone.rs` behavior change, but the next window should run them).

## Live checks

1. **The rig pieces' pick lifetime** — watch (gdbwatch) the object record for glist hash 20007e6 (the r 1.80 rig piece) from the first wave's death to the console phase: expect its `obj+0x90` matrix to rise from y ≈ −8.8 to ≈ −2.9 between frames 1782–1900, and its stream to leave the script's kind-1/2 set (the pick loses it) once risen — the port sees it as a candidate only mid-rise.
2. **The console-8 shot at the parked stance** — fieldprobe: place Bond at (44.7, −8.66, 1.68) facing console 8's screen (51.78, −7.51, −9.42), fire one laser round; expect the round to clip the bank's static geometry short of the screen (the port's `sphere_shot_ok` says the world stop is ≥ 0.55 u in front of the sphere centre) — i.e. the miss is the WORLD, not the snap.
3. **The zero-g volume's lower boundary at the bank's end** — poke Bond to (46, −8.3, 0) and to (46, −7.9, 0): expect the first to land/walk (outside the volume) and the second to float — that pins the boundary the crossing has to respect.

## Usage

not visible to me — this session exposes no token, credit, context-window or cost figures to copy.
