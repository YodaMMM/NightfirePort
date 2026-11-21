# Driving: the player chase camera

Port: `crates/nf-game/src/driving/camera.rs` (`ChaseCamera`, `HeliParams`, `ArmParams`, `CarPose`).
Disc check: `crates/nf-view/tests/driving_camera_ini_live.rs`.

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

On RACE (car at z 168.8 facing -z, barrier 4.2-5.0 behind) the target eye is z 173.8, at the
barrier's back face; with the barrier in the cast set the casts hit it about 4.2 back and the
eye lands near z 172.7, in front of it.

## Not ported / unproven

- `FUN_001b8ae0`: the eye is also pushed out of other vehicles' spheres
  (kCameraObjectSphereRad 2.598); not ported.
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
  traced past `FUN_002166f8`. Whether the RACE POS2_Cautionguard barrier is in that set is
  unproven.
