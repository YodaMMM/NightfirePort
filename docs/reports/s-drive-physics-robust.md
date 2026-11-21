# s/drive-physics-robust: the AI cars' land pass, the hull response and the 2-D walls

No change to the shared random stream: nothing here draws from any generator. Nothing on foot
is touched. `crates/nf-view/src` is untouched; the view keeps calling `Car::step` and
`CollisionGround::new` exactly as before (player kind, no hull list, no wall list), so the
driving view behaves as before until it opts in (see the last section).

## What a player would notice

Once the view passes the walls, the hull and the driver kind: enemy cars, traffic and the race
rival stay on their wheels and on the roads. Before, a pursuer that clipped a kerb rolled and
sank through the road, cars drove into and under raised blocks, and fast cars ran off the
mesh. The player's own car also gets the game's landing lift and its wheel-drive rule.

## The causes, with evidence

Reproduced with the traffic brain driving `Car::step` on each track's collision (81 spawns
across MIS01, MIS4 and RACE, 3600 ticks each). In the pre-branch configuration 26 runs fail
(rolled, below the ground, off the mesh, or off the road); the sister test
`the_pre_branch_configuration_fails_the_same_runs` keeps that configuration.

1. **AI cars ran the player's pass.** `FUN_00188200` sends every car with actor+0x2c8 != 0 to
   `FUN_0018ecb8` / `FUN_0018e548` [M 0x188bf8-0x188c60]: 5× (7.5× for hostiles) grip shares, a
   lateral limit of 20 on every wheel, drive `max(MAXACC, 12.5)` on both rear wheels, steer
   `max(MAXSTEERING, 0.2)` with no speed falloff, no surface table. This is the "missing tyre
   side grip" the traffic AI patched over. Ported as `Car::controller_ai`; `DriverKind` carries
   +0x2c8 (0 player, 2 the AIEl cars [M 0x12cb08:201-205, 0x1862c0:175-179], 3 traffic
   [M 0x13fae0:162]).
2. **Pass C used stand-in points and response.** The game tests the car model's hull list
   (`Base` +0x60, count +0x3c, y lowered by `FUN_001d38a8`) [M 0x1f72d8, 0x195ae0, 0x1c98f8],
   skips points above the centre while the centre is above ground, and responds through the VU0
   program at micro 0x1a0 over every contact (lifted: restitution 0.05·min(|vn|, 10), a
   tangential push, applied only when the impulse's y ≥ 0). The old upright stand-in points sat
   near the floor and a single-corner inelastic impulse spun the body: the kerb rolls.
3. **Pass B for land cars reads 2-D walls the port never loaded.** The land gather flags query
   only each article's wall list (`ca` +0x02 / +0x04, 32-byte records) [M FUN_00219a50,
   FUN_00219328, FUN_0021a248, FUN_00231548]; MIS01 has 24935, MIS4 and RACE 6275 [D]. Raised
   blocks such as MIS01's 5 m planters have no steep triangles, so the triangle stand-in let cars
   in and under them. Ported: `Crp::extract_collision_walls`, `CollisionGround::with_walls`, and
   `FUN_001fb318`'s land branch (body y and z torque ×0.2).
4. **The vtable slot the wheel passes call** is `FUN_001931f8` = `prev_compression != 0`
   (vtable 0x379ef0 +0x214) [M], not "true": a landing wheel lifts the body by its unclamped
   `w + REST` (scratch +0x324) in both passes, and the player's drive needs the wheel or its axle
   partner down last tick [M 0x18c1b0:160-166, :396-409, :657].
5. **The integrator** (micro 0x568) is lifted: per-component M_LIMIT clamp before the force,
   L as the state with a per-component AM_LIMIT clamp on ω, the quaternion step. It agrees with
   the independent lift in `sub_physics::Body::integrate`. This was not a failure cause by itself
   (the semi-implicit Euler was close); the gyroscopic part now follows the game.
6. **In-air levelling** `FUN_0018bb90` for kinds 1 and 2 [M]: ported (`Car::air_level`).

## Tests

- `nf-game --test drive_ai_land_disc` (new, NF_ISO): 81 runs × 3600 ticks of the traffic
  hatchback (kind 3, 7.5 m/s) and the hostile sedan (kind 2, segment limits) with the tracks' own
  .atr and model hulls: none below the ground, none rolled, none more than a lane width past the
  road edge (worst 3.9). A run is stopped, not failed, when the brain crawls at full lock round a
  carrot inside its turning circle for 4 s (5 runs; FUN_00131770's stuck re-join is not
  ported). Also pins the wall counts and the vanquish hull [D]. Sister: the pre-branch
  configuration fails 26 runs. Pass.
- `nf-game --test drive_ai_land_physics` (new, 5 unit tests, each with an old-path sister): the
  25 m/s² AI drive, the 0.2 steer floor, the levelling (hostile levels, traffic does not), a car
  stopped by a 2-D wall (and passing through without the list), the quaternion round trip. Pass.
- Regressions, all pass: `drive_land_disc` (the player's MIS01 run), `drive_reset_disc`,
  `drive_land_physics`, `drive_physics`, `driving_ai_physics`, `driving_ai_traffic`,
  `driving_ai_follow`, `driving_ai_hostiles`, `driving_ai_patterns`, `driving_ai_heli`,
  `driving_roads`, `drive_sub_physics`, `drive_sub_collide`, `drive_sub_physics_disc`,
  `drive_damage_disc`, `driving_controls_disc`, `drive_hud`, the nf-game lib tests (1010),
  `nf-formats` `carmesh_disc` and `crp_mesh_disc`. `nf-view` builds (cargo check).
- A stress run in development at 2.5× and 4× the segment limits (up to 44 m/s) had no roll or
  below-ground case; the brain then runs wide on bends (up to 36 off the centre line), which is
  the carrot, not the body.

## For the view (not done here)

Per land track: `CollisionGround::with_walls(tris, crp.extract_collision_walls()?)`; for each
car `params.half_extents = carmesh::header_half_extents(model)`, `params.hull =
carmesh::hull_points(model)`, and `car.kind = DriverKind::Hostile` / `Traffic` for AI cars;
AI cars step through `AiDriver::step_land`. The hostile runtime (s/drive-hostile-runtime) and
the race rival (s/drive-race) should pass the kind and hull when they build their `Car`s.

## Unproven / not done

- The player's car without a hull list keeps the box stand-in points and the old pass C
  impulse; with the vanquish hull the player would get the game's response, not checked
  against the player tests here.
- Walls of rotated `ci` items are tested in world space, the game tests them in the item's frame.
- Not ported: the hit stun +0x29d, flat tyres, the kind-3 two-query wheel shortcut, the kind-2/3
  tipped damage (100 hit points per tick), kind-1 righting, the wall query's object lists, the
  surface 0x0e event, the snowmobile restitution, the matrix re-orthonormalisation details of
  FUN_002cc690.
- The race rival's own profile and the hostile chase brain were not run on this physics; the
  disc test uses the traffic brain with the road-limit profile as the hostile stand-in.
- Live questions: does a traffic car in the game crawl round an unreachable carrot until the
  stuck re-join fires, and after how long?
