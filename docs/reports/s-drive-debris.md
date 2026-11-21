# s/drive-debris: debris motion, hit marks, bullet impacts, smackable hits

No change to the shared random stream: everything added is a new pure function or a new
consumer (`bullet_impact` takes the caller's draw closure); nothing on foot is touched. No draw
code changed (nf-view untouched). Branched from s/drive-smash-fx (not on master yet).

## What a player would notice

Nothing yet on screen: nothing draws these items. Once the driving renderer consumes them:
glass, wood and rubbish pieces from smashed props fly off, tumble and drop (they fall through
the road and vanish after 70 ticks, as on the PS2), bullet holes and missile marks appear on
walls and the road (with a streak on the ground at the foot of a wall hit low), and bullet hits
play the right spark, puff and ricochet or impact sound per surface.

## What was decoded and ported

- **Debris motion** (`DebrisPiece::pose`, `DebrisPool::poses`, `vu_esin`): the `BondDebris`
  VU1 microprogram (DRIVING.ELF 0x349020, 209 instructions, found through the render list at
  0x348c60 that `FUN_001d9920` draws with) disassembled with tools/vu1dis.py. Per piece: age t =
  now - birth; pos = p0 + v t + (0, -0.002 t^2, 0) (`SimTick.y` = 0x334004, ELF .data, no
  writer); q = (axis ESIN(a), ESIN(a + pi/2)) with a = spin t - 3.95; the standard rotation
  matrix of q places the model. No ground test, bounce or drag exists in the program or in
  `FUN_001d9920`. Type 4 is never faded, expired or drawn.
- **Smackable hits** (`smackable_hit`, `FUN_0019e8c0`): the previous branch's `smackable_hit`
  was `FUN_0019b5a8`, which is `Sentry`'s damage virtual (vtable 0x37c260, typeinfo
  `6Sentry`); it is now `sentry_hit`. The smackable reads damage taken, not health left: event 9
  (damage entries) once 75% is taken, event 10 above 15%, the part anims, the push of small
  kind-2 hits, and a destroyed branch that repeats on every hit at 0.
- **Knocked-lamp glass**: no path starts it. Every caller of the Giotto start wrappers was
  traced (block, world load, `EPlayEffect`, `EPlayActorEffect`, `EPlayGiottoEffect`, `EPowerUp`,
  `ESpawnForceEffect`, explosions, `ECollision`, `EHitWindow`, bullet impacts); a knocked
  smackable sends part anims and event 10 only, the lamp's `Wd_Sprk` / `Wd_Glass` sit outside
  its block, and no script action on the eight tracks names a street light (disc test).
  `FUN_0015b2f8` was mislabelled `ESpawnExplosionStatic`: it is `ESpawnForceEffect` (action
  0x8d), unused on the disc; renamed in code and docs.
- **Destroyed-car explosion**: no code path spawns one at 0 hit points (all callers of
  `FUN_00200088` / `FUN_00159fb8` listed in docs/systems/driving-effects.md section 11); the car
  step's dead branches clear the group slot and credit the kill. A wreck shows its panel damage
  entries; scripted wrecks use `ESpawnExplosionStatic`. Nothing to port for damage.rs's
  destroyed state.
- **Hit marks** (`hit_decal`, `hit_mark_allowed`, `hit_object_anims`, the caller radii): the gate
  `FUN_0019ddb8` and the placement `FUN_001ea8e8` (checked against the disassembly where the
  decompile mangles the stack vectors: the two `FUN_001da6c8` calls at 0x1ead80 / 0x1eb02c).
- **Bullet impacts** (`bullet_impact`, `FUN_0019d5a8`): the Giotto effect by surface and target,
  the spark particles, the ricochet / impact voice.
- docs/systems/driving-effects.md sections 10, 11, 12 updated.

## Tests

- `nf-game` lib `driving::smashfx` (24, 7 new): series sine, debris pose (fall term, phase,
  axes), type 4 skipped, smackable hit, hit marks on a wall and floor (probes, halving, refusals,
  radius cap), bullet impact choices.
- `drive_smashfx_disc` (9, 4 new): the `BondDebris` program words, `SimTick.y`, the debris type
  table, the Sentry / Smackable vtables and the thresholds read from the disc's DRIVING.ELF;
  `Wd_Glass` pieces from the disc falling through their life; no script action names a street
  light (14 `EPlayEffect`, all hydrant entries; 0 `ESpawnForceEffect`); hit marks with the
  disc's decal records 2 and 5.
- `driving_script_disc` (4), `drive_trackfx_disc` (2): pass.
- Mutation checks (each fails a unit test): fall term x 0.5, spin phase dropped, smackable
  reading health left instead of damage taken, hit-mark lift 0.035 -> 0.03, three fit tries
  instead of two, ricochet threshold 0.25 -> 0.5.

## Unproven (stated, not guessed)

- The ESIN series coefficients are PCSX2's interpreter's (the Sony manual's series); the real
  EFU's rounding outside -pi/2..pi/2 is not measured, and the VU's own float rounding of the
  matrix chain is modelled with `ps2float` (toward zero), not measured.
- The debris model meshes (ELF table 0x338e28 `+0x00` / `+0x04` / `+0x0c`) are not exported:
  draw data, for the renderer.
- The writer of a car's `+0x6c` event-0 bits; what the segment query `FUN_00215cc8` reports on
  a miss (the port takes the closure's flags); the identity of `FUN_00199f88` / `FUN_0019d080`
  (two ray-hit paths, named here by address and the radius they pass).
- Live-capture questions: does a lamp knocked over by a car ever show glass; do debris pieces
  visibly drop through the road; is the wall-foot streak visible under low wall hits.
