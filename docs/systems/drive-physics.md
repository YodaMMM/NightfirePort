# Drive physics — the DRIVING.ELF car physics step (CARP)

Status: first port pass. Evidence tags: [M] code (function + address,
DRIVING.ELF unless named; decompile under `work/decomp/DRIVING.ELF/`), [D] disc, [L] live/trace,
[S] suggested. File offset = EE − 0x107080 (docs/atlas/driving.md build identity). The per-frame
loop, scheduler and 60 Hz tick are docs/atlas/driving-elf.md §1 (the timestep is `1/FUN_002d6b90()`
= 1/60 NTSC, stored at sim+0xdc0 = `DAT_00339f50` [M `FUN_001feda0`:9-12]).

## 1. Where the step lives (the per-tick contract)

The frame tick `FUN_00159d10` 0x159d10 [M] runs, in order: prune (0x200600) → the double-buffer
swap (`FUN_001ff2e8` — the instance/static arrays ping-pong, sim+4 selects) → tick++ (0x1fef20,
sim+0xdc4) → **the physics step `FUN_001ff3d8` 0x1ff3d8** → AI/weapons/roads → `+= 1/60`
accumulator → swap back [M :9-33]. The dispatch from the world list to `FUN_00159d10` rides
vtable 0x3704c8 slot 1 — the one link not pinned statically (atlas live-check L-3) [S].

`FUN_001ff3d8` (guarded by sim+0xdb4 != 3) [M]:

| pass | function | role |
|---|---|---|
| A | `FUN_001fbbd8` → response `FUN_001f9d98` | car↔car broadphase (radius inst+0x7c) → OBB → impulse |
| B | `FUN_001fc2f0` (+`FUN_001fb318`/`FUN_001fade8`/`FUN_001fc198`) | car↔world: 8 hull points (class+0x280+sel·0x80) extrapolated by `vel·dt` (CCD), quad/ray tests, response + `COLLISION_INFO` |
| C | `FUN_001f9500`, `FUN_001f8ff0` | uw-level pass, contact flush |
| D | `FUN_001f8088` + `FUN_001f89e0` → VU0 macro, flush `FUN_001f8cf0` | **integration**: spin ×= 0.98 (`NATURAL_ANGULAR_DAMPING`), then `_vcallms` into `.vutext` @0x324800 — pos/orientation/vel update; sleep check vs `SLEEP_VEL` |
| E | `FUN_001f7c60` | zero accumulators, apply gravity, ground damping |
| F | `FUN_001fd798` | static bodies: `pos += vel·dt` |
| — | tree walk sim+0xe68 → `vtbl+0x2c(obj)` = `FUN_00188200` | per-car controller → engine/tyres (forces land for the NEXT tick) |

`FUN_00188200` 0x188200 dispatches `FUN_0018cef8` 0x18cef8 (engine/gears/steering) and
`FUN_00190f88` 0x190f88 (tyre/lateral forces + world-collider contacts). Both consume
`PVehicleParams` at actor+0x23c.

## 2. The tuning: .tun text + per-car .atr

**The .tun files are ASCII `KEY=VALUE` lines (CRLF, no sections)** — no magic; the "fogS/Farp
magic" note in docs/formats/driving-viv.md misread the text keys `fogSTART=`/`Farplanedist=`
[D]. Loader [M]: namespace enter `FUN_00143de8` (class `DTuningDBMgr`, RTTI `12DTuningDBMgr`
@0x36d958); path `sprintf("%s%s\%s.tun", "data\tuning\", ns, name)` (fmt 0x36da48, prefix
0x36da58 — strings read from the ELF) with the `\default.tun` fallback (0x36da68); per-var
lookup `FUN_001426a8` — substring find (next char `' '`/`'='`, skip `=#([`), then
sscanf `%d/%u/%f/%s` (0x36cdb8-0x36cdd0); **no match keeps the compiled-in value**.

The Physics namespaces register into fixed globals:

- **Physics:Rigid** — 21 vars registered by `FUN_001f6600` → globals 0x3390d0-0x339164.
  `GRAVITY` 0x3390d0 = −34.0, `BODGE_PLAYER_GRAVITY` 0x3390d4 = −13.0, `SLEEP_VEL` 0x339100 =
  5.0, `GROUND_FRICTION_COEFF` 0x3390e8 = 0.05, `NATURAL_ANGULAR_DAMPING` 0x3390fc = 0.995,
  `M_LIMIT` 0x339108 = 100.0, `AM_LIMIT` 0x339104 = 10.0, `DEPTH_FORCE_SCALE` 0x339118 = 4.5,
  `SPEED_SCALE` 0x339114 = 0.25, `SPEED_FORCE_LIMIT` 0x33910c = 10.0, `TOTAL_FORCE_LIMIT`
  0x33911c = 40.0, `MICRO_THRESHOLD` 10.0, `GROUND_RESTITUTION` 0.5, `CAR_COLLIDE_COG_SCALE`
  0.4, `BUILDING_*` 0x339134-0x339148, `PF_SCALE` 10.0 [M .data initials; identical to the
  shipped `Physics\Rigid\default.tun` [D]]. The shipped .tun trio is byte-identical in all 8
  mission VIVs [D; an extracted set, not committed].
- **Physics:Physical** — 21 vars by `FUN_00185668` → 0x331970-0x3319d4: `PAD_DEAD_ZONE` 0.2,
  `SKID_AUDIO_SCALE` 0x3319a0 = 0.15, `ROLLING_RESISTANCE` 0x3319a4 = 0.1, `TWO_WHEEL_*`,
  `TYRE_DAMAGE_RADIUS` 1.25, `BASE_FRICTION_MASS` 0x331974 = 8.0, `SHRED_DRAG` 0x331980 = 2.0,
  `MAX_WHEEL_SPIN_RATE(_AI)` 0x3319c0/c4 = 0.05, `POST_BRAKE_ACCEL_*` 0x3319c8-d0 [M].
- **Physics:Friction** — `friction[WSurface_k*]` / `lateralLoss[WSurface_k*]` arrays at
  0x3319d8 / 0x331a18, 13 surfaces each (kNODRIVE, kPAVED, kGRAVEL, kGRASS, kCOBBLE, kDIRT,
  kWATER, kWOOD, kICE, kSNOW, kPAVED_ROUGH, kPAVED_GRATE, kRAILROAD), compiled-in defaults all
  1.0 [M .data]. ⚠ The shipped `Friction\default.tun` keys read
  `frictionCoefficients[WSurface_kGRAVEL]=0.9` etc., which **cannot** match the registered
  `friction[k*]` names under `FUN_001790f8`'s substring rule — no normalizer found — so the
  file values are likely dead in this build and the code defaults (all 1.0) apply [S; live
  check requested in the report].

**Per-car physics is not a .tun — it is `data\sim\attrib\pvehicle\<car>.atr`** [D]: INI-style
`[section]`, typed suffixes `.f/.i/.u/.s/.v` (suffixless keys parse as float), `//` comments.
The player's Aston Martin is **`vanquish`** (MIS01.VIV; 11 pvehicle files per mission;
`attrib.dir` lists the full set incl. cobra/snowrover/jungletank/vanquishsub). Schema +
defaults registered by `FUN_001968f8` 0x1968f8 on "pvehicle"/"CarPhysics" [M] — offsets:
`IS_4X4`+0x00, `SPEED_CUTOFF_REVERSE`+0x08 (200), `SPEED_CUTOFF_RATE_REVERSE`+0x0c (1000),
`MAXACC`+0x14 (30), `MAXREVACC`+0x18 (30), `MAXBRAKE`+0x1c (30), `MAXSTEERING`+0x20 (0.2),
`MODSTEERSPEED`+0x24 (50), `MINSTEER`+0x28, `FRIC_MOD_SPEED/RANGE/MIN`+0x2c/30/34,
`COUNTER_STEER_SCALE(_LIMIT)`+0xb0/b4, `FRICTIONLIMITFRONT/REAR`+0x40/44 (50),
`TYREGRIPFACTOR`+0x48 (16), `SPRING_STIFFNESS_FRONT/REAR`+0x50/54 (1000),
`SPRING_DAMPING_FRONT/REAR`+0x58/5c (1000), `SPRING_REST_LENGTH`+0x60 (2),
`SPRING_COMPRESSION_LIMIT(_DRAW)`+0x64/68, `WHEEL_RADIUS`+0x6c (1), `BRAKING_FRICTION`+0x70
(3), `ACCEL/COAST/BRAKE/REVERSE/HANDBRAKE/REV_HANDBRAKE_FRW_RATIO`+0x74-0x88,
`FRONT/REAR_SLIPPING_SCALE`+0x8c/90, `SLIPPING_RANGE`+0x94 (10), `HANDBRAKE_FORCE`+0x98 (30),
`TOTAL_HANDBRAKE_FRICTION_SCALE`+0x9c, `WHEELSPINSCALE`+0xa4, `WS_SPEED`+0xa8 (20),
`WS_ACCEL`+0xac (10), `IS_SUB`+0xbc, `SUB_PHYSICS`+0xc0, `NO_WORLD_COLLISIONS`+0xc4,
`GEAR_LIMITR/1..5`+0xd0-0xe4 (100), `GEAR_RATIOR/1..5`+0xe8-0xfc (1.0), `MASS` (2000) [M —
the FUN_0016e590/0016ec90 registration call list].

`vanquish.atr` values used by the port ([D]): MASS 1660, MAXACC 3, MAXREVACC 2.4, MAXBRAKE
3.15, MAXSTEERING 0.09, MODSTEERSPEED 10.0, MINSTEER 0.425, GEAR_LIMITR 10 / LIMIT1..5
18/29/50/54/69, GEAR_RATIOR 1.0 / RATIO1..5 0.65/0.59/0.52/0.45/0.44, FRICTIONLIMITFRONT/REAR
19, TYREGRIPFACTOR 8, SPRING_STIFFNESS 80/98, SPRING_DAMPING 400/350, SPRING_REST_LENGTH 0.24,
WHEEL_RADIUS 0.365, BRAKING_FRICTION 0.3, FRONT/REAR_SLIPPING_SCALE 0.421/0.425,
SLIPPING_RANGE 25, SPEED_CUTOFF_REVERSE 10, SPEED_CUTOFF_RATE_REVERSE 0.075,
COUNTER_STEER_SCALE 1.72 (limit 2.0), HANDBRAKE_FORCE 0.1,
TOTAL_HANDBRAKE_FRICTION_SCALE 0.15, ACCEL/COAST/BRAKE/REVERSE/HANDBRAKE_FRW
0.25/0.35/0.65/0.6/0.9, BOOST_EXTRA_ACC 1.75 (BOOST_TIME 190), CAR_WHEEL_ZF/ZR −0.894/+1.006
(wheelbase 1.9 m), NOSE_JUMP_ANGLE 0.03, SLOPE_SCALE 0.5.

## 3. The engine / gears / steering (`FUN_0018cef8` + `FUN_0018aa88`)

**RPM + auto-shift `FUN_0018aa88` 0x18aa88** [M]: `|v|` = |instance+0x20 velocity| (VU0
`vel·vel` → sqrt). If `SUB_PHYSICS`(+0xc0) == 1: rpm = |v|·100. Else if the car is not in the
driving state (+0x299 == 0): **rpm = throttle(+0x250) · 7000** (revving in neutral). Else the
automatic gearbox (when +0x258 == 0):

```
gear := 1
if GEAR_LIMIT1(+0xd4) <= |v|:
    loop { gear++; if gear > 4 break;
           if |v| < GEAR_LIMIT[gear] (+0xd0 + gear·4) break; }
if gear < last_tick_gear(+0x2d9) && shift_inhibit(+0x2d7) != 0: gear := last_tick_gear
rpm := |v| · 6000 / GEAR_LIMIT[gear]        // the limit read last in the walk
rpm += DAT_003319ac · actor+0x238           // + wheelspin extra
```

So gear g holds |v| ∈ [LIMIT(g−1), LIMIT(g)) (LIMIT0 := LIMIT1 gates gear 1's top; gear 5 =
|v| ≥ LIMIT4). For vanquish the shift points are 18/29/50/54 m/s and gear 5 runs to the 69
limit. (The +0x2d7 inhibit is set to 0xf on every gear change in `FUN_0018cef8`:410-413; its
decay site is not pinned [S].)

**Drive force `FUN_0018cef8`** [M]: `engine_accel(+0x41c)` starts at `MAXACC`(+0x14) (or a
damage/boost variant), then line 414-416 `engine_accel ·= GEAR_RATIO[gear]` (+0xe8 + gear·4;
gear 0 = reverse ratio). `+0x420 := MAXREVACC`(+0x18), decayed in reverse (below). Then
(:417-431):

```
drive_accel(+0x1dc) = boost(+0x428) + throttle · engine_accel − brake(+0x254) · MAXBRAKE(+0x1c)
```

Reverse state (+0x29c set, :401-407): `cut = speed − SPEED_CUTOFF_REVERSE(+0x08)`; if cut > 0:
`reverse_accel = MAXREVACC − cut · SPEED_CUTOFF_RATE_REVERSE(+0x0c)`, floored at 0; the brake
term uses the decayed value.

**Steering (:458-474)** [M]:

```
steer_scale = 1.0                                  (speed <= MODSTEERSPEED)
steer_scale = 1 − min(1, (speed − MODSTEERSPEED)/25)   (above)
steer_scale = max(steer_scale, MINSTEER(+0x28))
slip(+0x28c) = steer_input(+0x240) · MAXSTEERING(+0x20) · steer_scale
```

For vanquish: full 0.09 rad at ≤10 m/s, floored at 0.09·0.425 = 0.03825 rad from 35 m/s.
Reverse flips the steer force sign (`FUN_00190f88`:428-433 [M]). The steering slew
`(throttle − brake)·7.0` blends drive/reverse states [M 0x190f88:344, 404].

## 4. Tyre forces and grip (`FUN_00190f88`)

[M 0x190f88] — the numbers the port keeps:

- `speed(+0x288) = |vel|` (:384-394).
- Low-speed grip boost (:395-400): `front = (1 − min(1, speed/15))·2.5 + 1.0` (3.5 at rest →
  1.0 at 15 m/s); the slip-angle factor `(1 − min(1, speed/15))·3.0 + 1.0` (4.0 → 1.0).
- The coast/handbrake drift path (:401-416): grounded with no throttle → both factors clamp to
  2.0 and the slip terms scale by `min(speed,20)·0.05` (0 at rest → 1.0 at 20 m/s).
- **The effective slip angle** (:464-474): `slip = term · factor · (player ? 0.125 : 0.222)`
  clamped ±0.222 → actor+0x28c (overwrites the steer angle — the tyre model's own slip).
- Low-speed yaw assist for AI cars (:476-486): `slip·(5 − |speed|)·4` added ± to the axle
  accumulators (type != 0, speed < 5, `FUN_00195a30() > 0`).
- **The axle lateral forces** (:490-508): `sincos(−slip)` rotates the axle velocity into the
  wheel frame; `front_lateral = −(rotated x) · 5.0` **clamped ±12.5**; the rear mirrors it
  (the ±9.375-12.0 rear band per the response limits) — units are the force-API's
  accelerations (see §6).
- ~~The 18 world colliders @0x331a60 (stride 0x50): spring-damper `F = fps · Δv · compression`
  (:870-975) — the suspension consumer; constants per collider~~ **Corrected (G-056):** the
  table at 0x331a60 is 19 invisible boundary sheets (4 corners + normal + thickness), and
  :870-975 is their response: `C = n·s·(fps·v_n − (f·n)/m)`, applied as `C·s` only when
  `C·n > 0`, with `s = max((t − d)/t, 0.1)` and fps the integer at 0x339f4c, plus a scrape
  torque [M 0x192130-0x1923bc]. Note this function, `FUN_00190f88`, is the pass the controller
  `FUN_00188200` runs only for `SUB_PHYSICS` vehicles (subs, ultralights, plane, subcopter
  [D]) [M 0x188bf8-0x188c04] — so the grip/lateral constants cited from it in this section
  may not be the land car's; the land car (SUB_PHYSICS 0) takes `FUN_0018cef8` or
  `FUN_0018ecb8` (unresolved here). Full decode: docs/systems/driving-world-bounds.md; port:
  `crates/nf-game/src/driving/bounds.rs`.
- `ROLLING_RESISTANCE` 0x3319a4 and the friction/lateralLoss surface tables feed the same
  accumulators [M registration 0x185668].

## 5. Gravity and the per-tick reset (`FUN_001f7c60`)

[M 0x1f7c60] — the accumulators zero (+0x60..+0x78, :55-60); the class latches tick (+0x4de,
+0x4ff := +0x4fe, :64-73); then:

- **The working gravity `DAT_003390d8` = `BODGE_PLAYER_GRAVITY` (−13.0) for cars** — the type-1
  branch (:136-144) sets it back to `GRAVITY` (−34.0) only when the car's model name is
  `jungle_truck` (string 0x396588 [D]); `paradis_car` (0x396578) takes the same type-1 path
  (:150-156). The vector `(0, gravity, 0)` applies at :168 through the force API.
- **Ground damping** (:174-184): `vel −= vel · (4 · GROUND_FRICTION_COEFF)` = `vel · 0.8` per
  tick (0.05·4 = 0.2) — but the gate (:171-173) reads instance flags/kind bytes `+0x6c > 2 &&
  +0x6d > 3`, and the car kinds are 1-3 (`FUN_001f6e98` stores param_4 ∈ {1,2,3} at +0x6d [M]),
  so the branch reads as the PROP path. Applied to a car it would cap the speed at
  `accel·dt/0.2` ≈ 0.16 m/s — impossible, so **cars do not take this damping**; the car's
  coast-down is `ROLLING_RESISTANCE` (0.1, Physical 0x3319a4) in the tyre pass [S for the
  exact form; live check requested]. The port keeps the 0.8 factor as
  `Rigid::ground_friction_coeff`-derived data (asserted) but applies only the rolling
  resistance to cars.
- :185-190 `+0x40 = vel · mass` — a momentum mirror for the response pass.

## 6. The force API and the integration

**`FUN_001f87d8`** (apply force) [M]: `acc(+0x60) += F · inst+0x4c` and wakes the instance
(state := 2 at +0x6e, clears a +0x7c bit) unless dead (+0x6e == 3). `FUN_001f8888` is the
torque twin (+0x70). **inst+0x4c = the MASS attribute** [M `FUN_001f6e98`:114 = param_1, fed
from the `MASS` attr read in the ctor `FUN_001862c0`:249-257] — so the accumulators hold
`accel · mass` and the VU0 integration divides by mass: **the force API works in
accelerations (m/s²)**; collision impulses enter through the same path and land as Δv [S for
the VU0-side division — `.vutext` @0x324800 not lifted yet].

The integration itself (`FUN_00189e0` → `_vcallms 0x568`, flush `FUN_001f8cf0`) runs as VU0
microcode: pos/orientation/vel update with the M_LIMIT/AM_LIMIT clamps [M :141-164 for the
setup; the macro body is phase-2 work]. The port models it as semi-implicit Euler
(`vel += acc·dt; pos += vel·dt; yaw += yaw_rate·dt`) with the documented clamps [S], and the
report asks for the live measurements that pin the macro's exact form.

## 7. The port's model (crates/nf-game/src/driving.rs)

- `Tuning` — the .tun text parser with the game's substring-lookup semantics (§2); `Rigid` /
  `Physical` / `Friction` structs carrying the compiled-in defaults [M] with from-tun override.
- `CarParams` — the pvehicle schema with the game defaults [M] and `vanquish()` = the shipped
  .atr values [D]; `parse_atr` reads the INI form.
- `Car` + `CarInput` + `FlatWorld` — the headless step in the game's order (§1): integrate the
  accumulated accel → reset → gravity + ground damping → engine/gears/steer → tyre lateral →
  accumulate. The flat surface holds the ride height, kills vy, and scales grip by the surface
  friction entry.
- Tests assert the [M]/[D] numbers: shift points 18/29/50/54, rpm = |v|·6000/LIMIT[g], neutral
  rev = throttle·7000, drive = MAXACC·RATIO[g] (vanquish 3·0.65 = 1.95 in gear 1), brake −3.15,
  steer 0.09 → 0.03825 floor, gravity −13/−34, the reverse decay 2.4 − cut·0.075, the lateral
  clamp 12.5, the grip boost curve 3.5→1.0 at 15 m/s, ROLLING_RESISTANCE 0.1, and the
  parsers round-tripping the shipped text forms.

## 8. The SUB_PHYSICS pass (`FUN_00190f88`) and the rigid integration

Port: `crates/nf-game/src/driving/sub_physics.rs`; tests `crates/nf-game/tests/drive_sub_physics.rs`
(unit) and `drive_sub_physics_disc.rs` (NF_ISO).

**Who runs it.** `FUN_00188200` picks one pass per vehicle per tick [M 0x188bf8-0x188c60]:
SUB_PHYSICS (attr +0xc0) != 0 → `FUN_00190f88`; else IS_SNOWMOBILE (+0xc8) → `FUN_00190510`;
else the player (+0x2c8 == 0) → `FUN_0018cef8`, an AI → `FUN_0018ecb8`. A path-driven vehicle
(the vtbl+0x5c object test at 0x188b70) takes `FUN_00192498`, which returns at once for
SUB_PHYSICS [M 0x192498:86]. The attr names behind the offsets are the schema strings
[M `FUN_001968f8`:261-277 → 0x37b238 IS_SUB +0xbc, 0x37b240 IS_SNOWMOBILE +0xc8, 0x37b250
NO_WORLD_COLLISIONS +0xc4, 0x37b268 SUB_PHYSICS +0xc0, 0x37b278 IS_BOAT +0xcc]. `IS_FLYING` (in
the ultralight files) is not registered, so the game ignores it.

**Which vehicles** [D, the decompressed pvehicle files]: SUB_PHYSICS=1 in vanquishsub (MIS11,
also IS_SUB=1), ultralight (MIS13A, MIS13B), ultralightbig (MIS13C), and the other subs, the
plane and the subcopter. The patrol boat (MIS13B/C) has IS_BOAT=1 and no SUB_PHYSICS: it is a
land-pass vehicle (IS_BOAT makes `FUN_001f7c60` scale its gravity by 0.2 and add a −v horizontal
drag). The sub and the ultralights run the same code: nothing in `FUN_00190f88` reads IS_SUB.

**No gravity, no buoyancy.** `FUN_001f7c60` skips the gravity force for a body whose attr
SUB_PHYSICS == 1 [M 0x1f7c60, the `+0xc0 == 1` jump past `FUN_001f87d8(gravity)`]. The pass has
no water surface, no buoyancy and no dive or surface state: the craft moves in 3-D under thrust
along its nose and the pitch stick. The only vertical-only term is a position bob. The
horizontal boundary sheets (driving-world-bounds.md; normal (0, −1, 0), so the allowed side is
below) act as ceilings: sheet 0 at y 78.44 covers the MIS11 start (71.3, 2.2, 41.0) [M table +
D start].

**The pass, in order** [M 0x190f88; actor offsets; angles in TURNS: `FUN_002cbd50` /
`FUN_002cbeb0` are sin/cos(2π·x) by a 9th-order polynomial]:

1. Reverse latch: brake == 0 → counter +0x29a = 30; else local forward speed ≥ 2.5 → counter 0;
   counter > 14 → reversing (+0x29c) when forward ≤ 0; else counter+1, and forward < 0.5 zeroes
   the stored brake.
2. Player (+0x2c8 == 0): steer³, pitch⁵; a profile flag (`DAT_003339bc[profile·0x1c+7]`,
   meaning unproven) and the +0x259 pad byte choose the force frame: the body rows, or (byte
   set, flag clear) a roll-free frame x = norm(up × nose), y = nose × x. With +0x259 set the axle
   points lose their 0.19 height.
3. Thrust: `(throttle − brake)·7`, doubled, along the body's nose (local (0, 0, 2f)); boost:
   timer +0x294 counts down, adding BOOST_EXTRA_ACC while timer > 3·BOOST_TIME.
4. Drag: −0.5·v (player) / −0.2·v (others), an acceleration. Player top speed 28 m/s. MAXACC and
   the gear table are not read by this pass.
5. `speed` +0x288 = |v|; curves `(1 − min(1, |v|/15))·2.5 + 1` (pitch) and `·3.0 + 1` (steer);
   player braking or going backward → both 2.0 and the sticks ×min(|v|, 20)·0.05; going
   backward negates the pitch stick and the stored speed.
6. Yaw block: point velocities `u = ω_l × axle + v_l` (front (0, 0.19, 1.25), rear
   (0, 0.19, −1.25), `DAT_00379e50/60`); slip = clamp(steer·curve·(0.125 player / 0.222),
   ±0.222) → +0x28c; the front u rotated about y by −slip; lateral force `−5·u.x` clamped ±12.5
   (front) / ±9.375 (rear, unrotated), world y lane zeroed (unless flag and +0x259); torque
   `offset × F`; an AI below 5 m/s gets a low-speed assist that replaces the forces, and an
   AI's torque z is ×4.
7. Pitch block (when |pitch term| > 0.01 or |steer term| < 0.01): the same with the pitch slip
   about x, the y lane, `−4·u.y` clamped ±12 on both axles.
8. Angular drag `−2·L` (local) plus the roll leveller `−10·m·(up'·r0)` on local z (skipped when
   |nose.y| ≥ 0.7; the +0x44c/+0x44d/+0x44f/+0x2d6 script bytes change the target roll), added
   without the mass factor (`FUN_001f8778`).
9. Bob: pos.y −= old; `(sin p1 + 1 + sin p2)·0.075`, p1 += 0.0022, p2 += 0.0035 (wrapped at 1);
   pos.y += new. p1 starts at a 16-bit random draw / 65536 (`FUN_00187350`:101).
10. The 19 boundary sheets (driving-world-bounds.md §3-§4), every holder applied, each push
    feeding the next sheet's accumulator read, plus the scrape torque.
11. +0x29e (a script flag, `FUN_001931b8`): a local angular impulse (pitch, steer, 0)·throttle·m.

**The integration** is the VU0 microprogram at micro 0xad (`FUN_001f89e0` → `vcallms 0xad`;
the program is the `.vutext` overlays at EE 0x324e70 → VU 0x000 (0x800 bytes) and 0x325678 → VU
0x800 (0x2f8 bytes), disassembled with tools/vu1dis.py) [M]. It replaces the Euler guess of §6
for every body:

```
p   = clamp(p, ±M_LIMIT·m)            0xad/0xb1   (momentum inst+0x40, before the force)
p  += F·dt;  L += τ·dt                0xaf-0xb6
v   = p/m                             0xb9        (Q = 1/vf24.w, the mass lane)
ω   = W·L                             0xba-0xbc   (W = last tick's world inverse inertia)
pos += v·dt                           0xbd/0xd5
ω   = clamp(ω, ±AM_LIMIT)             0xc0/0xc4
L   = R·I·Rᵀ·ω                        0xc8-0xd4   (I = data qword 19 = 1/(inst+0x1c,+0x2c,+0x3c))
q   = norm(q + ½(ω·dt, 0)⊗q)          0xd6-0x10c
R   = rows(q); W = R·diag(inv I)·Rᵀ   0x110-0x14f
```

`FUN_001f8cf0` writes q to inst+0x00, pos/vel/ω/p/L to +0x10..+0x50, R to class+0x00..+0x2f and
W to class+0x40..+0x6f. The inverse inertia comes from `FUN_00210938(mass, half extents)` with
the half extents of the model header box (`FUN_001c9778`). `physics::Car::step` (the land car)
still integrates with its own Euler form; it does not use `Body::integrate`.

## 9. Function table (SUB_PHYSICS layer)

| function | role | port | status |
|---|---|---|---|
| `FUN_00188200` 0x188bf8-0x188c60 | per-vehicle pass choice | `sub_physics::drive_pass`, `Vehicle::player` | ported (the path-driven test is a bool input) |
| `FUN_00190f88` 0x190f88 | the SUB_PHYSICS pass | `SubCraft::controller` | ported; unproven parts below |
| `FUN_00192498` 0x192498 | path-driven pass | `DrivePass::Path` | choice only; returns at once for SUB_PHYSICS [M :86] |
| `FUN_00190510` | IS_SNOWMOBILE pass | `DrivePass::Snowmobile` | not ported |
| `FUN_001f7c60` 0x1f7c60 | reset + gravity | the tail of `Body::integrate` (zero accumulators, no gravity) | ported for SUB_PHYSICS == 1 only |
| `FUN_001f89e0` + VU0 micro 0xad-0x15b | rigid integration | `Body::integrate` | ported op for op |
| `FUN_001f8cf0` | VU0 result flush | the `Body` fields | ported (the hull-point transform is not) |
| `FUN_001f8088` | angular ×0.995 for kind > 3 | none | not applied: vehicles are kind 1-3 |
| `FUN_001fcfc0` | low-motion check → `FUN_001fd170` | none | not ported |
| `FUN_001f87d8` / `FUN_001f8888` / `FUN_001f8778` / `FUN_001f8168` | force and torque API | `Body::add_accel` / `add_torque_mass` / `add_torque` / `add_ang_impulse_local` | ported |
| `FUN_001f82e8` / `FUN_001f8380` / `FUN_001f8250` / `FUN_001f8508` / `FUN_001f84d0` | world↔local | `to_local` / `to_world` | ported (row 3 = (0,0,0,1) per `FUN_00210540`) |
| `FUN_002cbd50` / `FUN_002cbeb0` | sin/cos in turns | `sin_turns` / `cos_turns` | ported with the stored polynomial |
| `FUN_002cc590` / `FUN_002cc510` | y / x rotation matrices | inlined in the two blocks | ported |
| `FUN_002104a8` | input slew | `approach` | ported (the pad layer itself is not) |
| `FUN_00210938` | inverse inertia | `inv_inertia` | ported |
| `FUN_001c9778` | model half extents | none | the disc test uses the decoded LOD-0 box instead [S] |
| `FUN_00190e08` / sheets 0x331a60 | boundary sheets | `bounds::ZoneQuad`, called from the pass | ported (bounds.rs) |
| `FUN_00195a30` | health ratio (> 0 = alive) | `SubCraft::alive` | flag only |
| `FUN_001fc2f0` | car↔world collision | none | not ported for any vehicle; vanquishsub has no NO_WORLD_COLLISIONS line, so the game's sub does collide with the level |

**Unproven or not done.** The model box the game reads for the inertia (the disc test's LOD-0
box stands in). Which option the profile flag is. The start speed (`EStartMission` argument ×
car+0x4c: MIS13A 35, MIS13B 2) is not applied. The world collision for the sub. Which vehicle
the MIS13A/B/C player drives: the view picks `ultralight` for 13A/B and the patrol boat for 13C,
and the player vehicle choice (`DAT_0033196c`, the profile's car name) is not traced. Inside
one port tick the order is controller then integration; the game integrates at the top of the
next frame, which gives the same sequence of values.
