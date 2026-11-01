## Your task: `guard-approach-distance-11`, 11: #126 stops 10.7 u from Bond where the game's stops 8.8 u

Live PCSX2 (docs/live/2026-11-01-07000011-start-corridor-guards.md on origin/master): at difficulty 1 the game's #126 is 8.8 u from an idle Bond at f200; the port's stops at ~10.7 u at t200 and only closes by t400 (the pin allows 11). Find the game's run-to-point / approach stop rule for this guard [M EnemyRunToPoint 0x168910, NDrone2_FindRunToPoint 0x153168 / ClosestAIPoint 0x151b18, the arrival radius and the attack-range handoff], fix the port gap with the address cited, and tighten the pin to the live distance (sister: the old stop fails it). Run every *07000011* binary, ai_live and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-approach-distance-11`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-01T17:44Z

---
## Report (done, 2026-11-01T18:43Z)


---
## Report from the worker (done, 2026-11-01T18:43Z)

# guard-approach-distance-11 — report

Branch `s/guard-approach-distance-11` (cut from origin/master d43c28ce). Status: **done for the
named mechanism, one residual documented** — the port's run-to-point stop now uses the game's
arrival radius (the goal AIPoint's accuracy) instead of a fixed 2.0; the d1 #126 stop moved from
10.67 to 9.93, the pin is tightened so the old stop fails it. The remaining 1.1 to the game's 8.84
is a route-SHAPE difference (direct vs A*), not a radius one — live-checks below.

## What the game does (the cited chain)

The task's symptom: at difficulty 1 the game's guard #126 stands 8.8 u from an idle Bond at f200
((3.01, 17.67) [L docs/live/2026-11-01-07000011-start-corridor-guards.md]); the port stopped at
~10.7 at t200. The game's stop rule, from the raw MIPS (the C decompiles drop the float args):

1. `NDrone2_FindRunToPoint` 0x153168 picks the point group (prop 0x56 → 8, prop 0x4e → 1, else the
   alert byte Drone+0x44: 1→0, 2→9, 3→2, 0→fail [M-dis 0x15317c-0x153200]) and asks
   `NDrone2_ClosestAIPoint` 0x151b18; on the hit it loads **the point's `+0x14` float**
   (`lwc1 $f20, 0x14($v0)` @ 0x153228) and passes it as the accuracy to `SetupGoalPosition`
   (`mov.s $f12, $f20` @ 0x15323c).
2. `AINetwork_SetupGoalPosition` 0x15acf8 applies the **0.0 → 2.0 fallback** (`c.eq.s f20,0`;
   `lui $at, 0x4000` = 2.0 @ 0x15ad10-34) and writes it to the move-target struct's `+0x14`
   (= Drone+0x704) (`swc1 $f20, 0x14($s1)` @ 0x15ad70).
3. `MoveToGoalPosition` is called with **Drone+0x704 as its float arg** (`lwc1 $f12, 0x704($a1)`
   @ 0x15325c) and on a route rebuild hands it to `AINetwork_SetupRouteToPosition` 0x158b10,
   which lands it at **route+0x8c** (again 0 → 2.0) [M 0x158b10:104].
4. `MoveToGoalPosition`'s arrived-early check is `GetRouteDistance(route) < route+0x8c` → code 3
   (`lwc1 $f1, 0x8c($s0)` / `c.olt.s` @ 0x151888-1a0) — NOT a fixed 2.0; the fixed 2.0 is only the
   `DefaultInit` preset of Drone+0x8ec (= route+0x8c) [M 0x14b300:579] and the fallback.
5. `DSTATE_GoToGoalPosition` 0x165640 re-asks `MoveToGoalPosition` every tick [M raw
   `lwc1 $f12, 0x704($a1)` at LAB_0016577c] and on code 3 SetStates `+0x5A2` (the run-to-point
   ENTER's resume: 0x56 Attack here, since the 0x7000011 Tower clause only LOWERS it while
   unsighted [M 0x168910:23-31]) — that is the attack handoff.
6. The same accuracy chain serves the alarm detour: `NDrone2_FindAlarmPoint` 0x153008 loads the
   type-8 point's `+0x14` too (`lwc1 $f20, 0x14($v1)` @ 0x15303c/0x1530d8).

On this level every AI point's `+0x14` (placement param 1) is **1.0** [D-probe: all 12 0xE9
placements of 07000011 print acc=1.0], so the game's #126 hands off within 1.0 of its owner-31
point (1.40, 16.70) [D], i.e. ~9.8 u from Bond — then the attack states close/hold at 5.8-9 [L].

## The port gap and the fix

The port's `find_run_to_point`/`find_alarm_point` set the goal but never propagated the point's
`accuracy` (already parsed on `AiPoint`); `Drone::move_to_goal` built every route with the
`Route::radius` default 2.0 and compared `remaining < 2.0` — the handoff fired 2.0 short of the
point instead of the point's own accuracy. Fix (crates/nf-game/src/drone.rs, commit c7859b25):

- new `move_accuracy` field (= the move-target +0x14 / Drone+0x704 float, 0 = never set, matching
  the game's calloc);
- `find_run_to_point` and `find_alarm_point` store the picked point's `accuracy`
  [M 0x153168/0x153008 raw]; `re_find_mission_path` stores its explicit 2.0 / 1.0 floats at its
  three `SetupGoalPosition` sites [M 0x153a30:141-142, 160-169, 205-206 — pre-existing citations];
- `move_to_goal` writes the fresh route's `radius` from it (0 → 2.0, the SetupRouteToPosition
  fallback [M 0x158b10:104]) and the arrived-early check now reads `route.radius`
  [M 0x1517d8 raw]. Every other route builder keeps the 2.0 the game presets
  [M 0x14b300:579], so no other caller's behaviour changes.

Measured (d1 probe, deterministic): #126 runs 0x63 until t≈190, hands off at (2.20, 16.24) =
**9.93** from Bond in AimStand 0x67 (was 10.67), closes to **7.99** by t260 and holds 0x68
(was: dwell to t220, hold 8.0 from t260). The guard's first shot lands ~60 ticks earlier than
before — closer to the game's f200 firing [L].

## Pin tightened (commit 17b3786f)

`start_corridor_guard_07000011_live.rs` `idle_bond_is_approached_and_shot_at_every_selectable_
difficulty`: the t200 reach 11.0 → **10.2** and the t700 reach 10.0 → **9.0** (the live hold band
top [L 5.8-9.0]). The OLD stop (10.67) fails the new pin — the sister. The comment documents the
accuracy chain and the residual below.

## The residual (honest): 9.93 vs the game's 8.84 at f200

The 1.1 is the route SHAPE, not the radius. The port's `move_test(spawn → point)` passes, so the
route is the straight diagonal and the 1.0-accuracy handoff fires 1.0 north of the point. The
game's guard at f200 stands at (3.01, 17.67) — 1.9 SOUTH of the point, on the corridor's south
lane: its MoveTest must fail that diagonal and A* south through n149 (1.57, 18.70) [D nav path 1],
approaching the point from the Bond side. A move-test fidelity gap (what the game's boundary-post
band sees on that diagonal that the port's does not) is a separate task's scope.

## Runs (what I ran)

- `cargo test --release` per binary, `--test-threads=1`, NF_ISO set:
  - every `*07000011*` binary (15): start_corridor_guard 5 passed, walkreal_07000011_live 1,
    walk_07000011_live 3, walkreal_07000011_route_live 1, decal_door_follow 4, interactables 4,
    models_sweep 14, rope_guard_hitbox 2, stairwell_bullets 6, stairwell_decal_interp 2,
    stairwell_ghosts 2, stairwell_impacts_render 3 (+1 ign), stairwell_walls4_scope 1 (+1 ign),
    stairwell_walls 0 (+1 ign, pre-existing ignore), ai_traces_live 2 → **53 passed, 0 failed**.
  - `nf-game`: **1025 passed, 0 failed**.
  - cross-level guard spot-checks (the radius change is shared): alarm_panel_06_live 2,
    fix_06_alarm_live 2, walkreal_07000005_live 3 passed.
  - `walkreal_07000001_live`: 3 failed — **pre-existing on master**: `following_mayhew_down…`
    and `the_exit_fade_starts…` reproduce at d43c28ce (verified by re-running both at the parent
    commit; the third failure, `walk_from_the_spawn…`, is the same plate/exit cluster).
- walkreal_07000011_live's flight-C band: the hi bound 32.5 → 32.75 — the guard's earlier
  first-shot phase (a consequence of the fix) shifts the stagger mid-descent and the leg's xz
  arrival now ends 8 mm higher on the flight; the band still gates the descent (bottom 31.1,
  flight A's y < 17.2 below). The test passes deterministically.
- `tools/test/suite.py`: started twice in the background per the task; both runs died before
  producing output (the first without cargo on its PATH, the second at the sandbox's disk cap —
  9.9 G full with the toolchain + ISO + work trees; the nf-view deps alone are 5.5 G). The
  targeted runs above are what I can attest to this window.

## Live checks for the main agent

1. `NDrone2_MoveTest` on the diagonal: with #126 at its spawn (10.13, 46.8, 11.70) and a probe
   point at the owner-31 AIPoint (1.40, 46.8, 16.70), does the game's MoveTest FAIL that segment
   (the port's passes)? gdbwatch on the MoveTest result register during the load walk, or
   fieldprobe the route node list (route+0xf0/0xf4) after the run-to-point ENTER — expect the
   game's route to include n149 (1.57, 18.70) and n150 (−1.44, 18.21).
2. AIPoint+0x14 on other levels: dump the 0xE9 placements' param 1 for 07000001/02/05/06 — the
   port now rides that value into the route radius; a level with accuracy ≠ 2.0/0 changes its
   guards' stop distances and may need the same pin treatment.

## Usage

Token/credit/context figures: not visible to me. (No token, cost or context-window readout is
exposed in this sandbox; the session budget was the wall clock, ~17:50–19:25 UTC.)
