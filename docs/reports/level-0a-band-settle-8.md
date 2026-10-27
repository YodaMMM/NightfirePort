# level-0a-band-settle-8 — the walk's vertical: the walked dy landed, the tail re-land measured and held

Branch `s/level-0a-band-settle-8` (from `origin/master` @ dcacdf1e, the batch carrying
settle-6-c). Task: port the feet band as the y-settle authority, then re-land the [M]-faithful
tail deltas (D2 at least) without lifting Bond anywhere; the 15 dead-ladder settle keeps
y 40.2..40.7; the settle-6-c 21-test list stays green; un-ignore the 0a trio when green.

## What landed

**The walked dy** — the walk's y now rides the view pitch with the forward move:

- [M Player_Update 0x193600:284-295] the walk tick's move add is the FULL matrix product —
  `obj+0x30 += walk_matrix · local move`, no lane dropped.
- [M Player_SSWalk 0x192310:70-74] the walk matrix is `obj+0x90 · RotMatrix(BL+0x30)` — the
  rate vector's x lane is the view pitch (the same `RotMatrixX(view pitch)` the G-046 dir
  reads), so the forward row's y = sin(pitch·π/2) rides into the position with the forward
  move. The right row stays horizontal (RotX leaves (1,0,0); RotY keeps its y at 0) and the
  walk's move has no vertical lane, so this term is the walk's whole vertical.
- Ported in `crates/nf-game/src/player.rs`'s walk branch: `pos[1] += self.look()[1] * vf` — the
  same `look()` term the swim branch already rides (the swim's dive/surface convention).
  [S] unverified, noted in the code: the game's matrix accumulates the per-tick rate while the
  port reads the clamped view pitch; the clamp's divergence is not modelled.
- The harness companion (`crates/nf-view/tests/common/walker.rs`): the walker walks LEVEL —
  the walk legs ease the look pitch back to 0. The door-use and aim legs pitch the view and
  used to leave it; with the dy live that leftover pitch drifted the walk's y by up to
  ~0.009/tick and broke the tuned routes (measured: `the_pickups_take_on_the_walk` wedged at
  the exit hall hovering y 41.47 over floor 40.28 until the leveling landed).

**Verification** (all through the real input paths):

- `interactables_07000015_live` **12/12** — the headline `the_dead_ladder_never_grabs_at_point_blank`
  settles y 40.5 (in 40.2..40.7, the walkway ≈ 40.35 [D floor tris]), every sister green.
- `walkreal_07000001_live` **3/3** — the escort walk incl. the contact (the D2-sensitive
  resupply pocket at (5.48, 6.67, 11.60) holds).
- `walkreal_07000009_live` **3/3** — the tower walk incl. the stairwell approach.
- `nf-game --lib` 578/0 earlier in the round; the full fast suite total is at the bottom.

## What was measured and NOT landed

**D2 (the Dist2Tri flags-99 first-query skip [M 0x109770 + 0x10a33c `beq v1,99`]) stays
env-gated OFF.** The round re-measured its landing:

- On the substitute tail, D2 re-enters the settle-6-b lift exactly as 6-c predicted: a probe
  teleports Bond to (85.08, 40.4, −2.4) on the 07000015 walkway; with D2 landed the settle
  ratchets to **41.315** (the spine bottom pushed to r above the skirt faces) — the sister
  canary (`capsule_hits_default(pre_collide)`) flips false because the settle drifts 0.76 off
  the ladder plane.
- The enabler the task's design points at — the **band-grounded anti-lift** (while the feet
  band [M Player_FeetOnPoint 0x194188:106] grounds the walk, an up-facing face's lift
  component is dropped from the tail's push; the 0.7 threshold is the game's own pick-bit dot1
  [M 0x10a650-0x10a674]) — was implemented and measured: with it, D2 lands and
  interactables_07000015 goes **12/12** (the settle parks ON the floor, y 40.35, the canary
  re-pinned to the point-blank stand the settle keeps) and 01 stays **3/3** — but
  `walkreal_07000009_live::walk_tower_a_start_to_finish` **wedged at the elevator-panel
  approach** (Bond stuck 2.4 short of the panel). The anti-lift kills the step-top lifts the
  tower steps ride. It is reverted this round (the code shape is in the WORKLOG); landing it
  needs the stair/slope vertical live evidence below.

**The 0a trio stays ignored.** The west-hall glide's pin (the settled 1.1285 [L 2026-10-26])
needs the glide's per-frame look trace (settle-7's live check 1, unanswered): the walker walks
level, so the dy is 0 on the glide and the port rest stays 1.1201 — the +0.0084 the game shows
came from ITS walk's look. The corner test needs the walk-speed crossing (NF_WALK_KILL, 870
ticks vs the game's ~60 frames — unrelated to this round's mechanism); the ring run needs its
own live ref. No ignore was lifted; none was weakened.

## Live checks for the main agent

1. **The stair climb's writer** (decides the anti-lift's fate): fresh boot 07000009, walk Bond
   up the tower stairwell steps. Per frame dump **obj+0x30.y, the view pitch (obj+0x50 /
   BL+0x8a8), and the forward speed**. Expected if the walked dy is the stair mechanism:
   Δy/frame ≈ sin(pitch·π/2)·vf with the pitch UP the stairs (the player looking up as he
   climbs). If Bond climbs with the pitch pinned 0, a second writer exists (the step-top
   tail lift) and the anti-lift must carve it out per-face.
2. **The tail operand decode** (settle-6-c's check, still open): break on Intersect_CylGeom
   0x10a568 at a walked wall contact and read the operand registers — `dot(Q − HITTEST+0x50,
   n)` (the accumulator-relative arm, the tail dead) or `dot(query − Q, n)` (the substitute)?
   This settles whether the game's walk tail can lift at all.
3. **The 0a glide's look trace** (settle-7's check 1, still open): fresh boot, probe-place at
   (−8.0, 1.12, 16.29), walk west; per frame dump obj+0x50 and obj+0x30.y. Fills the west-hall
   pin's look sequence so the test can reproduce the settled 1.1285.

## What's left

- The anti-lift + D2 landing, gated on live check 1 (the stair writer) — the 09 tower wedge is
  the one measured blocker.
- The 0a trio's un-ignore, gated on live check 3 (the glide's look trace) and the corner's
  walk-speed crossing.
- The tail operand decode (live check 2), which decides whether the substitute tail should
  shrink to the accumulator arm wholesale.

## Usage

not visible to me — this session's driver surface shows no token, cost or context-window
figures (the IM gateway metadata carries only session/chat/trace ids).

## Suite total (the fast runner, this branch)

(filled from `tail -1 suite.out` before the finish steps)
