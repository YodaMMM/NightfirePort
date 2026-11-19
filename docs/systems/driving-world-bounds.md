# The driving world-boundary sheets (DRIVING.ELF `unk_331A60`) — G-056

The invisible walls of the driving maps. Decoded from the IDA database of `work/elfs/DRIVING.ELF`
(sha1 `d7b8a57f…`, IDB md5 `ab8e4841…`, base 0x107000). Every code fact below is [M] (address +
role, checked in the disassembly/pseudocode); the table values are [M] (bytes read from the IDB at
0x331A60); inferences are [S]. This corrects `drive-physics.md` §4, whose ":870-975 spring-damper
colliders" reading was wrong — see the correction note there.

## 1. The static table

`unk_331A60` holds **19 records of 0x50 bytes** (the scan is a counter-18..0 walk [M 0x192130,
`LABEL_135`], and `sub_190EB8` walks `+$s0 += 80` until 19 [M 0x190f58-0x190f5c]):

| offset | type | meaning |
|---|---|---|
| +0x00 | f32×4 | corner 0, `(x, y, z, 1.0)` [M loads at 0x190e28-0x190e2c edge walk] |
| +0x10 | f32×4 | corner 1 |
| +0x20 | f32×4 | corner 2 |
| +0x30 | f32×4 | corner 3 — the plane test's reference point [M `lqc2 $2, 0x30($s0)` 0x192128] |
| +0x40 | f32×3 | plane **normal** (unit-ish; the walls' y carries float noise ~±1e-8) [M `lqc2 $2, 0x40($s0)` 0x192138] |
| +0x4C | f32 | **thickness** — the response band [M compare `_$A0 >= *($s0 + 76)` 0x192184] |

One record = a convex quad sheet in world space plus the plane normal pointing INTO the allowed
side: a point is "colliding" when `d = (p − corner3)·normal` sits in `[0, thickness)` AND `p`
projects inside the quad [M the two-stage test below].

Contents [M, bytes at 0x331A60]:

- **Records 0-4 — horizontal slabs** (normal `(0, −1, 0)`): ceilings above drivable bands at the
  per-map altitudes — rec0 at y = 78.438 (thickness 10), rec1/2 at y = −240.477/−240.482 (15),
  rec3 at y = −260.330 (15), rec4 at y = −95.460 (15). The five missions' maps sit at different
  world origins, so one static table serves them all: a car only ever enters its own map's sheets
  [S — consistent with the disjoint altitudes and the absence of any writer to the table: the
  only two data references are the two readers, `sub_190EB8` + `sub_190F88` @0x190edc/0x192114
  [M-negative xref scan]].
- **Records 5-18 — vertical and slanted walls** (normals in the xz plane, y noise): a ring of 14
  sheets spanning y ≈ [−64, 216] around x ∈ [−51, 454], z ∈ [−140, 720] — the drivable area's
  boundary walls (thickness 10/11).

## 2. The sheet test

Per car per tick, after the wheel-force pass, gated on `actor+0x2C8 != 2 || sub_195A30(actor) > 0`
[M 0x1920e8-0x19210c; the +0x2C8 state word's writer is unpinned [S]], the loop walks all 19
records with the car's position `inst+0x10` [M `lqc2 $1, 0x10($s2)` 0x192130-0x19214c]:

1. **Plane stage** [M 0x192150-0x192184, the same shape in `sub_190EB8` 0x190ee8-0x190f28]: two
   VU0 macro pairs — `SUB` of the position and corner 3, then the mul+accumulate dot against the
   normal (the raw `cop2 0x1C208EC`/`0x1C208EA` + `0x10318C1`/`0x10318C2` sequences decode by
   role; exact per-lane macro bodies not lifted) — giving `d`. `d < 0` ends the record (the point
   is past the allowed side); `d >= thickness` skips it [M 0x192184].
2. **Quad stage** `sub_190E08` [M 0x190e08-0x190eb0]: for each edge i (0..3, corners wrap
   `(i+1)&3`): `edge = c[(i+1)&3] − c[i]`, `to = p − c[i]`, and the cross-products dot with the
   plane normal; any `< 0` rejects the point [M the `v9 = v0 < 0.0` break at 0x190e94]; four
   passes return "inside".

The first sheet that passes both stages is the response's sheet (loop order 0..18, `LABEL_134`
decrements [M 0x1923b4]).

## 3. The response (penalty force + scrape torque) — `sub_190F88` @0x19219c-0x1923a8

With `v = *(inst+0x20)` (velocity), `f = *(inst+0x60)` (the force accumulator, §6 of
drive-physics.md), `mass = *(inst+0x4C)` [M drive-physics.md §6], `n` the sheet normal,
`t` its thickness, and `dt = flt_339F4C` (the per-frame delta-time global [M: added into timers
as the frame step at 0x20ea3c, scaled into durations at 0x1591dc; it sits beside the pinned
1/60 at `DAT_00339F50` and reads 0 in the IDB — runtime-written [M-negative]]):

1. **Approach speed** `v_n = −(v·n)` [M 0x1921bc: `_$F3 = −dot(inst+0x20, n)`].
2. **Clamps** [M 0x1920e0-0x192230]: `scale = max($f0 / $f2, 0.1)` (`$f25 = 0.1` loaded at `0x1920fc` `0x3DCCCCCD`) then, if `−(v·n) >= 0`, `max(1.0, v_n)` (`$f23 = $f21 = 1.0` from the `0..1` wheelspin phase wrap at `0x192098`) else `min(-1.0, v_n)` (`$f24 = -1.0` at `0x19210c`).
3. **Linear spring** [M 0x192234-0x192254]: `F_lin = n · (dt · v_n_clamped · scale)` where
   `scale = max($f0 / $f2, 0.1)` [M 0x1921e8-0x1921ec], then the accumulated inward acceleration is removed:
   `F_lin −= n · (−(f·n)/mass · scale)` [M 0x192248: `−dot(inst+0x60, n)/mass`; the two vectors
   are summed via the `cop2 0x1C408D8` add at 0x192258-0x192278].
4. If the summed vector still points into the allowed side (`·n > 0` [M 0x1922a0]), it is scaled
   by `$f4`-turned-`$a0` and applied: **`sub_1F87D8(inst, F)`** — the force API of
   drive-physics.md §6 [M 0x1922c4].
5. **Scrape response** [M 0x1922d0-0x1923a8]: `p2 = *(inst+0x5C)` is a POINTER to a paired body
   (its `+0` position vec4 is read — the double-buffer twin or attached static [S]; the writer of
   +0x5C is unpinned). With `sp = max(*(car+0x288), 10.0)` (`$f21 = 10.0` loaded at `0x192118` [M 0x1922f4-0x1922f8]): `mag1 = sp · |v·n| · |n.y|` (n.y read at sheet +0x44 [M 0x192318]),
   `mag2 = $f0' · |v·n| · |n.z| · 1.5` (n.z at +0x48, sqrt at 0x192324-0x192348), the local
   vector `(±2·mag1, ±mag2, 0)` — x flipped when `v·n > 0` [M 0x192350], y flipped when
   `dot(p2+0, n) < 0` [M 0x192398] — goes through **`sub_1F84D0` and `sub_1F8888`** (the
   accumulator and torque twins [M 0x1923a0-0x1923a8]).

Net behaviour: crowding within `thickness` of an invisible sheet returns a dt-scaled push along
the normal and damps the slide along it. The VU0 integration of the applied force is the
drive-physics.md §6 macro (still un-lifted, phase-2).

## 4. The standalone lookup — `sub_190EB8` (0x190eb8)

A point-in-which-sheet helper with the identical two-stage test, returning (in `$s2`) the index
of the first sheet whose band contains the point [M 0x190f28 break / 0x190f33 counter], and the
`sub_190E08` quad check [M 0x190f39]. Callers: the tyre pass inlines the same loop, so this
wrapper's caller set is separate (its consumers are [S] — mission code or the camera; not
traced yet). The tyre pass passes `inst+0x10` where the wrapper takes the point directly [M
0x192130 vs 0x190ee8].

## 5. The port

`crates/nf-game/src/driving/bounds.rs`: the 19 sheets as `ZoneQuad` constants (the exact IDB
values), `signed_dist` / `inside_quad` / `hit` / `first_hit` as §2, and `boundary_force` with the
§3 form taking the unpinned scalars (`scale`, the clamps) as arguments — so a live trace can pin
them without reshaping. Wiring into `physics::Car`'s step and the per-map sheet subset (§1's
origin question) is the mission-runner work, not this module.

Tests pin: the table shape (19 records; rec0 the y=78.438/10 ceiling; rec5 the z=419.897 wall;
the slanted ring normals), the plane-band arithmetic, the 4-edge inside test (inside, outside
per-edge, wrap edge i=3), the scan order, and the force direction (push along +n for an
approaching body) — each with a sister assertion that proves the comparison can fail.
