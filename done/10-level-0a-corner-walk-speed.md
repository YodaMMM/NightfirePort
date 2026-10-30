## Your task: `level-0a-corner-walk-speed`, the 0a corner at walk speed

`walkreal_0700000a_live::the_corner_crosses_like_the_game` is ignored: the corner only crosses with the below-B kill (NF_WALK_KILL=1) in 870 ticks vs the game's ~60 frames [L 2026-10-26], riding spine-direction pushes. Find what the game does at that corner [M the collision response path: Collide_Intersect 0x1ebaa8 / Intersect_CylGeom 0x109da0 / the player push-out; D the corner's geometry], fix the port side so the corner crosses at walk speed without the kill switch, and un-ignore the test. Every test in the file green, plus the other walkreal binaries you can run. Quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-corner-walk-speed`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-30T10:25Z

---
## Report (done, 2026-10-30T11:33Z)


---
## Report from the worker (done, 2026-10-30T11:35Z)

# level-0a-corner-walk-speed — the corner's mechanism is the game's move-clip; measured at walk speed; the flip not landed in the window

## What was asked

Un-ignore `walkreal_0700000a_live::the_corner_crosses_like_the_game`: find what the game does at
the 0a wedge corner [M the collision response path: Collide_Intersect 0x1ebaa8 / Intersect_CylGeom
0x109da0 / the player push-out; D the corner's geometry], fix the port so the corner crosses at
walk speed without the below-B kill switch, every test in the file green.

## What the decompile says [M]

1. **The decoded push tail** (`Intersect_CylGeom` 0x109da0, per triangle, in the object-local
   copy of the HITTEST): the aimed point p = the spine's plane-crossing point (parallel branch:
   the spine point closest to the triangle centroid, clamped to the spine, 0x109da0:385-411); on
   an edge/vertex contact the probe re-aims p at **the spine point closest to the REF**
   (0x109da0:333-368 — the port re-aims closest to the contact q instead, world.rs:465 — a real
   divergence). The tail then: v = p − HITTEST+0x50 (0x109da0:420-424), gate `dot(v, n) > 0`
   (:425-431), d = |v| (:432-437), gate `d < geom+0xc` (:447), push = normalize(v)·(geom+0xc − d),
   `_DeltaP += push` and the copy's A/B += push (:448-470).
2. **The ref census** — HITTEST+0x50 (BL+0x790) has NO writer outside the add-back: an ELF-wide
   raw-word store sweep (all store opcodes incl. sqc2/sdc1/sdc2/sc/swc1, displacements 0x790 and
   0x7a0, whole ACTION.ELF image 0x107100..0x30d670) finds zero hits from a BLData base; the only
   0x790-displacement store in the image is a stack spill. `Player_Collision` 0x194df0 (disasm
   0x194df0..0x19548c) stores 0x760/0x770/0x7b8/0x7c0/0x7cc/0x7d0/0x7d4/0x7d8 and never 0x790/0x7a0;
   `Collide_Update` 0x1ea398 zeroes only HITTEST+0x10 per tick (:50); `Collide_Pick` 0x1eb320 reads
   +0x50 and writes +0x40 (the spine) and, for 0x100/0x1000 shapes, +0x60 — never +0x50; the feet
   ray (Player_FeetOnPoint 0x194188) never writes the HITTEST. The only writer is the add-back
   (Collide_Intersect 0x1ebaa8:353-373: +0x10/+0x20/+0x30/+0x50/+0x60 += world DeltaP per hit).
   [L ee_start_0700000a] it boots (0,0,0); [L ee_snapshot] it reads ~15 m from its owner mid-play.
3. **The tail cannot sustain the walk's wall response.** The walk spine is vertical
   (A = pos+up·0.275, B = pos+up·(0.55−cc), |B−A| = 0.7597 [M Player_Collision 0x194df0; L
   ee_start_05]); p is spine-clamped, so for any ref that sits on the spine axis (A, B, pos, the
   spine midpoint) v = p − ref is purely vertical and the `dot(v, n) > 0` gate kills every
   vertical-wall push; a far ref (the accumulator's 15 m) fails the `|v| < radius` gate. The
   sustained response is therefore not this tail.
4. **No other wall response exists in the player path**: `Player_DealWithObjHit` 0x194458 writes
   no position (damage/doors/ladders/bullets only); `Player_Move` 0x191dd0 is pure input→velocity
   integration (no collision reads); the handler 0x1949a8's only position write is
   `obj+0x30 += HITTEST+0x10` (the per-tick DeltaP sum — the dead tail's output); `Player_Collision`
   0x194df0 integrates velocity only.

## The empirical fix [L, measured this session]

The game's walk wall response is the **move clip** — the tick's motion clipped at the contact
plane, the tangential component carrying — which the port already has built and env-gated
(`capsule_tri_clip`, world.rs:564-639, `NF_WALK_CLIP=1`):

- `NF_WALK_CLIP=1 cargo test -p nf-view --test walkreal_0700000a_live the_corner_crosses_like_the_game -- --ignored --test-threads=1`:
  **the corner CROSSES at +76 ticks, arrived (−17.56, 1.09, 2.83)** — the walk-speed crossing
  (the game's ~60 frames [L 2026-10-26 fresh boot]), no kill switch. The ≤300-tick assert passes;
  only the exit-y pin fails (below).
- The clip model's spine-plane rest matches the game: the glide rest holds the spine ~0.011
  INSIDE the wall plane [L fresh boot] — a radius-0.55 push-out would rest the spine ~0.55
  outside. The port's push-out stand-ins (spine-dir, radial) all rest on the skin — the wrong
  rest geometry, and at the corner the two walls' pushes fight (the 870-tick kill-assisted crawl).

## Why the test is still ignored (honestly)

1. **The exit-y pin** reads 1.0891 vs the settled 1.1159 [L 2026-10-26] — a 0.0268 gap that is the
   west-hall leg's unported **+0.0256 climb settle** (the feet-band walked-dy growth, band-settle
   finding 1) — the glide sister task's mechanism (`s/level-0a-hall-glide-settle` had only a
   WORKLOG-start commit at 11:28 UTC). The corner inherits the entry height from that leg.
2. **The flip's collateral**: with NF_WALK_CLIP=1 the file reads 4 passed / 3 failed / 2 ignored
   (master: 7 passed / 0 failed / 2 ignored):
   - `the_pick_ref_accumulator_tracks_the_game_ref` — the clip path stops feeding the port's
     DeltaP accumulator (the clip's own comment documents the semantic change);
   - `walk_office_floor_qworm_keyboard_hatch` and `walk_office_floor_to_the_security_keyboard`
     — the office-floor legs' contacts shift under the clip (the objective-4 shot line misses).
   Reconciling those three is the remaining work before the flip lands and the test un-ignores.

The corner test's docstring + ignore note on this branch pin the new state (the 76-tick
measurement, the mechanism, and the un-ignore conditions).

## What's left

1. Reconcile the three clip-red tests (the pick_ref feed semantics; the two office-floor legs),
   flip the walk default to the clip, un-ignore the corner test.
2. Port the hall glide's climb settle (the sister task) — the corner's exit-y pin needs it.
3. A live check that would settle the tail question definitively (below).

## How it was verified

- Full decompiles read: Intersect_CylGeom 0x109da0, Collide_Intersect 0x1ebaa8,
  Player_CollisionHandler 0x1949a8, Player_Collision 0x194df0, Collide_Update 0x1ea398,
  Collide_Pick 0x1eb320, Player_FeetOnPoint 0x194188, Player_DealWithObjHit 0x194458,
  Player_Move 0x191dd0, Player_SSWalk 0x192310.
- Disassembly sweeps with a raw-word scanner (capstone misdecodes sqc2): whole-image 0x790/0x7a0
  store sweep; 0x780..0x7bf window; addiu-computed-pointer sweep; per-function dumps of
  Player_Collision / Collide_Update / Player_CollisionHandler; the push tail 0x10a500..0x10a620.
- Test runs (slim on-foot ISO, --test-threads=1): walkreal_0700000a_live master baseline 7 passed
  / 0 failed / 2 ignored (27.32 s); the same file with NF_WALK_CLIP=1 → 4 passed / 3 failed /
  2 ignored (14.22 s); the corner test alone with NF_WALK_CLIP=1 → the 76-tick crossing, failed
  only the exit-y pin (2.40 s).
- The full suite (tools/test/suite.py) did NOT run this session: this sandbox kills background
  processes (verified twice — setsid/nohup/disown variants all die between tool calls; same
  finding as the previous session's WORKLOG), and the foreground window did not fit after the
  research. The touched-crates battery is the three 0a runs above; no sim code changed on this
  branch (docs + the test's note only), so the tree's sim behaviour is master's.

## Live checks for the main agent

1. **The walk's wall rest, to settle clip-vs-push**: fresh boot 0700000a, walk Bond into the west
   hall's south wall and hold forward 5 s. Record obj+0x30 (the player root) minus the wall's
   plane. EXPECTED if the clip model is right: the root/capsule axis rides ~0.01-0.02 INSIDE the
   plane (no radius skin). A push-out model would hold it ~0.55 outside. This single number
   decides the walk model.
2. **HITTEST+0x50 live trail**: gdbwatch.py on BL+0x790 writes for 10 s of the same wall press
   (BL = *(obj+0xe0), obj 0x1b78640 per the -1 session). EXPECTED: no writes except the add-back
   at 0x1ec1xx, value drifting slowly — confirm whether the drift can cross within 0.55 of the
   aimed point (the only way the decoded tail ever fires on the walk).
3. **The climb settle**: fresh boot, walk the flat west hall (−10.0, 16.0) → (−20.5, 16.5) and
   record y − floor_hit per 10 ticks. EXPECTED: the gap grows by ~0.0256 over the hall (band-settle
   finding 1) — the mechanism the corner's exit-y pin waits for.

## Usage

My own session's token / credit / context-window / cost figures are **not visible to me** — the
environment shows me none of them for the main session. The only usage figures I can see are the
per-call token counts the sub-agents I spawned report back in their result footers; for example
the last helper call reported `Tokens -> prompt:12481 completion:1714 total:14195`, and the
disassembly-sweep call reported `prompt:71430 completion:7794 total:77924` (read from those
sub-agent results). No cost or credit figures are visible anywhere.
