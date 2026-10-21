# The drawn gate (`obj+0xFC`) — the sim-side frustum

*Task ai-sight-drawn-gate (2026-10-20). The writer the sim replaces, the game's test, and every
reader the flag feeds.*

## What the game writes

`View_AddObjects` (0x1e62d8 [M]) walks each visited cel's object list and, when the object's world
sphere passes `Vision_InView` (0x1e89e0 [M]) — plus the display-flag gates (+0xf8 display mask,
+0xf0 & 0x8010 hidden, +0xfa/+0xd8 glist checks) — writes `obj+0xFC = 2` (0x1e62d8:71). The cels
come from the portal walk (`View_CaptureScene` 0x1e6df0 → `Vision_Init_Portal_Recurse` 0x1e89a8 →
`Vision_Portal_Recurse` 0x1e86e8), so an object is only tested when its owner cel is visible; the
clearing/rewriting happens every rendered frame.

## The frustum test

- `Vision_Calculate_View_Fustrum` (0x1e7df8 [M]) builds the far-quad corners in camera space at
  depth 200 — x = ±tan(view/2)·aspect·200, y = ±tan(view/2)·200 (`fVar4 = tanf(*(viewer+0x118) *
  0.5)`, `fStack_90 = fVar4 * *(viewer+0x114) * 200.0`) — transforms them by the viewer's
  camera-to-world matrix rows (+0x120/+0x130/+0x140, translation +0x150), and crosses them into
  five planes (`Plane_PlaneEq` 0x1df8d0 [M]: normal = normalize(cross(p1−p0, p2−p1)), d = −n·p0).
  Four side planes are copied into viewer+0x70..0xB0 (`amemcpy … 0x40`); the far plane is not.
- `Vision_InView(point, slack, viewer)` (0x1e89e0 [M]) accepts when every one of the four planes
  meets the point. The port ships the signed-distance reading `n·(P − eye) >= −slack` — the same
  form nf-view's cel cull already uses (`vision_in_view` in scene.rs, fix-portal6: the raw
  4-lane reading `n·c + d·r >= r` is untestable-absurd with the game's own stored planes
  [L ee_start_05.bin: |n| = 1, d = −n·eye, |d| 51..117]).
- The view angle is the on-foot default 0x3F860A92 (1.0471976 = 60°) [M Camera_CreateCameras
  0x1bb2b8:63-121, Camera_CalcViewAngles 0x1baf70:32]; the GunImp scope's 30° arm (0x132f00) is
  out of scope (the sim has no scope state). The aspect is the DAT_002a37c4 == 0 branch 0x3FAAAAAB
  (4/3) [M 0x1baf70:29-33; L ee_snapshot.bin @ 0x2a37c4 = 0, ee_start_07000001.bin @ 0x2a37c4 = 0].
- The viewer's matrix is the player camera's: `Camera_SetToPlayer` (0x1bc048 [M]) composes the
  yaw matrix (obj+0x90) with `RotMatrixX(−pitch·π/2)` (the pitch at BL+0x8a8, in π/2 units) and
  the translation row is `Player_GetHeadPos` — so right = (cos yaw, 0, −sin yaw), up =
  cross(forward, right), and the frustum tilts with the look pitch. The port's equivalent inputs
  are `player.eye` (the established head/LOS origin) and `player.aim` (= `Bond::look()`, the
  game's Mat_GetDir).
- The slack is the object's world-sphere radius obj+0x8C [M 0x1e62d8:27-29 +
  `Control_BuildWorldSph` 0x133070: glist+0x1C · abs(+0xE8)]. The port's drones carry no glist —
  `DRONE_DRAWN_SLACK = 1.0` [S]: [L ee_snapshot.bin] every guard glist+0x1C read 1.1589-1.1602
  and the built obj+0x8C radii 1.019-1.128, so 1.0 sits just under the live range (the port's
  gate is marginally tighter at the plane edge, ≤ 0.13 units).

The port's functions: `nf_game::drone::frustum_sides(forward)` (the four inward normals),
`nf_game::drone::in_view(eye, pos, slack, sides)`, consumed by `visible_to` (the drawn gate)
and `drone_position_visible` (the spawner's gate, game slack 2.0 [M Drone_PositionVisible
0x13a7a0]).

## The stand-in it replaces

`visible_to` was a 60°/2-unit cone around Bond's HORIZONTAL aim (`dot > 0.5` on (aim.x, aim.z),
or within 2 units) plus five bullet rays from the eye. The cone was pitch-blind and wider than
the game's frustum (120° horizontal fan vs the frustum's 60° tall × 2·atan(tan30°·4/3) ≈ 75°
wide): looking up left horizon guards "drawn" the game puts below the tilted bottom plane; a
guard 45° off the aim was "drawn" the game leaves outside the 37.57° half-angle. The bullet-ray
walk STAYS — it stands in for the portal/cel room gate the sim has no graph for (guards-0c: the
BULLET segment, pass-flagged occluders drop [M Collide_Intersect 0x1ebff0-0x1ec010], so glass and
foliage don't hide a guard — the 0c museum playtests).

## Readers the flag feeds (all [M])

- `Check_AutoAim` 0x1a73f8: skips drawn==0 — Bond's crosshair only snaps to on-screen enemies.
- `NDrone2_DoAnimation` 0x149f88: an undrawn guard slides at the danim's speed with no gait (the
  guards-0c slide fix's precondition).
- The spawner kill-cleanup's draw-counter gate [M `lbu $v0, 0xfc($s0)` 0x13805c] and the spawner
  spawn-when-unseen path via `Drone_PositionVisible` 0x13a7a0.
- `DSTATE_Dead` 0x163fe8: corpses despawn once undrawn.
- `Bullet_heatseekable` 0x12f248: thrown-hat seek targets must be drawn.

## Tests

- `crates/nf-game/tests/frustum_sides.rs` — the plane directions at the half angles (60° view,
  4/3 aspect), the acceptance band (35° right in, 45° out, 26.6° up in, 42° up out, behind out),
  and the pitch tilt (the horizon falls out while looking up 45°). ISO-free.
- `crates/nf-view/tests/sight_frustum_live.rs` — the real path (`nf_view::load` +
  `Sim::step_bond` inputs, no state pokes) on 07000001: 120 idle ticks fix the start census
  (placements 158/161/261 bullet-clear and drawn); 40 look-up ticks (input.look = 0.01/tick —
  the on-foot pitch path) put Bond at pitch ≥ 0.39 with placements 158 and 261 still bullet-clear
  and still inside the old cone, and the port marks them UNDRAWN (fails on the stand-in); the
  sister pins the start-view drawn set (261 passes only through the world-sphere slack — dropping
  the slack flips it).
