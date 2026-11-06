# level-0a-west-hall — the west hall doorway (the two ignored walkreal 0a tests)

Task: `the_west_hall_glide_height_matches_the_game` (settled y 1.0905 on master vs the game's
1.1285) and `the_corner_crosses_like_the_game` (the port wedges, the game slides through).
Start points given: `Player_Collision` 0x194df0, `Player_CollisionHandler` 0x1949a8. Outcome
this window: **the measured next lead** — the push tail is proven dead on 0a, the corner
wedge's own mechanism is identified as a port-only device, and the game's y gap re-reads as a
y-hold plus a doorway acquire whose writer is pinned down to two live-check candidates. No
game code changed; both tests stay ignored with the notes re-pinned.

## What was done

**1. The decompile chain, with addresses.** `Player_Collision` 0x194df0 sets the body capsule
per substate: sphere A = obj+0x30 + up·0.275 (0x3e8ccccd) into BLData+0x760, sphere B =
obj+0x30 + up·(0.55 − cc) into BLData+0x770, radius 0.55 (0x3f0ccccd) into +0x7cc, shape 0x800
into +0x7d4, where `cc = *(float *)(*(obj+0xdc) + 0xcc)` is read fresh per call [M
0x194df0:86]. `Player_CollisionHandler` 0x1949a8 runs `Player_FeetOnPoint` (the floor ray into
the BLData+0x740 HITTEST), then — when the hit list (obj+0xd0) is non-empty — applies the
push accumulator `obj+0x30 += BLData+0x750` and dispatches each hit through
`Player_DealWithObjHit` 0x194458 [M 0x1949a8:136-152]. `Player_DealWithObjHit` is pain-only
(classes 5 / 0x27 / 0x2c / 0x2d → HandlePain / CollWire / CollLadder; everything else
returns 0) — it is not the wall response. The wall response would be `Intersect_CylGeom`
0x109da0's push tail (type 0x800, dispatched by `Collide_Intersect` 0x1ebaa8 off HITTEST+0x94,
fed per tick by `Collide_Update` 0x1ea398: `t+0x9c = 0`, `Collide_Pick`, one
`Collide_Intersect` per picked object, the list sorted nearest-first into obj+0xd0).

**2. The push tail is PROVEN dead on 0a (this window's new [L]).** The boot body HITTEST was
located in `ee_start_0700000a.bin` by signature (r 0x3f0ccccd at +0x8c, shape 0x0800 at +0x94,
B−A y = −0.7597): EE **0x1b78e90**, spheres A = (−3.93, 1.3641, 10.40), B = (−3.93, 0.6044,
10.40) — the spawn pose. Its ref +0x50 (**0x1b78ee0**) is all-zero [L]. The ref's only writer
is the `Collide_Intersect` add-back (+0x10/+0x20/+0x30/+0x50/+0x60 += `_DeltaP` [M 0x1ebaa8
tail]), `_DeltaP` is zeroed per `Intersect_CylGeom` call and produced only by the tail's own
push arm (`|closest − ref| < r` [M 0x109da0:136, :447-457]), and each push moves the ref at
most r. From a (0,0,0) boot the ref can therefore never leave a ~0.55-ball around the origin,
while every 0a doorway/corner contact sits |closest| ≈ 10-25 away: **the tail cannot fire,
ever, on 0a** — not merely "is dead at boot". `Player_FeetOnPoint` 0x194188 was re-read in
full: the floor ray from obj+0x30 down `cc + 0.6` [M :88-100], the band `hit − 0.1 <= cc`
[M :106] writing BLData+0x110 = hit+0x14, the floor hits added to obj+0xd0 with the up normal
[M :138-155], and the grounded flag from t+0x9c [M :176-185] — a tolerance, not a snap; no y
writer.

**3. The port's doorway trace (NF_CAPS_LOG on the through-line).** Every jamb contact pushes
HORIZONTALLY, and the reason is structural: the walk spine is vertical, so the closest
spine point shares q's y and `v = p − q` is horizontal (q = (−8.304, 0.733, 16.279) →
p = (−7.791, 0.733, ·), push (0.0255, 0.0000, 0.0053) on the jamb base inner face). The
through-line crosses at the game's rate (230 ticks to (−9.93, 17.08)) and settles
**1.0905** at (−9.935, 17.020) — the port never lifts. Note the push magnitude 0.0255 ≈ the
missing +0.0256 is coincidence of the contact geometry, not the lift.

**4. The corner wedge's own mechanism identified.** The wedge is the port's spine-dir
fallback push (`capsule_tri_contact`'s `(spine_dir, r − dist)` arm — a port [S] device behind
the dead radial arm) lifting at below-B edge contacts (+0.0165/tick [L, the -2 probe]),
dithering y across the band edge 1.1347 and wedging the walker. The game cannot produce that
displacement (the dead tail above). Measured with the port's own env-gated below-B
displacement kill (`NF_WALK_KILL=1`): the corner leg **crosses** — no wedge, no dither — but
not as the game does: 836 ticks from the wedge corner and the route runs on into the
level-exit trigger (`stop_on_exit` fired), while the hall legs stay y 1.09. So the kill alone
does not make the corner test pass (the ≤300-tick walk-speed pin and the 1.1159 exit-y pin
both still fail), and defaulting it re-paths the 01 plate approach
(`walk_from_the_spawn_to_the_exit_with_the_contact`'s AND-touch never fires — the earlier
session's measurement). It stays env-gated.

**5. The game's y gap re-read.** The game's numbers are a y-HOLD plus a doorway ACQUIRE: the
corner probe placed at 1.12 stays 1.1186 → 1.1159 [L 2026-10-26] (no downward walk settle);
the glide acquires +0.038 at the doorway and holds 1.1285 steady across frames 21/28/36 and
the post-release rest [L]. Both game walked settles sit just INSIDE the band's top edge
(hit − 0.1 = 1.0285 / 1.0159 <= cc 1.0347 [M 0x194188:106]) while the spawn rest sits 0.045
inside — consistent with the game's walked y growing against the band ceiling on the flat
hall slab [L band-settle finding 1] and resting just under it. With the body anim a 2-frame
stub [D 0x2b91b0] cc cannot bob, so the acquire/growth writer is not yet pinned.

## What is left

The doorway acquire's writer (the +0.0256..0.038 lift) and the game's walked wall response
(the xz slide). Both need the live checks below; the corner test additionally needs the
game's walk-speed slide mechanism before its tick pin can be met.

### Live checks for the main agent

1. **BL+0x790 (HITTEST+0x50 ref) per frame** through the doorway brush and the corner brush
   (watch, 4-byte float ×3 at the player's BLData+0x790; the 0a BLData is the obj whose
   +0x760/+0x770 hold the capsule). Expected if this report is right: stays ~(0,0,0) or
   moves only by ≤0.55 pushes — pinning the tail dead LIVE and pointing the wall response
   elsewhere. If it jumps to the sphere position, the tail is alive and this analysis is
   wrong.
2. **AnimObj+0xcc (obj+0xdc → +0xcc) per frame** during the walked glide from the lobby
   through the doorway into the hall. Expected: static 1.0347 (the stub anim) — or moving,
   which pins the acquire on the root ride.
3. **The walk move quad BLData+0x10..0x1c per frame** through the doorway brush. Its y lane
   (+0x14) passes the yaw-only matrix unrotated [M 0x193600:284-295]; a nonzero +0x14 during
   the doorway transit pins the acquire on the move quad's y lane.
4. **obj+0x30.y per frame** across the doorway (x −8.6..−8.3): the acquire's shape — a step
   at the sill (the 0.19 base pieces) vs a ramp — discriminates a floor-band mechanism from
   an anim/move one.

## How this was verified

- walkreal_0700000a_live whole: **13 passed / 0 failed / 2 ignored** (the two task tests).
- walkreal_07000002_live whole: **3 passed / 0 failed / 1 ignored**.
- walkreal_07000005_live whole: **14 passed / 0 failed / 0 ignored**.
- The glide test run with NF_CAPS_LOG=1 (the doorway trace above); the corner test run with
  NF_WALK_KILL=1 (the crossing measurement above). No source change landed, so no other
  binary's counts move.
- The full suite did not fit this window (the standing chunk ceiling from the stream-batch
  sessions); with no `crates/*/src/` change the suite exposure is nil.

## Usage

not visible to me

## ntfy failed

All three fallbacks failed (2026-11-06 ~19:36 UTC):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "level-0a-west-hall: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds`, exit 28.
2. python3 urllib (fallback 2) → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`, exit 1.
3. `wget -q -O- --post-data "level-0a-west-hall: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → killed at the 30 s tool timeout with no output (same unreachable network).

