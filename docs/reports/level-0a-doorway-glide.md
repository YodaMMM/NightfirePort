# level-0a-doorway-glide — both reds reproduced on master; the game-side push pipeline read [M] and confirmed shape-faithful in the port; the walk-vertical named as the open surface; the ledgers carry the live checks

## What is done

**Setup.** Branch `s/level-0a-doorway-glide` from `origin/master` 784ba2d99 (the sandbox
carried the repo, `work/`, the ISO and the toolchain from the previous window; the branch was
created, the WORKLOG entry committed and the branch pushed within minutes).

**Both ignored tests reproduced red on master** [L the runs on this tree]:

- `the_west_hall_glide_height_matches_the_game`: the settle reads **1.0905** at (-9.935,
  17.020) after 230 ticks against the game's pinned 1.1285 [L 2026-10-26] — the assert red at
  0.038 short.
- `the_corner_crosses_like_the_game`: the crossing **arrives** but takes **6000 ticks** (the
  assert <=300 red) — Bond parked at the wedge corner (-17.87, 1.17, 4.87) the whole cap,
  **grounded false** (the band-edge dither the ledger names), and only the walker's stuck
  mitigation finishes the leg [L corner.out].

**The game-side mechanism read [M], end to end** (the task's named pieces):

- `Player_Update` 0x193600:284-295 integrates the walk move unclipped into obj+0x30; the walk
  matrix is the full product (`Player_SSWalk` 0x192310:70-74: obj+0x90 · RotMatrix(BL+0x30)),
  so the forward row's y = sin(pitch·pi/2) rides the position with the forward move — the
  walked dy band-settle-8 landed on its branch and master does NOT carry it (checked: no dy in
  master's walk branch).
- `Player_Collision` 0x194df0 builds the HITTEST at BLData+0x740 (the capsule bottom/top at
  +0x760/+0x770, radius 0.55 at +0x7cc, query type 0x800, cc = AnimObject+0xcc read fresh per
  tick at :86; cc is the ROOT HEIGHT 1.0347 [L port ROOT_HEIGHT; the crouch arm swaps to 0.55]).
- `Collide_Update` 0x1ea398:50 zeroes HITTEST+0x10 (= BLData+0x750, the frame push accumulator)
  per object per tick, then runs `Collide_Intersect` per contact node [M :93].
- `Collide_Intersect` 0x1ebaa8 copies the HITTEST (0xa0 bytes, :157), runs `Intersect_CylGeom`
  on the copy (:296); the push tail accumulates `(r - dist)·v^` into `_DeltaP` WITH the copy's
  segment shifted per tri [M 0x109da0:447-470]; on a hit the add-back transforms `_DeltaP` and
  adds it to the ORIGINAL BLData's +0x10/+0x20/+0x30/+0x50/+0x60 [M 0x1ebaa8:353-373].
- `Player_CollisionHandler` 0x1949a8:136-140 **adds BLData+0x750 to obj+0x30** — the walked
  wall response lands here; the last non-flag-2 hit's closest point vs BLData+0x760 sets the
  0x10/0x20/0x8 grounded bits [M :154-166].

**The port already models this pipeline's shape**: `world.rs capsule_push_scoped` shifts the
segment per tri contact and accumulates the push (the same DeltaP shape), and player.rs's
collision pass adds the push to pos (the 0x1949a8 mirror). So the horizontal slide machinery is
present; the two tests' missing behaviour is the walk's **vertical**:

- the feet band [M 0x194188:106] — the floor within cc+0.1 of the ray grounds; with cc the
  root height the band accepts floors up to ~0.1 ABOVE the feet, which is exactly the glide's
  +0.038 step — the port grounds on it but does not ride it;
- the walk-anim root ride (cc fresh per tick [M 0x194df0:86]) and the walked dy
  [M 0x193600:284-295] — neither is in master.

**A floor-sweep probe** (`probe_glide_floor_and_push`, `#[ignore]`, in the 0a test file) ran:
along the pre-slide line z 16.29 the first surface reads 1.875 (an overhead surface — the
walked line slides north to z ~17.02 by the settle), so the doorway floor's own height needs
the post-slide line; the probe stays in the tree for the next round.

## What was NOT done

No ignore was lifted: both tests stay red-on-master and the mechanisms that would move them
(the band ride, the root ride, the walked dy, the corner's walk-speed crossing) need the live
captures below before any [M]-honest landing. No shared helper changed; no `crates/*/src/`
change landed this window — the only code change is the new probe test and the two grown
ledger messages.

## Live checks for the main agent

1. **The glide's look trace** (settle-7's check 1, still open): fresh boot 0700000a, walk the
   doorway glide (lobby -> west hall). Per frame dump **obj+0x50 (the view pitch), obj+0x30.y
   and AnimObject+0xcc**. Decides how the +0.038 splits between the look-dy and the root ride;
   the walker walks level, so if the game's probe walk had pitch ~ 0 the whole climb is the
   band/root and the dy is not the glide's writer.
2. **The hall floor's live y**: at the settled stand (-9.9, 17.0) read the floor height the
   game's FeetOnPoint ray reports (the ray from obj+0x30 down 0.6+cc). Disc floor 1.09 or
   1.1285? If the disc floor is 1.1285 there, the port's floor query on the post-slide line is
   the defect; if it reads 1.09, the game's settle rides the band/root above the floor.
3. **The corner's band hook**: fresh boot, place Bond at (-17.87, 1.12, 4.87), hold forward
   toward (-17.5, 2.5). Per frame: obj+0x30.y, AnimObj+0x60 & 8 (the grounded bit), BLData+0x940
   and **BLData+0x750's y lane** (the push accumulator). The horizontal slide is pinned [L
   2026-10-26]; the y lane through the corner is the open question.
4. **The tail operand decode** (settle-6-c's check, still open): break at Intersect_CylGeom
   0x10a568 on a walked wall contact; read whether the push operand is `dot(Q - HITTEST+0x50,
   n)` (accumulator-relative) or `dot(query - Q, n)` (substitute).

## How it was verified

- Every measured number above is quoted from a run on this tree: glide.out (1.0905 / 230
  ticks), corner.out (the 6000-tick STUCK dump), the probe's floor sweep [L].
- The decompile citations carry their addresses; the port-shape claims point at the named
  functions (world.rs `capsule_push_scoped`, player.rs's `pos += push`).
- No test was weakened; both ledgers grew their evidence and their live-check lists.

## What is left

1. The four live checks (they decide the glide's writer split and the corner's band hook).
2. The walked-dy landing (band-settle-8's mechanism, missing from master) with its walker
   leveling companion, then the every-walkreal re-verification.
3. The post-slide floor sweep probe and, if the disc floor reads 1.1285, the port's floor
   query fix on that line; the corner's walk-speed crossing.
4. The full suite with the fast runner on the final tree (this window ran the two targets +
   the probe only; nothing in `crates/*/src/` changed, so no other binary is obligated).

## Usage

not visible to me (no token, context-window or cost figures are exposed in this session).
