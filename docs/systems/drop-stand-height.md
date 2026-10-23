# drop-stand-height — Bond never stands on a drop or a corpse; the take ray's end band

The batch-1022l finding: on 07000001 a dead guard's dropped gun rests with its centre ON the floor,
and `Bond::stand_at` exactly over it reportedly left Bond's root 0.139 higher than floor + 1.1,
with the take's LOS ray then refusing (`pickup.rs touches`). The game's rules below were settled and the port
re-verified the port against them (2026-10-22).

## The game's filter chain (what each query skips)

`Collide_Pick` (0x1eb320) walks the cel's STATIC geometry AND the cel-linked objects; the object
half runs through `Collide_PickObj` (0x1eae80), whose skip mask is built from the query flags
[HITTEST+0x98] [M 0x1eae80:56-64]:

| query flags | skip mask over `obj+0xF0` |
|---|---|
| (every query) | bit **0x20** — the corpse bit (`NDrone2_SetAsDead` 0x14a2f0 sets it; bullets additionally skip `& 0x60`, the corpse|pickup pair, at 0x12ce80:124) |
| bit 0x20 set | `+ bit 0x1000` — the missile cage |
| bit 0x8 set (bullets) | `+ bit 0x40` — **pickups** (`Pickup_Create` 0x18dda0:137 gives the pickup obj `glist+0x44 & ~0x200 \| 0x840`) |
| bit 0x4 set | `+ bit 0x10` — disabled |

Plus the class gates (0x1eae80:64-74: flags 0x400 → skip obj class 3/0x12; 0x100 → 2/0x11; 0x200 → 5)
and the shape gate: an object without a collision record (`obj+0xD8 → +8` empty) is skipped [M].

**The ground/stand query** — `build_PointOnFloor` 0x1c95a8:44-52 casts the down-ray with flags
**0x70c**: bit 0x8 set → pickups are skipped; bit 0x20 is in the base mask → corpses are skipped.
**Bond never stands on a dropped gun or a body in the game; floor + 1.1 is the standing height
everywhere a drop rests.**

**The take query** — `Pickup_Update` 0x18e148:210-214: the distance gate
`Vec_SqDist3D(pickup+0x80, player+0x30) < (radius + 1.1)²` (the player's POSITION, not his sphere),
then `Collide_LineOfSight(player+0x80, pickup+0x80, player cel, **the pickup's own obj**, 0, 0x721)`.
`Collide_LineOfSight` 0x1edf50 is exactly `Collide_RayIntersect(...) == 0` [M, the one-call
wrapper]. The pickup's own obj sits in RayIntersect's exclude pair (HITTEST+4/+8, skipped first in
`Collide_PickObj` 0x1eae80:15-19) — **the ray's endpoint, which is inside the pickup's own world
sphere (dist 0 < r), never blocks its own take**. Flags 0x721: the cage skipped (0x1000), corpses
skipped, and the surface classes 0xd/0xe dropped (`Intersect_PointGeom` 0x1e92d8's filter
`(surface & 0x3f) − 0xd < 2` under query bit 0x1 [M-dis 0x1e9890-0x1e9894]).

## The endpoint contact and the port's end band

A resting drop's sphere centre sits |centre.y| above the floor hit [M the fall rest at 0x18e148].
The Raptor's model centre y ≈ 5e-7 [L the drop-stand-height probe: bounds centre (0.0002, −0.0000,
0.0000), the floor ray at the drop xz 5.3452096 vs the centre 5.3452] — the take ray's endpoint
effectively touches the floor plane. The game takes these drops anyway [L 01-raptor.trace 6.0 s], so the game's triangle test absorbs the contact: its VU0 fixed-point path quantizes
to 2^-14 (the 6.103888e-05 constant in `Intersect_PointGeom`'s decompile) — a sub-quantum endpoint
contact lands ON the plane and does not produce a blocking hit short of the end.

The port's float rest carries no quantization: the resting centre can sit a float-noise sliver
INSIDE the plane, and the vertical take ray then crosses the plane a hair BEFORE its end — a real
`t < 1` hit. The old end band (1e-4, marked [S]) refused one such rest (batch-1022l: the take's
"LOS ray ends on the floor short of the 1e-4 tolerance"); this tree's sweep of ~50 stands around
two guard drops measured no refusal at all (every ray returned no hit — the endpoint sits on or a
hair below the plane, t ≥ 1 or None). The band is now **1e-3** — the port's float-noise slack for
the rest composition (a millimetre: 10× the batch's ≥1e-4 case, ~16× the game's fixed-point
quantum, far below any real cover's thickness), replacing the [S] guess with the settled chain
above. `pickup.rs touches` carries the full citation block.

## What the port registers (and why that is correct)

The port registers NO collision for pickups or corpses — `script_parts` (drone.rs) excludes the
script pickup entities (kinds 0xf0, fix-safe08), drones never join the dynamic set, and pickups are
pure data. Per the chain above that is the game's own behaviour for the player's queries: the
ground ray skips both, the take ray excludes the pickup itself and skips corpses, and bullets skip
`0x60`. Bond standing at floor + 1.1 over a resting drop is the game's standing height.

## The batch-1022l phantom (0.139) — not reproduced

Three probes on this tree found nothing that lifts Bond: the collision sets under two resting
drops (static/dynamic/missile tris in the drop boxes) contain only the floor tiles (y 5.3434..5.3658)
and a real low static kerb near the batch's coordinates (x 6..8, z 18.84..18.91, top 5.5056..5.5507
— outside its z 18.82); `dynamic_boxes` near the spot: none; and the batch's own test flow re-run
here (`fix_pickups_trace_live with_dev_god...`) killed 4 Raptors with every drop taken, one resting
ON that kerb (centre 5.551 = kerb top + ε, Bond over it 6.651 = kerb + 1.1 — correct standing ON
the kerb, take succeeded). The 0.139 lift did not reproduce in any run; per the settled filters
nothing in the game or the port should lift Bond there. If the phantom ever resurfaces, the open
live check is the PCSX2 ground ray behaviour.

## Tests

- `crates/nf-view/tests/drop_stand_live.rs` (NF_ISO): kill the first Raptor, find the resting drop,
  `stand_at` exactly on it → root = floor + 1.1 (±1e-3), the take happens through the real tick
  path within 40 ticks.
- `pickup.rs::the_take_end_band_absorbs_the_rest_noise` (ISO-free): a centre-0 model rests ON the
  floor, sunk 5e-4 → taken (the band); the sister 5.5e-3 inside the plane → refused (a genuine
  short hit). Fail-first: with the old 1e-4 band the first assert fails.
- The existing wall sister stands: `pickup.rs::the_take_ray_origin_decides_the_take` (the 0.5-high
  wall refuses below the top, clears above) — the "drop behind a real wall is not taken" pin.
