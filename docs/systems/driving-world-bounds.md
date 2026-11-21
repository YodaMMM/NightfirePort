# The driving boundary sheets (DRIVING.ELF table 0x331a60) — G-056

Invisible sheets that bound the SUB_PHYSICS vehicles. Evidence: [M] DRIVING.ELF code (sha1
`d7b8a57f…`, the decompile under work/decomp/DRIVING.ELF plus the raw instructions), [D] disc
data, [S] suggested. Reviewed function by function in docs/reports/s-pr5-drive-sim.md; that
review corrected the first draft of this page (see §6).

## 1. Who runs the sheets [M]

The vehicle controller `FUN_00188200` picks ONE pass per vehicle per tick
[M 0x188bf8-0x188c60]:

| condition | pass |
|---|---|
| pvehicle `SUB_PHYSICS` (+0xc0) != 0 | `FUN_00190f88` (holds the sheet loop) |
| else vtbl+0x1d4 object's +0xc8 != 0 (`IS_SNOWMOBILE` in the schema, if that object is the attr block [S]) | `FUN_00190510` |
| else +0x2c8 == 0 | `FUN_0018cef8` |
| else | `FUN_0018ecb8` |

(`FUN_00192498` runs instead of all four on a separate branch, 0x188be8.) The .atr files that
set `SUB_PHYSICS=1` [D, decompressed pvehicle files]: alpha_sub, mini_sub, vanquishsub
(MIS11), ultralight (MIS13A/B), ultralightbig (MIS13C), plane and subcopter (MIS4). The
vanquish has no SUB_PHYSICS line, and the registered default reads as 0 [S]. So the land car
never meets these sheets; `physics::Car` does not run them.

## 2. The table [M bytes]

19 records of 0x50 bytes at 0x331a60:

| offset | type | meaning |
|---|---|---|
| +0x00..+0x3f | f32x4 x4 | corners 0-3, `(x, y, z, 1.0)` |
| +0x40 | f32x3 | plane normal |
| +0x4c | f32 | band thickness (the normal's w lane; the dot products use .xyz, dest mask 0xe) |

Records 0-4 are horizontal (normal (0, −1, 0); y = 78.438 / −240.477 / −240.482 / −260.330 /
−95.460; thickness 10/15/15/15/15). Records 5-18 are vertical (normals in xz, stored normal-y
values of order 1e-8; thickness 10/10 then 11). Which mission or map each sheet serves is
unproven. A scan of the code for stores to this address found none [M-negative], so the
values read as constant; a live read would settle it.

## 3. The test [M]

- Plane: `d = (p − corner3)·n` (0x192130-0x19215c); the band is `0 <= d < thickness`
  (0x192168 `c.ole.s 0, d`; 0x19217c `c.olt.s d, t`).
- Quad, `FUN_00190e08`: for each i in 0..3, `e = c[(i+1)&3] − c[i]`; reject when
  `(p − c[i])·(n × e) < 0` (VSUB/OPMULA/OPMSUB order checked in the raw words at
  0x190e38-0x190e50; the reject branch is 0x190e94 with `v0 = 0` in its delay slot).
- `FUN_00190f88` walks all 19 records (s4 = 18 down to −1) and applies EVERY record that
  passes; the hit path ends at the same decrement as a miss (0x1923b4), there is no break.
- `FUN_00190eb8` is the same test returning a bool on the first hit. Its one caller,
  `FUN_00182978` :126-138, tests the point 10 below an actor whose +0x6f4 state is 0x16 and
  sets that actor's +0x70c on a hit. What that actor is, is unproven.

The loop is skipped when `+0x2c8 == 2 && FUN_00195a30() <= 0` (0x1920c4-0x1920e8;
`FUN_00195a30` returns `*(*(vehicle+0x54))` or 1.0 when the pointer is null). Meaning unproven.

## 4. The response [M 0x19219c-0x1923b0]

With `v` = inst+0x20, `f` = inst+0x60 (force accumulator), `m` = inst+0x4c (mass,
drive-physics.md §6), `n` the normal, `t` the thickness, `fps` = the int at 0x339f4c
(sim 0x339190 + 0xdbc, loaded by `lwc1` and converted by `cvt.s.w` at 0x192224 — an integer
frame rate, also used as frames-per-second elsewhere: `fps*300`, `fps<<2`, `x*fps/60.0`):

1. `v_n = −(v·n)`.
2. `s = max((t − d)/t, 0.1)` (0x1921e0-0x1921ec).
3. `v_n < 0 → min(−1, v_n)`; `v_n > 0 → max(1, v_n)`; `0` stays 0 (0x1921f0-0x19221c).
4. `C = n·(fps·v_n·s) + n·(−(f·n)/m · s)` (0x192220-0x19226c).
5. Only when `C·n > 0` (0x192298): `C *= s` (0x1922b8) and `FUN_001f87d8(inst, C)`, which
   adds `C·m` to the accumulator — so C is an acceleration. With `f = 0` and `s = 1` this
   cancels the approach speed in one frame.
6. Then the scrape torque: `r2 = (row 2 of the matrix at *(inst+0x5c))·n`,
   `r0 = (row 0)·n`, `k = max(vehicle+0x288, 10)·|r2|`; local torque
   `(2k|n.y|, 1.5k·sqrt(n.x² + n.z²), 0)`, x negated when `r2 > 0`, y negated when `r0 < 0`;
   `FUN_001f84d0` transforms it by that matrix (whether the translation row joins depends on
   a broadcast field the decompile hides, unproven) and `FUN_001f8888` adds it times mass
   to the torque accumulator (+0x70).

## 5. The port

`crates/nf-game/src/driving/bounds.rs`: `ZONES` (the table bytes), `signed_dist`,
`inside_quad`, `hit`, `first_hit` (0x190eb8), `hits` (every holder), `penetration_scale`,
`clamped_approach_speed`, `boundary_force` (steps 4-5, returns None when the gate fails),
`scrape_torque_local` (step 6 before the transform) and `sheet_pass` (the whole loop for one
body). No vehicle in the port runs SUB_PHYSICS yet, so nothing calls `sheet_pass`.

## 6. Corrections to the first draft

- It called the response "dt-scaled" with `0x339f4c` a time step; it is the integer fps.
- It said the first hit wins; every hit applies.
- It applied the scale once and had no gate on the returned vector.
- It read the scrape terms as `|v·n|`; they use matrix row 2 (and row 0 for a sign), and
  `*(inst+0x5c)` is the orientation matrix (`FUN_001f84d0` multiplies by its four rows).
- It called the sheets the drivable ring of the car maps and wired them into the land car's
  step; the dispatch in §1 shows they belong to the SUB_PHYSICS pass.
- The "+0x5C paired body", "twin", "camera / mission consumer" readings were guesses and are
  dropped.
