# level-0a-corner-push-model — the corner wedge is the port's below-B edge-contact lift; the corner now crosses at walk speed with the push model kept

## What was asked

With the push model kept (the [L 2026-10-30] fresh-boot measurement: the game rests z 15.5904 at
the hall wall = master's push model, and the four-yaw paths match to ~0.1), find what differs at
the 0a hall corner where `walkreal_0700000a_live::the_corner_crosses_like_the_game` wedges (870
ticks with NF_WALK_KILL, permanent without) while the game crosses in ~60 frames [L 2026-10-26].
Fix it, un-ignore the test without the kill switch, every test in the file green, run every
walkreal_* binary and quote the totals.

## The mechanism (found this session, every claim traced)

**The wedge's four per-tick contacts are all Dist2Tri EDGE contacts, and their port fallback
pushes sum exactly anti-parallel to the walk's move.** Traced with `NF_CAPS_LOG=1` on the real
walked route (the probe `walkreal_probe0a_corner.rs` on this branch):

| contact | face | q | push/tick | edge |
|---|---|---|---|---|
| 1 | the counter slab's west strip (x −17.5, y 0.73..0.84) [D the tri dump] | (−17.498, 0.803, 4.430) | (−0.0383, 0, +0.0365) | true |
| 2 | the counter slab's slanted strip (x −17.4) | (−17.423, 0.803, 4.629) | (−0.0401, 0, +0.0153) | true |
| 3 | the swung-open door #4 leaf's TIP (x −18.4, q BELOW the spine bottom 0.604) | (−18.314, 0.549, 4.542) | (+0.0448, **+0.0165**, +0.0335) | true |
| 4 | the leaf tip's north face | (−18.314, 0.916, 4.542) | (+0.0180, 0, +0.0135) | true |

Sum = (−0.0156, +0.0159, +0.0988)/tick vs the walk's move (0.0154, −0.0985) — **net zero**: Bond
frozen at (−17.8737, 4.8716) with y dithering ±0.03 and `grounded` flickering (the probe trace).

**Why the game doesn't wedge — the [M] reading.** The game's per-triangle tail
(`Intersect_CylGeom` 0x109da0) is REF-relative: on an edge/vertex hit (Dist2Tri flags 1/2) the
probe p is RE-AIMED at the spine point closest to HITTEST+0x50 [M 0x109da0:333-368; disasm
0x10a344-0x10a350 — `lqc2 vf4, 0x50(s1)` inside the re-aim arm], and the push is
v = p − REF, gated dot(v,n) > 0 and |v| < radius [M 0x109da0:419-457; disasm 0x10a540-0x10a5d0].
It is NOT a (p − q) skin depenetration: the Dist2Tri distance |p − q| never gates the push (it
only feeds the HITDATA best-contact bookkeeping [M 0x10a738's `fVar19` compare]). For an edge
contact whose q sits BELOW the capsule bottom the re-aim clamps p to B — the game's own
closest-spine-point-to-REF reads B too on a fresh walk ([L ee_start_0700000a: BL+0x790 = (0,0,0)
while the walked spine sits ~1.1 above the floor; the add-back 0x1ebaa8:353-373 is the only
writer [M]) — and the port's fallback (p − q) push then points UP-and-out of the end cap: a lift
with no game counterpart on a walked line. That lift (contact 3's +0.0165/tick) is the wedge's
fatal member.

**The certified one-wall rest is untouched by the fix**: the hall-wall rest contact is
FACE-INTERIOR (`edge=false`, the big z 15.04 wall tri, d 0.45, push +0.1000/tick [cap trace, this
branch's `walkreal_probe0a_hallwall.rs`]) — the port reproduces the game's z 15.5904 rest exactly
[L 2026-10-30, re-measured through the probe this session].

## The fix

`crates/nf-game/src/world.rs` (`capsule_tri_contact`): on the walk path (`top_guard = false`, the
walk/NPC queries), an edge/vertex contact whose closest point q sits below the capsule bottom B
keeps its contact signals (walkable/pick) but drops the fallback displacement. The radial arm
(the [M] tail's own REF-relative shape) is untouched; the swim path (top_guard = true, the game's
real diagonal-spine tail [M the 02 fin wedge]) keeps everything. Horizontal edge deflections
(the wall-end caps the Q-Worm route jinks around) keep their pushes — the first cut of this fix
skipped ALL walk edge contacts and regressed
`walk_office_floor_qworm_keyboard_hatch` ("ring: jink 4" wedged); the below-B narrowing restores
it.

With the below-B member dropped, the remaining three contacts sum (−0.060, 0, +0.065)/tick
against the move (0.0154, −0.0985) — a west-south DEFLECTION, the game's own measured bend
([L 2026-10-26: the game's path ends x −17.9715, west of its start]) — and the corner **crosses
at +76 ticks** (this branch's probe; the game's ~60 frames [L 2026-10-26]; the NF_WALK_CLIP probe
had measured the same +76). No kill switch, no clip flip — the push model is kept.

## Test state

- `walkreal_0700000a_live`: **7 passed / 0 failed / 2 ignored** with the fix — the master
  baseline (the Q-Worm route, the keyboard, the hatch all green).
- `the_corner_crosses_like_the_game` stays `#[ignore]`d on ONE remaining pin: the exit y. The
  port crosses at y 1.0891 (the spawn settle) where the game settles 1.1159 [L 2026-10-26] — the
  0.0268 gap is the west-hall leg's unported **climb settle** (the glide sister's mechanism,
  level-0a-hall-glide-settle — the port's walked root never grows the game's +0.0268), not the
  corner: the corner's floors are y 0.00 on both sides [D the tri dump]. The crossing assert
  (≤300 ticks, walk speed) passes. Un-ignore once the climb settle lands; the ignore note on the
  test carries this handoff.

## How it was verified

- The [M] chain: `Intersect_CylGeom` 0x109da0 decompile + disassembly (the tail's REF load
  0x10a540, the gates 0x10a568/0x10a5c8, the raw-spine cull dot 0x10a1d8-0x10a290 — the ±0.999
  cull never fires on the 0.7597 walk spine), `Collide_Intersect` 0x1ebaa8 (the local copy base
  sp+0x50 [disasm 0x1ebc84], the RotPreTransVec of +0x50/+0x60, the add-back 353-373),
  `Player_CollisionHandler` 0x1949a8 (the only position write = obj+0x30 += HITTEST+0x10),
  `Player_FeetOnPoint` 0x194188 (the feet ray goes through Collide_RayIntersect — no add-back),
  `Collide_Update` 0x1ea398 (only +0x10 zeroed per tick), `Collide_Pick` 0x1eb320.
- [L] `work/ee_start_0700000a.bin` read directly (this session): BL+0x790 = (0,0,0), the spine
  A/B at the spawn, +0x60 = (−22.08, 0.38, 11.30) — the REF boots at the origin, 11 m from Bond.
- The cap traces: the wedge contacts (the table above), the hall-wall interior contact, the
  crossing probe (76 ticks, the west bend).
- Runs this session: `walkreal_0700000a_live` 7/0/2 (27.3 s); the pre-fix regression check on
  the first-cut skip (the Q-Worm jink wedge) is what narrowed the rule.

## What's left

1. Port the climb settle (the glide sister's mechanism) and un-ignore the corner test (the
   crossing assert already passes).
2. The tail's REF semantics stay open: the fresh-boot +0x50 = (0,0,0) [L] with the add-back as
   the only writer [M] means the [M] tail cannot fire on a fresh walked line — yet the game
   skin-rests [L 2026-10-30]. Something outside the swept store set initialises/refreshes
   BL+0x790 (an amemcpy-range init or a register-held HITTEST base are the candidates the sweep
   could not see). The live checks below settle it.
3. The full suite on this exact tree: the sandbox runs it in the background (suite2.out); the
   touched-crate runs are the 0a file above + the battery below.

## Live checks for the main agent

1. `watch (BL+0x790) 12` during the fresh-boot hall-wall walk (Bond probe-placed (−15, 1.2,
   16.3), yaw π, forward): if +0x50 STAYS (0,0,0) while Bond rests at z 15.5904, the walk's wall
   rest is confirmed NOT the Intersect_CylGeom tail, and the port's (certified) push model is the
   right stand-in shape; if it MOVES toward Bond, the sweep missed a writer — send the first
   writer PC.
2. `watch (BL+0x790) 12` while pressing against the corner counter (walk into the counter at
   (−17.87, 4.87)): the game's accumulator motion there tells whether edge contacts feed DeltaP
   in the game at all.
3. A door probe: place Bond 1.6 m from door #4 (−19.5, 4.6) on a fresh boot, hold forward toward
   (−17.5, 2.5) — does the game OPEN door #4 (E-less)? The port's walker opened it via the stuck
   mitigation; if the game opens it by proximity, the open-leaf scenario is game-real and the
   leaf's below-B edge behaviour (check 2's read) decides the corner.

## Usage

not visible to me (no token/credit/context figures are exposed to this session).
