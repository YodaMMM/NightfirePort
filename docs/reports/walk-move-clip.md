# walk-move-clip — the game's move-clip walk model behind `NF_WALK_CLIP=1` (default OFF), measured against the walkreal battery

## What was asked

The level-12-every-objective report's two 07000012 stand-ins (the door[19] doorway pincer, the
north-wall funnel pinch) exist because the port resolves Bond's capsule with a two-sided
depenetration push (`capsule_push` / `capsule_tri_contact`) whose sums pin him where the game
walks through: the game resolves penetration along the move [M `Intersect_CylGeom` 0x109da0] and
its walk push tail is dead [M `Collide_Intersect` 0x1ebaa8:353-373; [L] the +0x50 accumulator —
ee_start_* boots it (0,0,0), ee_snapshot reads it ~15 m from its owner]. Port the game's
move-clip behind `NF_WALK_CLIP=1` (default OFF), run the WHOLE walkreal battery on and off,
replace the stand-ins with real input under the flag, full suite total line.

## What landed (commit bd9580cc, branch s/walk-move-clip)

- `crates/nf-game/src/world.rs`: `capsule_tri_clip` — the per-triangle contact with the game's
  skip set (AABB, water 0xE, the port's one-sided floor rule), the game's OWN head-on pairing
  [M 0x109da0:321-322 — skip when (n·d̂ > 0.999 ∧ f > 0.5) ∨ (n·d̂ < −0.999 ∧ f < 0.5); note
  `capsule_tri_contact`'s HEAD_ON_CULL_ON copy has the arms flipped and stays off], the contact
  gate `dist < r` from the plane-crossing query point (the game's f-clamp + Dist2Tri shape
  [M 0x109da0]), the walkable/pick signals unchanged — and the push tail left DEAD. In its place
  the tick's move is CLIPPED: a face-interior contact whose face the move pushes into retracts
  the move to the face plane (t* = −d0/dot(n, move), each clip applied before the next triangle —
  the game's shifted-segment flow [M 0x109da0's A/B += push]). Edge and side-overlap contacts
  never displace (the game walks through the door[19] slab poke and the 0.88 funnel pinch — the
  dead tail). `capsule_clip_scoped`/`capsule_clip` mirror the push pair's cel-scoped plumbing.
- `crates/nf-game/src/player.rs`: `walk_clip()` (the `NF_WALK_CLIP` env switch, default OFF); the
  walk path captures the tick's move before integration and, under the flag, swaps the push for
  the clip query (both the scoped pass and the floor-missing redo) and freezes `pick_ref`
  (the game's +0x50 accumulator only ever receives DeltaP, and the dead tail produces none).
  Default OFF — the main agent decides after a feel check.

The [S] port choices (marked so in the code): the spine-plane rest with no radius skin (the
game's own glide rest holds the spine ~0.011 inside the wall plane [L fresh boot, the 16.29
line]), the face-interior-only clip, and the front-side-only retract.

## How it was verified — the walkreal battery, on and off

`scripts/walkreal_battery.sh` (in the sandbox, not committed — it is scratch) runs each nf-view
`walkreal_*` live binary alone with `--test-threads=1`, `NF_SILENT=1`, `NF_ISO` set — the same
env `tools/test/suite.py` uses. This window's binaries are built from the pushed tree
(bd9580cc); the first window's ON numbers mixed stale links and were discarded after the sandbox
reset.

**Flag OFF (the master model): 61 passed / 0 failed / 1 ignored** across the 17-binary set:
01, 02, 04, 08, 09, 0d, 11, 11_route, 12, 13, 14, 15, 16, 1b, 41, 4a, 4a_satchel — every
measured binary fully green.

**Flag ON (the move-clip model): 28 passed / 21 failed / 1 ignored** on the same set. Per binary:
green — 02 (1/1), 11 (1/1), 11_route (1/1), 41 (5/5), 4a_satchel (2/2); failing — 01 (2/4: the
escort-follow descent at :730, the spawn-to-exit contact at :121), 04 (2/5: the full-mission walk
at :129, the kill outro, the results page), 08 (1/3: the fusebox laser at :162, the noise-path
floor plate at :554), 09 (2/3: walk_tower_a at :134), 0d (0/1: the c5-door approach at :150),
12 (0/1: the beam-burn at tower2.rs:47), 13 (0/1: the same tower2.rs:47 leg), 14 (1/5), 15 (0/1:
the omega exit at :139), 16 (3/6), 1b (7/8: the boundary-wall float stop at :388), 4a (0/1: the
lift-into-lobby at :190). The slow tail (03 ≈385 s, 05, 06, 07, 0b, 0c and the probe binaries)
was not re-run in this window — OFF they are master-green (the merged suites' history); their ON
numbers are the next session's first item.

The failure classes, with the mechanism:
1. **Precision aim/fire legs break** (12/13's beam-burn, 08's fusebox + noise plate, 04's
   results tallies): the clip's landing rest puts the spine bottom ON the floor plane
   (pos = floor + 0.35) where the push model's radial equilibrium sat higher — Bond's eye and
   the aim geometry shift enough to miss the pinned beam/fusebox lines.
2. **Head-on wall stops are lost for AIRBORNE drift** (1b's boundary-wall float): the float's
   side-overlap contact never crosses the plane, so nothing clips and Bond drifts through where
   the push used to hold him.
3. **The walk routes that stay green** (the castle 02, the stairwells 11/11_route, 41, the
   4a satchel run) are exactly the class the pincer/funnel stand-ins model — the clip walks
   THROUGH the side-pinches as the game does, and the stair descents ride the game's own
   head-on pairing.

## The stand-ins

None could be replaced this round: 07000012 fails BEFORE its two stand-ins (the Q-Decryptor
beam-burn at `common/tower2.rs:47` never fires under the flag), so the real-input crossing could
not be validated end to end. The stand-ins stay; the failure classes above are what the next
iteration must close (a feet-band rest instead of the spine-plane landing for class 1, an
airborne skin for class 2).

## The full suite

The sandbox reset at ~14:30 UTC cost the toolchain and the target dir; the second window rebuilt
nf-game (30 s) and the nf-view test binaries, ran the battery, and ran out of window before a
chunked full-suite pass could record a total line. The code compiles clean under
`cargo build --offline --release -p nf-game`; the battery binaries are the compiled proof for
nf-view's test targets. **The suite total line: not recorded this window** — the next session
should run `tools/test/chunk_suite.py` (state-file resume) on this tree first thing; with the
default OFF the tree is behaviour-identical to master outside the new dead code paths.

## What's left

1. The ON battery's slow tail (03/05/06/07/0b/0c + probes) for the complete both-ways tally.
2. The failure classes 1 and 2 above — the model needs the game's own feet-band rest (the walked
   dy growth [M Player_Collision 0x194df0:104-139's vy damping] is the separately-named gap) and
   an airborne contact rule before any stand-in replacement can pass end to end.
3. The full suite total line (chunk_suite on this tree).
4. The default stays OFF either way — the main agent decides after a feel check.

## Live checks for the main agent

1. `Intersect_CylGeom` 0x109da0, the push tail: watch HITTEST+0x50 (BL+0x790) during a plain
   corridor walk on a fresh boot — gdbwatch on 0x16cf0f0+0x790 (the 05 BL), reads+writes, ~10 s
   of walking at a wall. Expected per the model: no write ever passes the |q − acc| < r guard
   (the accumulator stays (0,0,0) until a lift/cab ride); if a walk contact DOES write it, the
   dead-tail reading is wrong and the clip model's premise changes.
2. The head-on wall rest: walk Bond straight into a flat wall on 07000001 (fresh boot) and
   freeze; fieldprobe obj+0x30 and the HITTEST spine B (BL+0x770). Expected: the spine B rests
   ~ON the wall plane (within ~0.05, no radius skin) — the clip model's rest — vs the push
   model's r-away rest. This single measurement picks between the two models.
3. The walked floor rest: on flat ground, fieldprobe obj+0x30.y and the floor plane y under him
   (the 0a office spawn line is pinned [L root 1.0891]) — expected per the game: the rest rides
   the walked-dy history, not a radial equilibrium; measure the eye height the beam-burn legs
   need (the 12 keypad lock at (7.0, 1.6, 0.2) burns from the office floor).

## Usage

not visible to me (this environment exposes no token, cost or context-window figures to the agent).
