# turret-round-spread — report (2026-11-04)

Branch `s/turret-round-spread` cut from origin/master (8deea246; the sandbox reset between
tasks — fresh clone, work-slim + the ISO re-downloaded, rustup + the user ALSA prefix
rebuilt). Window ~110 minutes.

## The finding: the premise is already fixed on master — this round proves it and pins it

**The decompile law [M].** `Bullet_init` 0x12c5a0's spread block at LAB_0012c7f4 is
UNCONDITIONAL: every round draws three shared-stream numbers —
`Rand_FRand_MVar2(2·((weapon+0x24) + bloom) · scale)·0.0014`, then `Rand_FRand(2π)`, then
`Rand_FRand(π)` — and `Vec_Spherical_2_Cartesian` turns them into the direction offset
(normalise the aim, add the offset, normalise again). The owner arms only scale the spread:
no owner or a non-player, non-drone owner keeps scale 1.0; the player-class owner reads
BLData+0xc0 (the bloom arm); a class-3 target computes the aim-error bloom (`+0x92/+0x98`);
the weapon+0x68 bit 0x400000 (with the aim bit) zeroes the scale — dead-on.

**The turret rounds take the full cone.** `GT_Update` 0x180498 case 4:347-348 calls
`Bullet_init(-1, owner, target, 0x2c55d0, muzzle, dir, -1)` — the owner arg is the turret's
+0xb8 gunner or the turret obj itself, whose class byte is 0x36 [M 0017ffa0:77 — not the
player's 2, not a drone's 3], so the scale arm stays 1.0 and the bloom never computes. The
turret weapon record at the fixed 0x2c55d0 reads **+0x24 = 4.0** (the spread base), +0x20 =
15.4833 (the speed), +0x6c bit 0x1000 (the 300.0 range arm), +0x68 = 0 (no 0x400000) [L
ee_snapshot.bin @0x2c55d0, file offset = the EE address]. So every turret round leaves
through the full 4.0 cone — the "owner -1" in the port was `Bullet_init`'s own first short
arg (a sound/type id), never the spread owner. `GT_Update` then draws one more
`Rand_FRand(2π)` into the direction vector's w pad [M 0x180498:349; the second fire site's
pad at :522] — **4 shared-stream draws per turret shot, in the game's order: the cone 3,
then the pad**.

**The port already carries this** — commit 01cac91a (2026-11-01, level-0a) landed the cone +
the pad draw in both turret fire arms (drone.rs:17373/17408, the player-mounted arm 17373),
with `bullet::spread`/`cone`/`player_spread` modelling the 0x12c5a0 law including the G-046
spherical form. This round verified that law against the decompile (the branch changes NO
`crates/*/src/` behaviour) and landed the missing pins.

## The pins

1. **The unit law** (`bullet::tests::cone_draws_three_even_when_dead_on`, new): the cone
   consumes EXACTLY three draws [M 0x12c5a0:171-180]; spread 4.0 moves the direction off the
   aim; the 0x400000 dead-on arm leaves the direction EXACTLY the aim vector while still
   drawing the three (the unconditional block); the same seed reproduces the same direction.
   **The dead-on sister lives here** (the 0x4080d0 sniper assert also sits in the existing
   `player_spread_blooms_and_the_aim_gate`).
2. **The burst integration pin** (`gun_turrets_live::
   the_wall_gun_burst_scatters_and_draws_in_the_game_order`, new): the 0700000a wall gun #636
   (ch 18, facing 1) fires a 12-shot burst at the tester standing 40 u in its front cone; the
   pin is the measured damage/draw-count golden sequence —
   `[(2.4, 11673), (2.4, 12127), (2.4, 12581), (2.4, 13035), (2.4, 13489), (2.4, 13943),
   (2.4, 14397), (2.4, 14851), (2.4, 15305), (2.4, 15759), (2.4, 16213), (2.4, 16667)]`
   — bit-exact (the fixed-60 Hz deterministic sim). The damage is the record's flat player
   damage (the cylinder path has no part boxes, so the ±0.0112 rad cone keeps every round on
   one number — the scatter is geometrically invisible on a standing player at this cone); the
   DRAW COUNTS are the pin's teeth: the pre-fix port (dead-on, no cone draws) sat 4 draws/shot
   lower and every later count desynced. The +454 draws between consecutive shots = the game's
   4 (the cone 3 + the GT pad) + the level's per-tick consumers.

## How it was verified (the totals)

Release, `--test-threads=1`, the real ISO:

- `gun_turrets_live`: **3 passed / 0 failed** (the two existing + the new pin).
- `ronin_live`: **9 passed / 0 failed**.
- `interactables_0700000a_live`: **13 passed / 0 failed**.
- `keyboard_0700000a_live`: **1 passed / 0 failed**.
- `walk_0700000a_live`: **3 passed / 0 failed**.
- `walkreal_0700000a_live`: **12 passed / 0 failed / 3 ignored**.
- `cross_process_determinism`: **3 passed / 0 failed**.
- `nf-game --lib`: **658 passed / 0 failed / 5 ignored** (the new cone test included).
- The full four-crate suite (tools/test/suite.py): restarted on this final tree in the
  background; this sandbox's first attempt died silently at 0 bytes (the fresh-box build), the
  restart's tail line is quoted in the WORKLOG if it finished a chunk by the window's end. The
  per-binary totals above are this tree's own runs.

## What's left

- The suite's total line on the final tree (the runner was re-launched; the window closed
  during its run phase — the per-binary totals above carry this tree's coverage).
- [S] unverified, for a live check: whether the game's wall-gun damage to the player is flat
  2.4 per hit as the port's cylinder path reads it (the port models the player hit as the
  cylinder + the record damage; a PCSX2 trace of the wall gun's burst on a standing Bond would
  pin the per-hit number and any part structure the port's player path lacks).

## Live checks for the main agent

1. **0700000a, the wall-gun burst**: stand Bond 40 u in front of gun #636, raise ch 18, let it
   fire a 10-shot burst — expected: every hit costs the same flat number and the burst never
   misses a standing Bond (the cone ±0.0112 rad vs the cylinder). A gdbwatch on
   `Player_HandlePain` 0x1902c8 during the burst pins the per-hit damage and the hit tick
   pattern against the port's golden sequence above.
2. **The draw stream**: a gdbwatch on `Rand_FRand` 0x1e2d28's callers during one wall-gun shot
   — expected: FRand_MVar2, FRand(2π), FRand(π) inside Bullet_init 0x12c5a0, then one
   FRand(2π) in GT_Update 0x180498 — the 4-draw order the port pins.

## Usage

not visible to me — no token, credit, cost or context-window figures are exposed anywhere in
this environment (no status bar, no session summary, no API dashboard access).
