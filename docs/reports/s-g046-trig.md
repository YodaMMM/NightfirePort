# s/g046-trig: exact maths at more of the game's own sites (G-046)

RNG: no change to the number or order of random draws. Bullet directions now differ in their last
bits, so a shot that grazed an edge can land differently, and that can move later draws on long
runs. None of the walkreal runs below moved.

## Tests

- nf-game lib: both new tests pass and fail on the old code. The trig tests pass.
- nf-view, all green:
  - 0700000d_gunimp_aim_live: 3 passed.
  - 0700000d_gunimp_beam_axis_live: 1 passed.
  - 0700000d_gunimp_door_live: 5 passed.
  - 0700000d_gunimp_rate_live: 1 passed.
  - gunimp_mounted_live: 2 passed.
  - walkreal_0700000d_live: 3 passed.
  - walkreal_07000001_live: 5 passed.
  - walkreal_07000005_live: 16 passed.
- Not run: the other levels' walkreal binaries (every level fires bullets through `cone`).

## Swapped

1. **Every bullet's direction** (`bullet::cone`, used by Bond's guns, the guards, the turret, MP).
   `Bullet_init` 0x12c5a0:180-184 normalises twice with `Vec_Normalise` 0x1e3000. The port used an
   IEEE `sqrt` and three divides. The new `trig::vec_normalise` follows the disassembly
   (0x1e3008-0x1e3058): VU0 `vmul` squares (chopped), `add.s` x+y then +z, `sqrt.s`, a zero length
   giving (0, 0, 0), `div.s 1/len`, then `vmulx` lanes (chopped). The three offset adds are `add.s`.
   Test: `bullet::tests::the_bullet_direction_is_the_games_vec_normalise`. On the old code it fails
   on its first aim, (3.7, -0.42, -11.3): x and z are each 1 ulp off.
2. **The 0700000d mounted turret** (`gunimp.rs`, `GunImp_Update` 0x1320d8:162-211).
   - The RotMatrixY/RotMatrixX lanes are `PS2Sinf2(a + 1.5707964, a)` [M 0x1e4718, 0x1e46a0].
   - `Concat`'s VU0 products are chopped [M 0x1e4a90]. With the identity base, each entry is one
     product.
   - The muzzle goes through `vmula/vmadda/vmadd + vadd` [M :205-211].
   - Bond's seat goes through RotY + `vadd` [M :176-189].
   - The fire arm now passes the raw `Mat_GetDir` row (`barrel_row`) into `Bullet_init`'s
     normalise [M :325-331].
   - The seat's `glist+0x28 * 0.75` is a `mul.s` [M 0x131f20:81].

   Test: `gunimp::tests::the_turret_aim_is_the_games_rotmatrix_lanes`. On the old code it fails at
   yaw -2.9, pitch -1.5: the barrel row was 8, 1 and 6 ulp off.

What a player would notice: nothing visible. Shots and the turret laser follow the game's exact
line down to the last bit.

## Not swapped (unproven or structural)

- **Bond's auto-aim** (`Check_AutoAim` 0x1a73f8). It uses `atan2f` and `asinf` 0x232d48, and the
  port has no `asinf`. It also locks through `View_3DPoint2Screen` into
  `Player_GetAimingPoint` 0x1a4a98's screen-space offset, while the port works in angles. This
  needs a restructure, not a trig swap.
- **Riding a moving platform** (`Control_InheritVelocity` 0x133e88). The game takes `atan2f`/`SQRT`
  angle deltas and builds a `RotMatrix`. The port rotates about a pivot with host `sin_cos`. This
  is also structural.
- **Distance `sqrt` sites** in drone.rs, grenade.rs, explode.rs and gadget.rs. They differ by at
  most 1 ulp, and each needs its own op-order trace.
- **The turret's 0.15 ease.** It runs on EE ops, but it is interpolation, so it stays as is.
- **A mount base that is not the identity** (the MP turrets) keeps the summed-yaw form [S].
- **The EE `sqrt.s`/`div.s` models** (`ps2float::sqrt` chop, `div` nearest) are the port's shared
  models. This branch did not check them live.

## Draw note

`base_matrix`/`barrel_matrix` also pose the drawn turret. The change is at the ulp level, with no
visible difference, and no draw code changed.
