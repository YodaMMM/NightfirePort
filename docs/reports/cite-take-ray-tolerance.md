# cite-take-ray-tolerance — the take ray's 1e-3 end band vs the decompile

The task: `crates/nf-game/src/pickup.rs` widens the take ray by 1e-3 at its end and claimed the
game's VU0 fixed-point triangle test "absorbs a sub-quantum 2^-14 contact" (0x18e148's fall rest)
with no line cited. This report census'es the decompile for what actually gives the tolerance (or
the fixed-point quantisation), and fixes the code's claims to match.

## What the decompile shows

**The take-ray chain.** `Pickup_Update` 0x18e148 gates the take on `(bounds+0x8c + 1.1)²`
[0x18e148:203] and `Vec_SqDist3D(pickup+0x80, player+0x30) < that` [0x18e148:210], then calls
`Collide_LineOfSight(player+0x80, pickup+0x80, player cel, pickup obj, 0, 0x721)`
[0x18e148:212-214]. `Collide_LineOfSight` 0x1edf50 is a one-line wrapper —
`Collide_RayIntersect(...) == 0` [0x1edf50:11-12]. `Collide_Intersect` 0x1ebaa8 dispatches per obj
ident: reads `*(ushort*)(obj+0x94)` [0x1ebaa8:180], ident 0x101 → `Intersect_PointGeom` [186-189],
ident 0x201 → `Intersect_RayGeom` [229]. So the take ray's floor-triangle test is
**`Intersect_RayGeom` 0x10a8c8**.

**The fixed-point quantisation — shown.** `Intersect_RayGeom` converts each triangle's three int16
vertex shorts to float (`_psraw 0x10` sign-extend, `_vitof0` int→float) and multiplies by the
constant quad `(0x38800000, 0x38800000, 0x38800000, 0x3f800000)` = **(2^-14, 2^-14, 2^-14, 1.0)**
[0x10a8c8:138-149]. The same 2^-14 quad converts the returned hit position/normal
[0x10a8c8:315-317, 327-335]. Sister in the point test: vertices × 6.103888e-05 (≈ 2^-14)
[0x1e92d8:185-187, hit pos 326-329]. So the static geometry's vertices live on a 2^-14 = 6.1e-5
grid — the quantum in the old comment was real, but it lives in the vertex data, not in a
tolerance.

**The t-range — exact, no band.** `Intersect_RayGeom`'s plane test skips backfaces
(`0.0 <= dot(n,dir)` → skip) [0x10a8c8:168], accepts `0.0 <= t` and `t <= 1.0` [0x10a8c8:171-172],
and takes a candidate only when `t < best`, with `best` initialised to 1.0 [0x10a8c8:95, 135, 180].
Net rule: **a hit blocks the LOS only for t strictly below 1.0; an exact t == 1.0 plane contact
does not block.** The three edge/inside tests are exact sign comparisons (`<= 0.0` twice, then
`0.0 < x → outside`) with no epsilon [0x10a8c8:220-244]. There is no 1e-3, no 2^-14, no epsilon of
any kind on the ray's t anywhere in the function.

**The rest — never sunk by design.** The fall's rest puts the pickup position at
ground-hit point + normal·(−|bounds centre y|) + model centre and sets state 1 [0x18e148:150-173],
so the bounds centre (obj+0x80, the take ray's endpoint) rests ON the plane for a centre-y-0 model
(Raptor probe: |centre.y| ≈ 5e-7 [L probe]) or |centre y| above it otherwise. The game's own rest
never intentionally places the endpoint below the plane.

## Verdict

- The **quantisation** is shown: [M Intersect_RayGeom 0x10a8c8:138-149, 315-317].
- The **end band** is NOT shown. The game's only end tolerance is the exact `t < 1.0` rule (with a
  t == 1.0 contact clear) [M 0x10a8c8:95,135,168-172,180,220-244]. The old claim that the game's
  triangle test "absorbs a sub-quantum 2^-14 contact onto the plane" has no decompile line: every
  comparison in the ray test is exact. A sub-quantum-BELOW endpoint would produce t < 1.0 and
  block, exactly like a deep one; the game's takes work because its rest lands on/above the plane,
  not because the test absorbs anything.
- The port's 1e-3 band is therefore a **port choice** — float-noise slack for the port's own rest,
  which (unlike the game's) can land the endpoint measurably below the plane (batch-1022l measured
  one >1e-4 deep; the take refused). Removing the band (to the game's exact rule) would regress
  that real case, since the port's rest arithmetic differs from the game's. So the band VALUE stays,
  now marked `// [S] unverified` and kept out of game-behaviour claims.

## Changes (branch s/cite-take-ray-tolerance)

- `crates/nf-game/src/pickup.rs` `touches()`: the comment block now cites the chain (0x18e148:203,
  210, 212-214 → 0x1edf50:11-12 → 0x1ebaa8:180-229), the quantisation (0x10a8c8:138-149, 315-317;
  0x1e92d8:185-187), the exact t-rule (0x10a8c8:95,135,168-172,180; edge tests 220-244) and the
  rest (0x18e148:150-173); the "absorbs onto the plane" claim is removed; the 1e-3 band is marked
  `// [S] unverified` at the `segment_take_scoped(...)` line. Behaviour unchanged.
- `the_take_end_band_absorbs_the_rest_noise` doc: reframed as pinning the PORT's own band (a port
  choice, [S] unverified as a game rule) with the same citations. Assertions unchanged (they pin
  the port's `touches()`, not the game).

## Tests run

- `cargo test --offline --release -j2 -p nf-game --lib -- pickup --test-threads=1`: 18 passed,
  0 failed (includes `the_take_end_band_absorbs_the_rest_noise`).
- Full suite via the fast runner `tools/test/suite.py` on the final tree (see the total line
  appended below).

## Live checks for the main agent

The band is [S] until one of these reads settles it (all on a FRESH boot — a save-state resume
skips the boot path and would not re-derive the rest):

1. **Does the game absorb a sub-quantum endpoint contact?** In a level with a droppable weapon,
   drop it on a flat floor, let it rest, stand Bond in take range. With `tools/trace/gdbwatch.py`
   on the return of `Collide_LineOfSight` 0x1edf50 (called every tick from 0x18e148:212 while in
   range), and `tools/trace/fieldprobe.py` reading the resting pickup's obj+0x80 (the ray endpoint)
   plus the floor triangle's three int16 vertices (×2^-14) from the cel, compute the endpoint's
   signed depth below the plane in double precision. Expected if the current [S] claim is right:
   LOS returns clear with the endpoint measurably below the plane by up to ~2^-14. Expected if the
   decompile's exact rule is the whole story: LOS returns clear only when the endpoint is on/above
   the plane, and the game's takes survive because the rest lands there.
2. **Measure the game's actual end band by poke.** Same setup; poke the resting pickup's position
   (obj+0x30 y, and obj+0xc0 if obj+0x80 tracks it) downward in steps (one quantum 2^-14, then
   1e-4, 5e-4, 1e-3), reading 0x1edf50's return after each step. The largest downward poke that
   still returns clear (take fires) is the game's measured end band; if even one quantum blocks,
   the band is zero and the port's 1e-3 stays a pure port choice (as marked).

## What's left

- Nothing blocking: the citation task is done on the branch. The [S] marking stands until the main
  agent runs one of the live checks above; if check 2 shows the game absorbs up to some band, the
  port's 1e-3 can be re-anchored to that measurement.

## Usage

not visible to me — this environment exposes no token, credit or context-window figures for the
session (no input/output token counts, no cost readout anywhere in the tool output).
