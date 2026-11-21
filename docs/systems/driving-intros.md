# The driving missions' scripted control and cameras (the openings)

Status: ported as pure state (2026-11-21). Code: `crates/nf-formats/src/drivepath.rs` (the
`AISp` splines, `Cams` cameras, `pt` path animations) and `crates/nf-game/src/driving/intro.rs`
(`IntroState`: control owner, spline + path player, forced stop, POV mode, scripted camera,
letterbox, cinematic flags, sim rate). `DriveScript` (`driving/script.rs`) now reports every
unported event as `Output::Effect { id, trigger, action }` with its argument block, and
`IntroState::apply` runs the ones below. Tests: `intro.rs` / `drivepath.rs` unit tests,
`crates/nf-game/tests/driving_intro_disc.rs` (needs `NF_ISO`). Evidence: [M] DRIVING.ELF
decompile (EE addresses), [D] the eight tracks' data, [S] inferred, listed in §6.

## 1. The data [M 0x2da838, 0x2da088, 0x2da590, 0x2dab48, 0x2db370; D]

- `AISp`: 0x70-byte records: a 4×4 matrix, `+0x40` the path animation (on disc an `sr` word
  naming `CARP::<<Map>>::{pt  NNNN}`, NNNN hex), `+0x44` the name (`path_BondMaster01`). The
  record matrix is not the path's start (the first key lies 3 to 1760 u from it) [D].
- `Cams`: 0x40-byte records: a 4×4 placement; `+0x1c` (0 except MIS13A camera 6) an `as`
  reference.
- Script arguments name records by byte offset: `{AISp}::224` = spline 2, `{Cams}::64` = camera 1.
- `pt`: `+0` duration, `+4` channel count, `+0xc` rotation channel, `+0x10` position channel,
  descriptors of 0x14 bytes at `+0x40` (`+4` times, `+8` values, `+0xc` count, `+0x10` flags).
  Flags 0: vec4 keys, linear; 2: quaternion keys, normalised blend with a sign flip; 1: cubic
  keys (0x30 bytes). Key = the last one at or before t; fraction `(t − t0)/(t1 − t0)`.
- On disc [D]: every spline's `pt` carries the spline's own name, except MIS13C spline 25
  (`path_(JC_freighter)`, its `sr` word names nothing). One channel is cubic (MIS13C `pt#1`,
  the position of `path_nomovement`); the port leaves its values empty.

## 2. The events [M, handler addresses]

| event | handler | effect |
|---|---|---|
| `EControlToSpline(car, spline)` 0xca | 0x14ae60 | needs the spline and its `+0x40`; car 0 = player: the controller's vtable `+0x24` gets the spline, `FUN_001bd198(dir, 1)`; else the AI element's controller, when its `+0x48` and `+0x44` are 1 |
| `EEndControlToSpline(car)` 0xef | 0x14c110 | the controller's vtable `+0x2c`; player: `FUN_001bd198(dir, 0)` |
| `EControlToCPU` 0x2a | 0x14ac38 | driver `+0x70` = 2 (`FUN_00130e88`), flag 1 |
| `EControlToPlayer` 0x2b | 0x14ad18 | driver `+0x70` = 1 (`FUN_00130f98`), `FUN_00167d80()+4 = 1`, flag 0 |
| `EForceCarStop(a)` 0xdc / `EEndCarStop` 0xdb | 0x14ede0 / 0x14c048 | vehicle vtable `+0x274(a ? 3 : 1)` / `+0x27c(3)` |
| `EAutoDriveSpeedChange(v)` 0x6c | 0x1482f8 | driver `+0x134` = v |
| `EPathSetDesiredThrottleStatic(e1, e2, v)` 0x74 | 0x152cb0 | path player `+0xc4` (target speed) = v; (0, 0) = the player's (`FUN_00145d00`) |
| `EPathSetTimeStatic(e1, e2, f)` 0x99 | 0x153340 | path time = duration × f, unless the path is parented |
| `ESetAutoDriveCamOn/OnMovie(angle, pitch)` 0x4d / 0xea | 0x1576e8 / 0x157838 | POV mode on (only when off), camera start angle/360, pitch/360 |
| `ESetAutoDriveCamOff/OffMovie` 0x4c / 0xe9 | 0x157460 / 0x157540 | POV mode off |
| `EPlayCameraAnim(cam, smooth, latency, flag, fov)` 0x3f | 0x153700 → `FUN_001bc510` | the camera at `cam` (nothing without one); field value 33 unless fov > 2; manager `+0x3cc` bit 0 |
| `EPlayCameraSpline(spline, smooth, latency, a4)` 0xcc | 0x1539a8 → `FUN_001bc890` | a camera on a spline; bit 0 |
| `EEndCameraAnim(smooth, latency, carSpace, camID)` 0x44 | 0x14bdb8 → `FUN_001bb098` | back to the chase camera; clears bits 0, 1 |
| `ELetterBoxOn(instant)` 0x68 / `ELetterBoxOff(instant)` 0x67 | 0x150dc8 / 0x150ce8 | below; bit 2 / clears bits 0..2 |
| `ESetSimRate(r)` 0x70 | 0x158d28 | `DAT_0032f2b0+0x24` = min(r, 4) |
| `EDisplayText(id, secs)` 0x2c | 0x14b8f8 | HUD text (`FUN_00245890`) |

Smooth / latency frame counts above 1 become `n × fps / 60`; a smooth count of 0 or 1 becomes 1.
When the `+0x3cc` bits reach 0 (`FUN_00206620`, also when they already were 0): programmer
event 3 cleared, `DAT_0032f2b0+0x20` = 0, and a HUD fade that is up restarts over fps/2 ticks.

**Letterbox** (`FUN_00244ea0` / `FUN_00244ec8` / `FUN_00245190`): hud `+0x2b0` state, `+0x2b4`
counter. On: instant → state 0 counter 30, else counter 0 and state 2 (+1 per update, state 0
at 30). Off: instant → state 1 (no bars; the counter is left), else state 3 (−1 per update,
state 1 at 0). Bar height `(int)(counter × 65 / 30)` (× 1.5 when `DAT_00333760+0x58` is 1), at
the top and bottom, colour 0x7f000000. The HUD draws its gauges only in state 1 [M 0x23cf88].

**Path player** (`FUN_001b1258` init, `FUN_001b14b8` update): speed `+0xbc`, step `+0xc0`,
target `+0xc4` (all 1.0 at init), delay `+0xc8`; each update the speed moves one step toward the
target (a speed of 0 at a target of 0 stops it), time += dt × speed × rate; past the duration
it wraps into the next animation (`+0x9c`, the same one at init), or stops at the end without
one. The clock is `seconds × DAT_00333ff4 × 60` (`FUN_001b0f48`), so 60 path units a second.

## 3. The race countdown [D, RACE]

Not its own machine: the start list (trigger 212) runs `EForceCarStop(1)` and stage 0; trigger
216 (radius 5, 0.4 u from the start) sets timer 5 to 5 s counting down; the stage-0 timer rules
0, 1, 6 show text 0x8ff, 0x8fe, 0x8fd (0.5 s each) as the timer passes 4, 3, 2 s; rule 7 below
1 s shows 0x227c (2 s), runs `EEndCarStop`, the rival's `EAICommand`, stage 1 and the race
timer. Run by the port: texts at frames 63, 123, 183, release + 0x227c at frame 243. The text
ids are not decoded to strings here (that they read 3, 2, 1, GO is not checked).

## 4. Which track opens with what [D, run by `driving_intro_disc.rs`]

| track | opening | at the end of the run |
|---|---|---|
| MIS01 | trigger 685: spline 0 (`path_Helicopter02`, throttle 0.71), POV (−60°, −20°), letterbox on (instant); trigger 596 near the path start slides it out (frame 10) | on the spline, POV (the sniper section); the hand-back comes later (rule 34 / trigger 741, driving-mis01-opening.md) |
| MIS11 | none (speech, objectives) | the player drives from frame 0 |
| MIS13A | none (start speed 35) | the player drives |
| MIS13B | trigger 78: spline 0 (`path_BondMaster01`, throttle 0.55, time 2%), letterbox on, camera 0; trigger 89 down the path: letterbox slides out, chase camera, POV on (0°, −42°) at frame 130 | on the spline in POV mode for the whole drive (no `EEndControlToSpline` for the player on this track) |
| MIS13C | start list: POV on; trigger 17: spline 0 (`path_nomovement`) at throttle 0 | on the spline, POV |
| MIS3 | trigger 215: `EControlToCPU`, POV (180°, 0°), speed 0, timer 1; trigger 223 (5.6 u on) then runs letterbox + camera spline 1 | CPU drives, POV; trigger 223 needs the CPU driver to move the car (not modelled) |
| MIS4 | none (start speed 24) | the player drives |
| RACE | forced stop + countdown (§3) | the player drives from frame 243 |

On the tracks where the player drives, the car stands 1.0 above the ground found under it.
MIS13B: the ground found under the start is 169 u below the marker, but trigger 78 and the
path it hands out both start at the marker's height, so a car on that ground never starts the
opening; the test puts the car at the marker. Which surface the game finds there is open.

Mid-drive scripted sections the same events run (not openings): RACE triggers 224/225 and MIS4
267/268 (the bridge jump: letterbox, camera spline, spline at sim rate 0.6, then
`EControlToPlayer` at rate 1), MIS13A 45/46 (the volcano jump), MIS3 221/255/269/287.

## 5. Inputs the caller fills

The player car's position: `IntroState::spline_position` while on a spline, the physics
otherwise. `DriveScript` takes it through `Facts`. The view does not run this layer yet
(`crates/nf-view` is outside this change).

## 6. Not proven

- [S] That `EControlToSpline`'s controller call (vtable `+0x24`, not decomposed) starts the
  driver's path player on the spline's `pt` at time 0 with the init defaults, wrapping to
  itself; and that the path player's matrix is the car's (the update writes `*param_1`).
- [S] `EEndCarStop` releasing either stop mode; what modes 1 and 3 do to the car.
- [S] One letterbox update per 60 Hz frame (the HUD draw rate) and `DAT_00333ff4` = 1.
- The quaternion blend's weights (the VU field selects are hidden in the decompile); cubic
  channels (one on disc) are not sampled; the base matrix `FUN_002cbb88` applies (identity, or
  the 90° turn at 0x381f70) is not applied to positions.
- The reader of `FUN_001bd198`'s flag (director `+0x354` → `+0x10`); the camera director's
  own motion for each camera type (camera.ini types Fixed, Spline, AutoDrive, … are not ported);
  `EPlayCarCameraAnim` (unused near any start) and the HUD fade's own update.
- MIS3's and MIS01's later steps depend on the CPU driver and the AI, which this layer does not
  move.
