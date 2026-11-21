# Driving: the player chase camera

Port: `crates/nf-game/src/driving/camera.rs` (`ChaseCamera`, `HeliParams`, `ArmParams`, `CarPose`,
`CamWorld`, `Obstacle`, `push_out`).
Disc checks: `crates/nf-view/tests/driving_camera_ini_live.rs`, `crates/nf-view/tests/drive_cam_collide_disc.rs`.

Tags: [M] DRIVING.ELF decompile/disassembly, [D] disc data. Nothing here is a live capture.

## Where the tuning comes from

`data\render\camera.ini` sits in all eight `/DRIVING/*.VIV` archives (refpack, 4322 bytes packed,
one identical file everywhere [D]). The loader `FUN_001a49e0` [M] walks its sections:

- Section names containing `Heli` become camera type 2 (the chase camera), `Bumper`, `Fixed`,
  `Spline`, `Ellipse`, `Dashboard` and others their own types. `:Arm<k>` sections are arms of the
  section before the colon. `Global` holds the `k...` constants.
- A section is kept only when its `car =` list names the player car (`FUN_001a6cf8`; no list
  means any car). `AnyCar...` heli cams are kept only while no car-specific heli cam was taken,
  hence the ini comment that AnyCarHeliCam0 "has to be the last car specific HeliCam".
- The first kept section with `defaultCamera = 1` is the start camera (`DAT_00333a12`).

Two sets are tabled in the port:

| key | AnyCarHeliCam0 | VanquishHeliCam0 |
|---|---|---|
| Min_Rate / Max_Rate | 0.06 / 0.25 | same |
| Speed_Rate_Diff | 0.00166667 | same |
| Fallback_Factor / Max_Fallback | 0.04625 / 1.22 | same |
| Vertigo_Lerp, MaxVertigoUphill/Downhill | 0.5, 1, 1 | same |
| lookBack | 1 | 1 |
| defaultFov | (absent, loader default 33.0) | same |
| Arm0 Heli_Sideways / Height / Distance | 0 / 0.2 / -5.0 | 0 / 0.85 / -5.10 |
| Arm0 Anchor_X / Y / Z | 0 / 0.95 / 0 | 0 / 0.3 / 0 |

`Loose_Arm_*`, `Tumble_Arm_Scale` and the `Anchor_Slide_*` keys are present in the file but
the heli update does not read them on the path below (`Loose_Arm_*` is not even a string in
the ELF [M]).

The RACE car renders as `paradis_car` and no `car =` list names it [D], so RACE gets
AnyCarHeliCam0, provided the name the game compares is that one (the compared string comes
through a vtable call and is unproven).

## The update (`FUN_001b2b48`, the type-2 member pointer at `DAT_0037da88`) [M]

Normal path: no mode transition (+0x12c <= 0), no flags at +0xe0, not paused
(`DAT_00339f44 != 3`), the target is a vehicle.

1. Arm = (Heli_Sideways, Heli_Height, Heli_Distance). If look-back is latched and the mode has
   `lookBack`, distance = -distance. Otherwise distance -= min(|v| * Fallback_Factor,
   Max_Fallback) (R5900 `min.s`, 0x1b2c70). |v| is the 3-D velocity length (`FUN_001d5888`).
2. Vertigo (non-rigid arms): `s += (fwd.y * K - s) * Vertigo_Lerp`, K = MaxVertigoDownhill when
   fwd.y < 0 else MaxVertigoUphill. With c = sqrt(1 - s^2): y' = s*z + c*y, z' = c*z - s*y.
3. To world on the car's flattened heading: f = normalize(fwd.x, 0, fwd.z), r = (-f.z, 0, f.x);
   target = r*x + (0, y', 0) + f*z' (0x1b2e30-0x1b2e70).
4. Anchor (`FUN_001d6498`): a += (R * Anchor - a) * 0.05; anchor point = car.pos + a.
5. Follow rate = clamp(|v| * (dt*60) * Speed_Rate_Diff, Min_Rate, Max_Rate) (dt = 1/60, so the
   factor is 1). The kept offset is rescaled to |target| and then
   offset += (target - offset) * rate. Eye = anchor + offset.
6. If `checkCollisions` (default 1): `FUN_001b9198(cam, eye, car.pos, 1)`, below.
7. Up target (+0x2c0) moves only when upRate > 0 (0 for land cars). Look matrix
   (`FUN_001d5700`): forward = normalize(anchor + up*lookUp - eye), right = normalize(up x
   forward), up' = forward x right.
8. FOV (`FUN_001b8740` mode 1): +0xa4 steps toward the mode's defaultFov (33.0) by
   kZoomIncSpeed (0.9) * +0x2b4 per update (`FUN_001d6720`). The projection gets 2 * +0xa4
   (`FUN_001d4db0`) and its x scale is 1/tan(that/2) (`FUN_00295910`), so 33.0 is a 66 degree
   wide view.

At rest on flat ground the AnyCar eye is 5.0 behind the car and 1.15 above its origin, looking
at a point 0.95 above it.

## World pull-in (`FUN_001b9198`, param_4 = 1) [M]

Origin o = car position, d = eye - o, b = o + d * 1.075. Three segment casts
(`FUN_002127f0` -> `FUN_002166f8`, a segment against the gathered collision triangles), ends:

- b - up*0.5
- b + up*0.5 - right*0.4
- b + up*0.5 + right*0.4

(up and right are the camera rows from the previous update.) For each hit at distance h from o
with h > 1.1: h = max(h, 2.5), scale = min(scale, h / |end - o|); if h - 2.5 < 1.5 the dip below
is disabled. If scale < 1: eye = o + d*scale + up*0.5*(1 - scale). With noise (`noiseAmount` > 0,
no land car) the floor is 2.0 and the lift 0.1.

The dip (when no hit came within 1.5 of the floor): with L = |eye - o|, cast
eye + carFwd*L*0.125 -> eye + carFwd*L*0.975. A hit is remembered (`DAT_00333a40`) and adds 1 to a
counter (`DAT_00333a38`, max 80). While the counter is above 0, the remembered point is tested
against triangle (eye, cast end, o) (`FUN_001b9020`, projected on the axis of the normal's
largest signed component, `FUN_001b8f80`): inside adds 1, outside takes 1. Then
eye += up * counter * -0.0125 (down to 1.0 lower at 80).

Last, `FUN_001b8ae0` pushes the eye out of bodies (below).

## What the casts see [M]

The camera keeps a collision gatherer at +0x35c (`FUN_002117e8`, flags 0xc, radius
|eye - car| * 0.9125 around car + (eye - car) * 0.5375). It is gathered again when
`FUN_002126d0` finds the new sphere (radius (|d| * 1.075 + 0.5) / 2) no longer inside it.

- Gathering (`FUN_00211ed0`): flag 8 runs `FUN_00217588(.., 1)`, flag 4 runs
  `FUN_00217588(.., 0)` then `FUN_00219a50`. `FUN_00217588` boxes the sphere on the `CGrd` grid
  (`DAT_0033a5c8`, `FUN_0021cf30` / `FUN_0021cf80`) and `FUN_00217228` walks each cell's
  **kind-0** entries: the cell's u16 index list (byte +4 count, u16 +8 offset) and its linked
  nodes whose kind word is 0. Every entry indexes the `ci` item array (world +8, 0x40-byte
  items), deduplicated by the +0x10 stamp and kept when the 2-D distance to the centre is under
  radius + item +0x3c. Kind 1 entries are triggers (`FUN_0022cad8` moves trigger entries with
  `FUN_0021cad0(.., 1, index)`), not collision.
- Flag 8 keeps the items' `ca` triangle groups (`FUN_00216ec0`). Flag 4 keeps their `ca` wall
  edges (`ca` u16 +4 count, u16 +2 offset, 0x20-byte entries: two end points and 1/length; a 2-D
  distance and a height window; class byte +0xd against the world +0x2c mask).
- The cast (`FUN_002166f8`): the edges first (`FUN_0021a248` -> `FUN_00219328`, horizontal hit
  normal), then the triangles (`FUN_00232180` / `FUN_00232638` -> `FUN_00214c08`: per group a
  centre/radius test against the segment, then `FUN_00213d68` on the strip in a frame built from
  the segment, both windings accepted, a triangle whose class byte +0x2d meets the world's +0x2c
  mask skipped). `FUN_00216d30` keeps the nearer of the two hits.

So the camera ray meets `CDat` collision only, walls included (there is no slope filter). It
does not see scene objects (`RSceneObj` props are not `ci` items), smackables, or cars.

Bodies are handled after the casts, by `FUN_001b8ae0` -> `FUN_001b8d08`:

- each vehicle in the sim list `DAT_00339f5c..60` except the camera's target (+0x118), gated by
  |eye - pos|^2 <= |size|^2 * kCameraObjectSphereRad, half-extents size * kCameraObjectRadiusEx
  (size = the vehicle's +0x5c object's +0x4c0 vector; the code does not name it);
- each live body of the 0x60-slot physics table (`0x339190`, `DAT_003394c0`) whose type byte
  +0x1c is 8, gated by |eye - pos|^2 <= radius * kCameraObjectSphereRad (radius +0x2c, compared
  unsquared), half-extents kCameraObjectRadiusEx * radius.

`FUN_001b8d08` takes the eye into the body's frame (its rows). When the eye is inside the
ellipsoid (every |local| under its half-extent and the normalised sum under 1) it sets local y
to sqrt((1 - x^2/rx^2 - z^2/rz^2) * ry^2), goes back to world, makes the world offset's y
positive and adds the position: the eye goes over the top. camera.ini `[Global]`:
kCameraObjectRadiusEx 1.5 / 1.5 / 1.5, kCameraObjectSphereRad 2.598 [D]; loader defaults 1.675 /
1.925 / 1.325 and 2.875 [M FUN_001a49e0 :801-804].

Near plane: the `RCamera` ctor `FUN_001d40f0` sets near 0.17 (+0x20) and far 3000 (+0x2c), and
`FUN_001d4878` builds the projection from them (`FUN_00295910`). No write that changes the near
plane while driving was found (`FUN_001d4db0` can, but no caller of it was found). There is no
step that pushes the eye off a surface by the near distance: the only limits are the 2.5 minimum
distance and the 1.1 ignore radius above.

## The RACE start [M + D]

- The start trigger (Trgr 212, at the car's start) holds `EStartMission`, `EForceCarStop`,
  `EChangeStage` and two `EStreamEvent`s (creator table 0x36fda0 -> constructor -> vtable ->
  type name). Trigger 216 beside it holds `EChangeStage`, `ESetTimer`, `EStartTimer` and
  `ECallStage`. Neither holds a camera event (`EPlayCameraAnim`, `EPlayCameraSpline`,
  `ECinematicCamera`, `ESetAutoDriveCamOn`, `EChangeCarCameraView`); RACE's only camera events
  sit on triggers 218, 224 and 225, hundreds of units from the start. So, as far as the
  triggers go, the start view is the chase camera. What the stages run (the `EChangeStage` /
  `ECallStage` targets) is not traced.
- The four `POS2_Cautionguard` rails behind the start (items 3166-3169) are scene objects, not
  `CDat` items [D: no collision triangle inside their boxes]. The game's casts cannot meet them,
  and nothing in the camera code moves the eye off them.
- In the port, at rest after 240 ticks, the eye is at (-2.65, 0.76, 173.78). It is outside every
  scene object's box, level with rail 3168 and about 0.05 behind its back face (local depth 0.29
  against a half depth of 0.25); the rail's far face is about 0.55 in front of the eye. The
  traced camera code puts the eye there for this car pose, so the full-screen rail is not a
  camera collision miss.

What that rail should look like from there is a draw question and is not settled here. With
the game's 0.17 near plane the back face (0.05 away) is partly clipped, and whether the game
culls the rail's back-facing inner faces is not traced (the port draws every face). The car's
rest height also moves the eye: the game spawns the car at ground + 1.0 (`FUN_001fee68`), while
the port settles the car origin on the ground (-0.39 here). The game's settled body height is
not traced.

## Not ported / unproven

- The `ca` wall edges the cast also tests: `CamWorld` holds triangles only (the port does not
  extract the edge list from the disc). The exact along-segment window of `FUN_00213d68` (its
  -1.0 floor and the +0.5 its caller adds) is not reproduced; `CamWorld::cast` is the plain
  nearest segment-triangle hit, gathered on a 16-unit grid instead of the game's sphere.
- `CamWorld` is not wired into the driving view yet. The view's ray still uses its ground grid,
  which drops every triangle with |n.y| < 0.25 and every class-4 triangle, so walls do not stop
  the eye there. The hook-up is to build a `CamWorld` from all `CDat` triangles and pass its
  `cast` as the ray.
- The view does not feed `update_full` any obstacles: other cars' size vectors (+0x4c0) are not
  traced, and the type-8 bodies are not identified.
- The rigid arm (`rigidArm = 1`: vanquishsub, missile cam), the noise term, mode transitions
  (+0x12c / +0x27c / `armTransition`), the +0x134 fixed rate 0.1, look-back's release rules and
  the other camera types (Bumper, Dashboard, Tumble, Spline, AutoDrive) are not ported.
- The spawn-time camera state: the port starts settled on the target (offset = target, anchor
  smoothed, up = world up). The game's initial +0x2c0 / +0x2f0 values are unproven.
- The FOV step rate (+0x2b4) is unproven; the port outputs defaultFov directly.
- The update rate: the port steps once per 1/60 sim tick. That the game calls the heli update
  once per 60 Hz frame is assumed from the dt*60 factor, not traced to the caller (the member
  pointer's call site was not found).
- The car's row 0 sign is assumed right = up x forward; it only reaches Anchor_X, which is 0 on
  both tabled cams.
- Which world geometry the casts see (static collision only, or barriers/props too) is not
  traced past `FUN_002166f8`. The RACE POS2_Cautionguard rails are smackable scene objects that
  nothing hides at the start (docs/formats/trackobj.md §2); whether the cast sees scene objects is
  unproven.
- The world +0x2c class mask's value (runtime state; `CamWorld::class_mask` defaults to 0).
