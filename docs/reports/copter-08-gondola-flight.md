# copter-08-gondola-flight — the 07000008 gondola helicopter flies its attack, not a hover

Task: on 07000008 the helicopter that attacks the gondola flies in, then holds one spot in the air,
and the mission completes. The port's copter hit walk (Copter_Update 0x17dbb8:277-320) and the gondola
completion on real shots are on master; the flight itself is the open part. Work in progress — updated
as steps land.

## Plan

1. Trace the port from the ride's start: the copter's position, route node, Copter_Update's case and
   target each tick until the kill; find where it stops moving.
2. Port the flight from the decompile: `Copter_Move` 0x17d628 (path follow), `Copter_Update` 0x17dbb8
   (case machine: approach, attack runs, strafes, hit reactions), the stream/route data [M + D]; cite
   each case's exit.
3. Tests from the ride (real input): the copter's position keeps changing between attack passes
   (moving-window distance check, the decompile's speeds), it dies on real shots, the mission completes
   only after the kill. Sister: an unshot copter never completes the mission.
4. Name the PCSX2 read that would confirm the flight (fresh boot, the copter's +0x30 over time).

## Status

- (04:47 UTC) Branch created from origin/master (8df0d8a4), report stub.
- (09:21 UTC) The trace + the attack machine ported; the flight was already on master (`Copter_Move`
  as `move_tick`, copter-path-flight) — the open part was the case machine's spot check and case 2.
- (09:41 UTC) The live attack pins + the unshot sister; the existing copter/walkreal tests stay green.

## What the trace found (task step 1)

The pre-task port never "stops moving" in its own state: the machine advances (case 1, the +0xF6 timer
counts), the route re-picks at the spline's end back into the same path at its reset-t (the [D] dump:
one path, count 8605, step 0.25309, reset 684 — `Copter_Move`'s pick walk finds only the pick-0 kind-0
path, [M 0x17d628:81-113]), and the visible body rides the matrix every tick (probed: the body entity
0x0200035D's world translation equals the copter matrix's row 3 at every 300-tick sample through tick
2100; there is no keyless-stream freeze on this level's body stream). The hover the symptom describes
is the ROUTE's own easing: the 8606-key spline's per-240-tick displacement decays 62.8 → 20.9 units
over the approach and bottoms out around unit-fractions per window at the dense tail — a near-stationary
copter the player sees hold one spot. What the game adds on top is the attack: the spot check leaves
case 1 for case 2 (the attack passes), and the case machine was exactly the piece not ported — the
case that never advances past 1 (the third of the brief's candidates; the other two are measured
innocent). The fix ports the spot check + case 2 so the machine flies its passes while the route keeps
moving underneath.

## The port (task step 2) — every case's exit cited

`crates/nf-game/src/copter.rs`:

- `diff_mod` — the game's `DiffMod` [M Copter_Create 0x17d340:76-90]: d2 1.5 / d3 2.0 / d4 4.0 / d1 1.0.
- `COPT_COS`/`COPT_SIN` = `PS2Sinf(3π/4)`/`PS2Sinf(π/4)` ≈ 0.7071 — the spot cone's 45° half-angle
  [M `___static_initialization_and_destruction_0` 0x17e468:7-8].
- `cone_sphere` — `Intersect_ConeSphere` 0x1ee100 verbatim (the apex-inside auto-hit [M :36-37, :54];
  the axial-wedge pass [M :47's else]; the surface test `t = cos·proj + sin·√(|d² − proj²|)`,
  `t ≥ 0 && t² − d² + r² ≥ 0` [M :49-52]).
- `spot_visible` — the case-1 spot test: `Mat_GetDir` 0x1e3398 (row 2) off obj+0x90, the cone on the
  player's world sphere (obj+0x80, r 1.0576 [L obj-tag-player.md]), then
  `Collide_LineOfSight(obj+0xC0 + dir·4.0, player+0x80, …, 0x2e)` [M 0x17dbb8:152-172; the probe
  scalar 0x40800000 = 4.0 at :157]. Mask 0x2e lives in `world.rs` as `LOS_COPTER_SPOT` (the shared
  `line_of_sight` walk is the documented equivalent while `LOS_MASKS_ON` is off).
- case 1 [M 0x17dbb8:138-187]: `Copter_Move`, then — behind `rec+0x74 != 100` (the Mayhew latch) — the
  +0xF6 timer counts every case-1 tick; from `FRAME_RATE_INT·4 / DiffMod` (240/DiffMod; the port's
  FRAME_RATE_INT = 60) the cone+LOS test runs every tick until it passes. Exit: state 2, timer 0,
  rec+0x6c 0, `attackmode = Rand_Rand(10)` [M :179-184]. No pass → stays in case 1 (the timer keeps
  counting; the test re-runs every tick).
- case 2 [M 0x17dbb8:188-242]: `Music_Event(2,1)` every tick [M :189] (the caller folds the flag),
  `Copter_Move` [M :190], the timer [M :192], then the shot — `attackmode & 1` picks the gun (odd) or
  the missile (even) [M :191-198]:
  - `Copter_FireMissile` 0x17d9f0: `Rand_Rand(0x86e6)` EVERY call [M :47]; the low bit picks pod slot
    8 (rec+0x50) / slot 7 (rec+0x4c) [M :49-54]; a missing pod returns after the draw [M :55-57]; on
    a tick where `GameState+0x34 % (FRAME_RATE_INT·DiffMod) == 0` [M :58-64] the shot spawns — on
    0x07000008 a further `Rand_Rand(8/DiffMod) == 0` aims it straight at the player (`normalise
    (player+0x30 − pod+0xC0)`), else the pod's `Mat_GetDir` facing [M :65-78]; the muzzle =
    `pod+0x80 + dir·(glist radius·1.5)` [M :81-94]; weapon 0x62 on 0x07000008, 0x6d elsewhere [M :81-86].
  - `Copter_FireGun` 0x17d828 via the existing `fire_gun_tick` (the 6-tick cadence, gun A slot 5 /
    gun B slot 6, the muzzle `gun+0x30 + dir·(glist radius)` toward `player+0x30 (+0.5 y)`, weapon
    0x61 on 0x07000008 / 0x71 else, `Sound_Play3D(…, 0x5df, …)` [M :55-96]).
  - The exits [M :199-242]: no player or the timer past `FRAME_RATE_INT·5·DiffMod` (300·DiffMod) →
    the timer resets; else with rec+0x6c ≤ 100 the spot test re-runs — a pass STAYS in case 2 (the
    damage tail runs after the case, [M :236 → LAB_0017e180]); a fail or rec+0x6c > 100 resets the
    timer; every non-staying exit lands in case 1 with rec+0x6c zeroed [M :241-242] — the fly/attack
    pass cycle.
- `PartPose` + `Scripts::copter_part_poses` (nis.rs) — the same 9-slot walk `copter_part_targets` uses
  (now mapped onto it), carrying the stream's world matrix (the pod's facing), its translation
  (obj+0x30/+0xC0 in step — the sensor module's own equivalence) and the world sphere (obj+0x80, the
  glist centre + the cel-glist radius `*(obj+0xD8)+0x38` [M 0x17d828:76, 0x17d9f0:82]).
- The caller (drone.rs copter tick): `SpotCtx` (the player's pos/sphere [M 0x17dbb8:149-155, :172],
  `DiffMod`, the frame for the fire gate, the level id) and the shot requests — each `CopterShot`
  spawns the shared `Bullet` off the weapon record's fields with the turret-spawn pattern (owner −1 —
  the hit-time drone reads skip a NULL shooter) and plays the gun's 0x5df.

## Deviations ([S] unless cited)

- `attackmode_159` is a per-record field; the game's is one static. Each level's attacking set is a
  single copter (05's patrols carry +0x74 = 100 and never leave case 1 [D the placement params], 08/
  0b/12 have one record each [D the route dumps]) — observationally equal.
- The part objs' obj+0x30 and obj+0xC0 read as one translation (the game's object loop keeps them in
  step; the sensor module cites the same equivalence [M 0x1af010]).
- `Mat_LookAt` on the gun parts each FireGun call [M 0x17d828:61-62] is not modelled — it aims the
  gun's MATRIX only; the bullets' directions come from the same positions the game's `Bullet_init`
  gets, so the shots are unaffected.
- The spot LOS starts the game's cel walk at the probe's `build_FindCel` cel [M :170]; the port's
  `line_of_sight` walks the shared world sets (the sight-mask work's documented delta).
- `Vec_Normalise`'s zero-length edge gets a guard (the game divides).

## Tests (task step 3) — what ran

- `cargo test --offline --release -j2 -p nf-game --lib copter` — 23 passed. New: the spot check opens
  the attack exactly at case-1 tick 240 with the `Rand_Rand(10)` draw; the cone gates it (a player
  behind the copter never flips it; the apex inside the player's sphere auto-hits [M 0x1ee100:36-37]);
  the attack pass fires on the gun's 6-tick cadence (weapon 0x61, sound 0x5df), holds while visible,
  bails at 300·DiffMod; the missile's draw order (the 0x86e6 pod draw every call, the 60·DiffMod fire
  gate, the homing draw on the gate tick), the 0x62 record and the muzzle radius·1.5; rec+0x6c > 100
  bails despite the sight (== 100 stays); the Mayhew latch (+0x74 = 100) never counts, never draws.
- `cargo test --offline --release -j2 -p nf-view --test copter_08_attack_live` — 2 passed. The
  approach: every flying tick advances t by the [D] route step 0.25309 and every 240-tick moving
  window displaces the copter by 62.8/36.4/25.8/20.9/26.5 units (the flight never parks). The attack:
  with Bond stood in the copter's facing line (the harness stand-in for walking into its fire line)
  the spot opens state 2, the pass holds past the gun's cadence, and stepping Bond behind the copter
  bails the pass with the position moved across the boundary — while an UNSHOT copter never writes
  ch 0x4B (the mission never completes). The machine draws the shared RNG (attackmode, the pod pick).
- The existing pins stay green on the changed machine: `copter_paths_live` (4 — the route node, the
  0x4B rule, the body ride, the 05 patrols' once-only flight), `copter_05_flight_live` (1),
  `walkreal_07000008_live` (4 — the ride, the shoot-down through real trigger pulls: the shots pushed
  damage 515 past hp 299 → state 4 → ch 0x4B → objective 4 → the mission end 0x0730008; the ally
  tests).
- The full suite (`tools/test/suite.py`) — see the final line below when the run finishes.

No existing test contradicted the decompile; none was weakened.

## Live checks for the main agent

1. Fresh boot 07000008, ride to the bottom. After the cinematic writes ch 5: sample the copter
   object's +0x30 (or +0xC0) every frame through the shooting phase — it should sweep the approach
   (the [D] route's keys) and KEEP MOVING while Bond is visible ahead of it, not sit on one spot.
2. The state byte obj+0xF4: expect 1 → 2 within ~240 ticks of Bond entering the forward cone
   (`Intersect_ConeSphere` with COPT_COS/SIN 0.7071, the 1000-unit gate), then 2 → 1 after
   ~300·DiffMod ticks out of the cone (or a hit pass over rec+0x6c > 100).
3. The shot: on 0x07000008 catch `Bullet_init` (weapon 0x61 from the gun part every 6th case-2 tick,
   or 0x62 from a pod on `GameState+0x34 % (60·DiffMod) == 0` ticks) and the muzzle offset
   `pod+0x80 + dir·(glist radius·1.5)`.
4. The part slots rec+0x30..0x54 after the SP spawns: WHICH slots are non-zero on 08. The port's
   probe cannot see the game's registrations from outside the sim; if slots 5-8 are zero on the real
   record, the attack draws its rolls but never spawns a bullet — the port should then show the same
   (state passes with no shots), and the live read settles it.

## The PCSX2 read that confirms the flight (task step 4)

Fresh boot (a save-state resume skips the boot path), 07000008, ride to the bottom: resolve the
copter object from the `CopterList` head (0x2a3830 — `Copter_Create`'s `LList_Add` [M 0x17d340:72]) →
the record's object, then record `obj+0x30` per frame during the ride. Expected: +0x30 sweeps the
route while obj+0xF4 reads 1, flips to 2 when Bond is in the forward cone, and the passes keep +0x30
moving (the route flies under the attack) until the kill drops it to 4 (→ 5). A frozen +0x30 with the
mission still completing would re-open the symptom.

## What's left

- The suite's final line on the merged tree (running at report time; the targeted runs above are all
  green).
- The draw side: the gun/missile muzzle flashes, the missile meshes and the explosion — the sim ships
  the shots; the render hand-offs stay open as usual.
- The real-record part-slot census (live check 4) to confirm the gun/pod registrations on 08.

## Usage

not visible to me
