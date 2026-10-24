# level-0a-office-hatch-10 — the HITTEST+0x50 settlement: the overlap gate's reference is a dead accumulator, and the wall-stop is not the cylinder tail

## What was asked

Settle what HITTEST+0x50 (BL+0x790) holds [M — its boot write, every writer, what the DeltaP adds
accumulate], port the overlap gate with that exact reference, and pin it: the glide holds
1.0891 → peak 1.18 → settle 1.1147 and the corner exits 1.127 → 1.120 (±0.01) [L]; keep the
walkreal battery green; un-ignore `the_west_hall_glide_height_matches_the_game`,
`the_corner_crosses_like_the_game`, `walk_office_floor_qworm_keyboard_hatch` as they go green and
drop the env switch once the gate is default.

## The settlement (done, and it changes the picture)

**HITTEST+0x50 is a lifetime push accumulator: boot (0,0,0), only ever `+= world DeltaP` per
landed hit, and in live play it drifts metres away from its owner.** Evidence, point by point:

- **Boot write: none — the field is born zero.** `Player_Init` 0x1910f8:40 sets `obj+0xd4 =
  BL+0x740` and calls `control_init_collision` — which is an empty stub (0x134878, size 8: a bare
  return) [M]. The player object comes from `control_create_object(0x970, …)` 0x1344e0:17, which
  `memset`s the whole 0x970+0x110 allocation to 0 [M] — the HITTEST block BL+0x740..0x850
  included. `Player_Collision` 0x194df0 re-seeds +0x760/+0x770 (A/B), +0x7b8, +0x7c0, +0x7cc,
  +0x7d0, +0x7d4, +0x7d8 each tick but never +0x790 [M 0x194df0:205/229/257/282/315/363/381].
- **Snapshot confirmation [L]:** `ee_start_0700000a.bin` at the 0a level start reads BL+0x790 =
  (0, 0, 0) EXACTLY (all 16 bytes zero) while +0x760/+0x770 hold the freshly seeded A =
  pos+up·0.275 and B = pos−up·0.4847, radius +0x8c = 0.55, kind +0x94 = 0x800, and obj+0x30 =
  (−3.9319, 1.0891, 10.3978) — the glide test's own spawn line. `ee_start_05.bin` reads the same
  zero at BL 0x16cf0f0+0x790 mid-settle. So no boot path and no pre-play tick writes it.
- **Every writer: the DeltaP adds, and only them.** `Collide_Intersect` 0x1ebaa8:353-373 adds the
  world-space DeltaP to +0x10 (the per-tick push, `Collide_Update` 0x1ea398:50 zeroes it each
  tick), +0x20, +0x30, **+0x50** and +0x60 after each landed hit [M]. I verified negatively at the
  instruction level: an exhaustive scan of ACTION.ELF for every immediate-offset memory or
  address instruction (sw 0x2B / lw 0x23 / swc1 0x39 / lwc1 0x31 / sh / lhu / sqc2 0x3E-0x3F /
  lqc2 0x37 / addiu-oru) with immediate 0x790, 0x788 or 0x798 finds FOUR words, all in
  `Menu_SpecialCodenameCheck` (stack spills at 0x2012c4-8) — no code anywhere addresses BL+0x790
  directly. The sqc2 encoding was calibrated on the real stores (`sqc2 vf2, 0x760(s2)` at
  0x1950ec writes the walk A). So the decompile's add block is the only writer.
- **What the adds accumulate [L, the decisive measurement]:** `ee_snapshot.bin` (a played state:
  Bond at (104.71, 24.97, 107.05), BL 0x16cf0f0) reads BL+0x790 = (1.2079, 9.0838, −14.4486) —
  about 15 m from its owner, while +0x7a0 (+0x60, accumulated by the same adds) = (29.8, 37.7,
  50.3). That is what Σ DeltaP looks like after the 05 lift ride: the cab's pushes land every
  tick of the descent and the accumulator keeps the sum forever. It is NOT the spine bottom, NOT
  the position, NOT per-tick state of any kind.

**Consequence for the gate.** The decoded tail [M `Intersect_CylGeom` 0x109da0:419-457] is:
`q` = the closest triangle point (Dist2Tri 0x109770 writes it out), the gate
`0 < dot(q − HITTEST+0x50, n)` (:422-431, hatch-9's address), then the magnitude guard
`|q − HITTEST+0x50| < radius` (:447) and the push `(q − HITTEST+0x50)/|…| · (radius − |…|)`
(:432-457) — a radius-shell response measured from the +0x50 point, moving A/B and DeltaP
together. With the reference being a drifting Σ-pushes point, the player's walk-vs-static-world
tail fires only against triangles within 0.55 of that point — near-never on a walked line. Two
independent readings agree: the [L] fresh-boot glide (the game holds the z 16.29 line with the
capsule 0.011 inside the z=16.279 plane, no shove) needs NO gate — nothing pushes there at all —
and the [L] drift kills the ref=bottom shape hatch-9 probed (`NF_CAPS_GATE=1`): it ports a value
the field never holds. The tail also gates the best-hit tracking (:518 → :524 — for the player
the hit list only records triangles whose push LANDED, since the scanner's `obj+0xf0 & 0x40`
car-bit is what sets `HITDATA+0x51` bit 1 [M `Collide_Intersect` 0x1ebaa8:93-99], and the player
is not a car), so if the tail is dead the game's HITDATA list stays empty on plain walls — and
`Player_DealWithObjHit` 0x194458 handles only damage/ladder/wire/bullets [M], `Player_FeetOnPoint`
0x194188 only grounds [M]. **The game's real wall-stop for the walk is therefore some mechanism
outside the cylinder tail, still unidentified** — that is the round's open point, and it is the
block on everything else.

## Why nothing landed (the gate stays OFF, the three tests stay `#[ignore]`)

- The exact-reference port (accumulator ref) removes the port's only wall-stop on the walked
  line (the port's `capsule_tri_contact` push is the sole stopper today): every walkreal walk
  ghosts walls. Not landable.
- The ref=bottom probe (hatch-9's `NF_CAPS_GATE=1`, kept off) is measured, not guessed: the
  corner crosses (~27 ticks) but the exit-y misses both ways (1.0655 floors-gated / 1.2142
  floors-exempt vs the game's 1.120 ± 0.01 [L]) and the glide walk floats ungrounded at
  (−8.05, 1.13, 15.03) — the port's ground model consumes the very floor pushes the game's tail
  never produces (the game's vertical rest is the feet band's job [M 0x194188:106]).
- No decompile-backed test was weakened; no [S] entered pass/fail. The env switch stays OFF and
  its comment now records the settled identity (the hatch-9 rationale — "ref = the spine's
  bottom … the game's player walks the 16.29 line" — is corrected in place).

## The NF_WALK_GHOST experiment (new, on the branch, default OFF)

To test the dead-tail reading end-to-end I added `NF_WALK_GHOST=1`
(`crates/nf-game/src/player.rs`, the walk integration block): it drops the walk capsule's
DISPLACEMENT while keeping its contact signals (walkable still feeds `grounded`). Measured:

- **The 0a wedges vanish.** `the_west_hall_glide_height_matches_the_game` walks through the
  doorway line and reaches the hall at (−9.270, 16.259) — the game's own line (the port's
  27,404-contact jamb-pocket wedge is gone); `walk_office_floor_qworm_keyboard_hatch` advances
  past its old 'ai lane to the N-S corridor' wedge to the DOCUMENTED Q-Worm-shot blocker at
  (−16.38, 0.97, −30.12). The wedges were the port's own pushes, exactly as the dead tail
  predicts.
- **But the rest sits low and other levels lose their walls.** The glide's height reads 1.0320
  vs the game's 1.1147 [L] (−0.0827: the port's rest rode on the floor push; the band-only rest
  is lower — the game's history-dependent climb settle [M `Player_Collision` 0x194df0:104-139's
  vy damping] is the separately-named gap), and the real walks of 01 (exit walk), 02 (castle)
  and 03 (alpine) FAIL with the push gone — the port's wall-stop is load-bearing on those
  routes, so the game must stop the walk somewhere the port has not found. Default mode is
  untouched: `walkreal_0700000a_live` 4 passed / 3 ignored on the final tree.

So the round's verdict, all [M]/[L]-anchored: the overlap gate as imagined (a per-tick gate on
an alive push) does not exist in the game — the reference is a dead accumulator — and the
port's divergence is that ITS push is alive where the game's is not; but the game's real
wall-stop (which keeps 01/02/03 working) is still unidentified, and until it is found no push
removal can land. The experiment switch stays on the branch (default OFF) as the repro
harness for the next round.

## What I ran

- `cargo test --offline --release -p nf-view --test walkreal_0700000a_live -- --test-threads=1`
  on the final tree (branch = hatch-9 + origin/master merge + this round's comments):
  **4 passed, 0 failed, 3 ignored** (the task's three — unchanged, honestly).
- The fast suite `tools/test/suite.py` from the merged tree (started 20:41 UTC in the
  background): the total line is pasted below when it landed.

## What's left (the next round's shape)

1. **Find the game's wall-stop.** The ghost experiment pins the requirement: a stopper that
   keeps the 01/02/03 walks working but never fires on the 0a doorway/corner lines. Candidates
   ranked: the velocity-side consumption of the obj+0xdc flag word (`Player_CollisionHandler`
   0x1949a8:154-165 sets bits 0x10/0x20 from the first hit's hitpoint vs BL+0x760, bit 0x8
   zeroes BL+0x50 when grounded) — but the flags come from the hit list, which needs the tail
   alive; the `Collide_Jointy` 0x1ec180 joint-sphere path (the level-14 actor push already
   ported) — the 05 lift cab rides suggest object rides DO land DeltaP; or the tail's guard is
   fed a different radius than the decompile's `param_1[8]+0xc` reading (the local-copy's +0x80
   word needs one live read — see check 3). The live checks below decide it in one session.
2. Re-derive the walk's vertical settle (the game's history-dependent rest: the vy damping
   [M 0x194df0:104-139] + the band catch [M 0x194188:106]) so the rest does not depend on
   capsule floor pushes, then re-try the two height pins under the ghost switch.
3. Un-ignore the three tests when their configuration is defended by [M]/[L], and only then
   drop the env switches.

## Live checks for the main agent

Fresh boot of 0700000a (a save-state resume skips the boot path):

1. **Is the tail really dead on a wall press?** gdbwatch reads of BL+0x790 (the 05 BL is
   0x16cf0f0; on 0a read obj+0xE0 first — the obj is `glb_players[0]` @ 0x2d88e0 → +0xE0) during
   a steady press into the doorway jamb (walk x −8.0 → −8.4 at z 16.29): expected = no change
   at all (no push lands → no adds). Then read obj+0xd0 (the HITDATA list head): if it stays 0
   while the game visibly stops Bond, the stop is hit-list-independent — the stopper is in the
   control/integration side.
2. **Where does the stop land?** With Bond pressed at the jamb (axis x ≈ −8.05), single-step
   gdbwatch the WRITE to obj+0x30 for 10 ticks and record the exporting address of each write:
   the port predicts `Player_CollisionHandler` 0x1949a8:136-140 (the BL+0x750 landing) if the
   tail is alive after all, or a `Player_Update`/control-func address if the velocity side
   clips. One 10-tick trace names the mechanism.
3. **The H50 drift on 0a** (cheapest, do first): after the SS-door sweep, read BL+0x790 once.
   [L ee_start_0700000a] pins (0,0,0) at level start; the 05 snapshot's (1.21, 9.08, −14.45) is
   the lift-ride prediction for 0a's own entry lift — if the 0a post-sweep value is ≈ the lift
   descent vector, the accumulator identity is confirmed on this level too and the ref=bottom
   gate can be deleted outright.
4. **The tail's radius word** (settles whether the tail is truly dead): while Bond stands in
   the 0a office, read the scan's local copy base — `Collide_Intersect` builds it on the stack;
   the field to identify is the one `Intersect_CylGeom` 0x109da0:447 compares `|q − H50|`
   against (the decompile's `param_1[8]+0xc`). A gdbwatch read at 0x109da0+the compare's
   load offset during one walk tick records the value: 0.55 → the tail is alive whenever the
   contact is near H50 (the drift alone explains everything); a pointer-as-float (≈2.4e-38,
   cel+0xc anchor) → the tail never fires anywhere and the port's push should be deleted once
   the real stopper lands.

## Usage

not visible to me (this environment exposes no token, credit or context-window figures).
