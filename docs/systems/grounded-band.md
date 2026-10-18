# The grounded band (Player_FeetOnPoint) — the game's exact f32 chain

How the game decides "on the ground", and the port's bit-mirrored reading of it. This is the
level-14-interactables-2 re-land: 717a's `GROUND_EPS = 1e-4` slop is gone; every step below is
the game's own arithmetic, mirrored.

## The ray (Player_FeetOnPoint 0x194188)

`Player_Collision` 0x194df0 builds the capsule into the BLData+0x740 HITTEST: the radius goes to
+0x7cc, the A arm to BLData+0x760 and the **B arm to BLData+0x770** (`pos + up·offset`, the tail
store 0x194df0:405-408). Per substate (the switch indexes `obj+0xF6 − 1`):

| substate | A offset | B offset | radius |
|---|---|---|---|
| case 3 (crouch walk / the port's swim + crouch) | `0.1 − (cc − 0.55)` | `0.45000002 − cc` | 0x3f0ccccd (0.55) |
| case 0/6 (climb) | `0x3e8ccccd` (0.275) | `+0x7cc − cc` = `0.275 − cc` | 0.275 |
| default (standing) | `0.275` | `0.55 − cc` | 0.55 |

`Player_FeetOnPoint` 0x194188 then builds the floor ray **from the B arm**, not from the root:
it loads `HITTEST+0x30` (= BLData+0x740+0x30 = BLData+0x770, the B arm) [M 0x194188:89], computes
the scalar `−(cc + 0.6)` [M :92, `fVar11` = BLData+0xcc], adds `up · scalar` [M :96-97] and calls
`Collide_RayIntersect(obj+0x30, that endpoint, obj+0x20 cel, …)` [M :99-100]. The **from** is
`obj+0x30` (the root), the **to** is `B + up·(−(cc+0.6))`. Upright the matrix up is (0,1,0), so
the endpoint is `B.y − f32(cc + 0.6)` with unchanged x/z. For the crouch (`cc` = 0x3f0ccccd) the
delta lands **bit-exactly −1.25** (0xbfa00000; probe14_ground.rs prints it), not the −1.15 a
root-based ray would give.

## The distance (Intersect_RayGeom 0x10a8c8)

`Collide_Intersect` 0x1ebaa8's type-0x201 (ray) branch recomputes `dir = to − from` in f32 and
stores it into the test copy's +0x40 [M 0x1ebaa8 0x201 branch]; `Intersect_RayGeom` then:

1. intersects the quantized (1e-4 fixed-point) collision geometry and keeps the nearest `t` in
   [0,1] [M 0x10a8c8:168-172];
2. stores the **hit point** `from + dir·t` (a componentwise f32 mul then add) [M :176-179, written
   to HITDATA+0x40 at :252-256];
3. stores **HITDATA+0xC = SQRT(Σ(from − hit)²)** — the absolute Euclidean distance from the ray
   origin to the hit point, with the VU pairwise-add order (x+y then z+w) [M :312-325]. This is
   the field every reader sorts by (`CompHitData` 0x1d33c0 sorts nodes by +0xC [M
   docs/fields/hitdata-spawner-ally.md:21]).

**HITDATA+0xC is not `t · raylen`.** The port's old `t * ray` at the call site was a different
float expression of the same analytic quantity, and at the crouch rest the difference is exactly
the grounded band.

## The band (0x194188:106)

```c
if ((*(float *)(hit + 0xc) - 0.1 <= cc) && …)   /* [M 0x194188:106] */
```

One f32 subtraction of 0.1f32, then `<= cc` (BLData+0xcc). No epsilon anywhere.

## Why the crouch rest is the exact case

At a hard-floor crouch rest the B sphere sits on the floor: the contact distance below the root
is `|B offset| + 0.55f32`:

- game B offset `0.45000002f32 − 0.55f32` = **−0.09999999403953552** (0xbdcccccc) → contact
  ≈ 0.650000006 — one ulp **inside** the band's f32 edge;
- the old port B offset `0.45f32 − cc` = **−0.10000002384185791** (0xbdccccd0) → contact
  = 0x3f266667 (0.6500000358) — exactly ON the edge: `0x3f266667 − 0.1f32` rounds to exactly
  `cc` (band holds), but `0x3f266668 − 0.1f32` rounds above it (band fails) — and the capsule
  push's iterated equilibrium dithers ±3 ulps of `pos.y` (probe14_ground.rs), so the old port
  flickered grounded/ungrounded at every hard-floor crouch rest (the 07000014 crawl-vent start
  [D p3=3]; [L] the crouched Bond crawls out of the vent on PCSX2).

The f32 edge table (probe14_ground + direct evaluation):

| hit (bits) | `hit − 0.1f32` (rounded) | `≤ cc` (0x3f0ccccd)? |
|---|---|---|
| 0x3f266666 (0.65f32) | 0.5499999523 | true |
| 0x3f266667 | 0.5500000119 = cc | **true (edge, inclusive)** |
| 0x3f266668 | 0.5500000715 | false |

## The port

`crates/nf-game/src/player.rs`:

- `capsule()`: the crouch/swim arm uses the game's literals in the game's order —
  `(0.1 - (cc - 0.55), 0.450_000_02 - cc)` [M 0x194df0:235/248 + the tail store :405-408]. The
  algebraic simplifications (`0.65 − cc`, `0.45 − cc`) are different f32 values — that was the bug.
- both grounded sites (the walk tick and `physics_tick_climb`): the ray endpoint is
  `b[1] + -(cc + 0.6)` from the capsule B [M 0x194188:89-98], and the band is
  `((pos.y − (pos.y + t·(to.y − pos.y)))²).sqrt() − 0.1 <= cc` — the HITDATA+0xC chain
  [M 0x10a8c8:176-179/252-256/312-325 + 0x194188:106]. `GROUND_EPS` is deleted.

Deviation kept and documented: the port's ray `t` comes from its Möller–Trumbore loop
(world.rs `segment_hit*`), the game's from the precomputed quantized plane division in
`Intersect_RayGeom` [M 0x10a8c8:169 `fVar18 = -fStack_f0 / auStack_100`] — the same analytic t,
rounded differently (≤ ~1 ulp). After the B-arm fix the rest sits 1 ulp inside the band edge, so
the t rounding no longer straddles it [S: measured on 07000014's crawl start; the plane-quad
mirror is the follow-up if another edge case appears]. For a tilted actor the game's up vector is
the matrix column, the port's is world-down — pre-existing, and the band has 0.1 of margin there.

## Verification

- `fix_input_kbd_trace::keyboard_trace_unchanged` — bit-identical (the 717a merge hold; the
  standing band has 0.1 of margin, so the mirrored chain never flips a standing tick).
- nf-game: 63 binaries, all ok (281 lib tests). nf-view swim (entry/live/vent5/vent), ladder
  (live/2/water/02/underwater-02), `walk_07000014_live` 2/0, `walkreal_07000005_live`,
  `walkreal_07000012_live`, `probe14_vent` — all ok with the epsilon gone.
- probe14_ground.rs (assertion-free): the rest y 1 ulp lower than 717a's tree, contact
  0.6499999/0.64999986, band true, stable across the push dither.

## The band gates BLData+0x110 too (level-14-interactables-3)

The band is not only the grounding condition — it is also the gate on the OTHER FeetOnPoint
output: `param_3` (= `BLData+0x110`, the port's `floor_ny`). FeetOnPoint writes
`*param_3 = 1.0f` first [M 0x194188:85] and overwrites it with the band-passed hit's `+0x14`
(the surface normal y) **only inside the band** [M 0x194188:106-109]. The gravity gate reads
that value: `Player_Collision` 0x194df0 applies gravity when `flag8 == 0 || BL+0x110 < 0.5`
[M 0x194df0:104] — a steep surface only pulls Bond off his feet when the band PASSED on it.

The port's `floor_ny` took the hit normal regardless of the band. That was harmless while the
ray was `pos − (cc+0.6)` (master) — a hit past the band edge was rare — but the B-hop ray
reaches ~0.48 further down (standing: `0.55f32 − cc` = −0.4847183 of extra reach), and on the
07000005 upper ledge it now hits the ny 0.45 slope ~1.85 below the walkable rest every tick.
The port then fed 0.45 into the gravity gate on ticks Bond was walkable-grounded, gravity fired
and he slid off the ledge (the 717c merge hold:
`interact_07000005_live::the_ledge_plate_plays_the_sound_trigger`). In the game the band FAILS
at that rest (1.85 − 0.1 = 1.75 > cc), so `BLData+0x110` stays 1.0 and the gate never opens
[M 0x194df0:104].

Flag-8 detail from the disassembly (the grounding flag FeetOnPoint sets at 0x194188:176-185):
flag 8 = `HITTEST+0x9c != 0 || band passed`. `HITTEST+0x9c` is written by `Collide_Intersect`
(read-modify-write at 0x1ec138-0x1ec150) **only when `HITTEST+0x94 & 0x200`** — but
`Player_Collision` stores 0x800 into `HITTEST+0x94` every tick [M 0x194df0:204/228/256/…], and
`0x800 & 0x200 == 0`, so for the player's feet ray the OR-store is skipped: ordinary world
triangles never set +0x9c. The one unconditional writer is `Intersect_CylGeom` (+0x9c |= 1 at
0x10a72c-0x10a734) — cylinder hits (poles) mark the ray. Net: for the player, flag 8 = the band
passed (triangle floors) plus the cylinder case; the port's `walkable` push net stays as the
documented port-scope stand-in for the game's per-cel geometry the port's buckets miss [S].

The port's fix (both grounded sites): `floor_ny = band ? hit ny : 1.0` — the game's
`BLData+0x110` chain, bit-mirrored. `grounded` keeps `walkable || band`.
