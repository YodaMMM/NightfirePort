# Drive suspension — the per-wheel spring-damper (DRIVING.ELF `FUN_0018fc00`)

Evidence: [M] DRIVING.ELF decompile + raw instructions (work/decomp/DRIVING.ELF), [D] disc data,
[S] suggested. Reviewed in docs/reports/s-pr5-drive-sim.md, which corrected the first draft
(§5). Complements drive-physics.md and driving-world-bounds.md.

## 1. Where it runs [M]

`FUN_0018fc00`'s only caller is `FUN_00190510` (:258). The controller `FUN_00188200` runs
`FUN_00190510` when `SUB_PHYSICS` (+0xc0) is 0 and the object returned by vtbl+0x1d4 has a
non-zero +0xc8 [M 0x188bf8-0x188c30]. In the pvehicle schema +0xc8 is `IS_SNOWMOBILE`
(`FUN_001968f8` :265); supersnow and small_snowmobile set it [D], and `FUN_0018fc00` compares
the vehicle name with "small_snowmobile" (:366). The vtbl+0x1d4 object is the attr block: the car
class's slot (vtable 0x379ef0 + 0x1d4 holds 0x193170) is `FUN_00193170`, `return car+0x23c` [M]. `FUN_0018e548` has the same per-wheel shape (rest + query + clamp + `+0x324` max).

The attrs come through `*(vehicle+0x23c)`; `FUN_00188200` writes that slot back after each
pass (`sw s0, 0x23c(s2)` in the delay slots at 0x188c10/0x188c3c/0x188c54/0x188c60) [M]. What
it holds during the pass (the first draft's "swap slot to dword_3D0D10") is not re-checked:
`FUN_0018fc00` itself uses `DAT_003d0d10` + 0x250..0x330 as scratch.

Attr offsets [M `FUN_001968f8` :149-172]: `SPRING_STIFFNESS_FRONT/REAR` +0x50/+0x54 and
`SPRING_DAMPING_FRONT/REAR` +0x58/+0x5c (default 1000 each), `SPRING_REST_LENGTH` +0x60
(default 2.0), `SPRING_COMPRESSION_LIMIT` +0x64 (default 2.0), `_DRAW` +0x68 (2.0).
vanquish.atr [D]: 80 / 98 / 400 / 350, rest 0.24, limit 0.75, limit_draw 0.125.

## 2. Per wheel, i = 0..3 [M `FUN_0018fc00` :49-354]

1. `x = wheel[i].w + REST` (param_4 is the per-wheel vector array; :51-52).
2. The ground query vtbl+0x214 with the wheel index (:53). When it returns 0, the scratch
   `+0x324 = max(+0x324, x)` (:55-58); after the loop the body's y (+0x14) gets `+0x324`
   added (:354). What the query writes into `x` is unproven; the port's `rest − distance`
   form is [S].
3. `x = max(min(x, LIMIT), 0)` (:64-70).
4. Contact when `x > 0 || prev[i] > 0` (:73; prev = vehicle+0x210 array), and
   `0.2 < up_y · wheel[i].y` (:74; `DAT_00331970` = 0x3e4ccccd; up_y = row 1 y of the body
   matrix at +0x5c). Failing either skips the wheel's forces.
5. `x > 0`: count++ (+0x330), `F = x·STIFF + (x − prev)·DAMP`, front (+0x50/+0x58) for i < 2,
   rear (+0x54/+0x5c) otherwise (:75-87). A second query result of 0 raises the vehicle's
   +0x2ec to at least `x · 5.0` (`DAT_00396330`, :88-94). `x == 0`: F = 0 (:99) and the
   tyre-force block below still runs.
6. The tyre-force block (:101-265) builds wheel-frame vectors with VU0 cross products and
   applies them through `FUN_001f8830` / `FUN_001f88c8`; not ported, not reviewed in detail.
7. `prev[i] = x` for every wheel (:345). The function returns the contact count.

After the loop (:354-401): with `r = vehicle+0x240`, a scratch vector
`(r·0.45·{1.5 small_snowmobile | 2.0 otherwise} or 0, max(DAMP_FRONT, DAMP_REAR)·−150, …)`
is crossed and applied through `FUN_001f84d0`/`FUN_001f87a8`. Roles of these terms are
unproven.

## 3. Neighbours in the same cluster (spot-checked only)

- `FUN_0018ecb8` and `FUN_0018cef8` (:389) and `FUN_00192498` (:128) share the wheel-roll
  step `speed / (WHEEL_RADIUS · 6.32 · fps)` with `fps` the integer at 0x339f4c, zeroed below
  0.01 and capped at 1.0 in `FUN_00192498` (:134-138).
- `FUN_0018ecb8` :240 and `FUN_00190510` :168-179 share the handbrake term
  `HANDBRAKE_FORCE · (speed·0.075 + 1.75)` (speed >= 1), `2·speed + 1` below.
- `FUN_0018cef8` :701: `(v − 0.3) · 1.4285715`.
- `FUN_0018bb90` :97-103: running average with the int attr at +0xb8.
Roles of the other cluster functions (`FUN_0018c1b0`, `FUN_0018e548`) are unproven.

## 4. The port

`crates/nf-game/src/driving/suspension.rs`: `SusParams` (`VANQUISH`, `GAME_DEFAULTS`),
`SusWheel::step` (steps 1, 3, 5 with the [S] distance form; returns `Some(0.0)` for a wheel
that only last tick was compressed), `end_tick` (step 7), `upright` (the step 4 gate, for the
caller), `axle_of`, plus the shared terms `watercraft_handbrake_drag`,
`watercraft_longitudinal_thrust`, `rotor_wheelspin` (names kept for callers; §3 says what they
are). No port vehicle runs this pass yet.

## 5. Corrections to the first draft

- `SPRING_COMPRESSION_LIMIT` for the vanquish is 0.75 in the .atr, not the 2.0 default.
- `FUN_00190510` is the IS_SNOWMOBILE branch, not a watercraft / submersible pass; subs run
  `FUN_00190f88` (SUB_PHYSICS).
- The wheel-roll term divides by `WHEEL_RADIUS · 6.32 · fps`, not `· dt`, and caps at 1.0.
- The rest term is `wheel.w + REST`; the "hit+0x0C" reading of `FUN_0018e548` was the
  same `*param_4 + 0xc` (the wheel vector's w).
- A wheel with `x = 0` but `prev > 0` stays in contact with F = 0; the draft dropped it.
- The "extra rear damping `max(DAMP_REAR, +576·0.45)·−150`" line mixed two terms: it is
  `max(DAMP_FRONT, DAMP_REAR)·−150` and, separately, `+0x240·0.45·{1.5|2.0}`.
- `0x331970` is the 0.2 attitude gate; the draft's PAD_DEAD_ZONE name for it is unproven.
