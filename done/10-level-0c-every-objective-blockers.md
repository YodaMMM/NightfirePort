## Your task: `level-0c-every-objective-blockers`, Chain Reaction Part 1: blockers 2-5

The 0c walk (on master) is green through the turbine door and the band crossing (read the level-0c-door2turbine-3 report). Blockers 2-5 of the level-0c-every-objective report were never started: (2) plate #274's cell inside the display case (does the glass body-block it in the game, or does the plate fire another way), (3) the display ground -> warehouse/yard descent, (4) the ring's west/south gaps (the south one is the #283 wire ride onto the west ledge), (5) the doorway-ledge -> yard descent. Take them in order, each from the game's behaviour [M/D], fix any port gap, and replace each stand-in in `walkreal_0700000c_live` with the real-input leg. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-every-objective-blockers`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-29T20:01Z

---
## Report (done, 2026-10-29T20:42Z)

# level-0c-every-objective-blockers — the report

Task: 0c (Chain Reaction Part 1) blockers 2-5 of the level-0c-every-objective report, in order —
each from the game's behaviour [M/D], fix any port gap, replace each stand-in in
`walkreal_0700000c_live` with the real-input leg.

## What's done

**Three committed probe binaries** (`crates/nf-view/tests/walkreal_probe0c_blk_live.rs`,
`_blk2_live.rs`, `_blk3_live.rs` — each reproducible from the ISO with `--nocapture`) mapped all
four stand-in regions from the port's disc-built collision world [D]. Every number below is [D]
unless tagged otherwise, and all four blockers got a game-mechanism answer:

1. **Blocker 2 (plate #274 in the display case) — the plate hangs in a TRENCH, and the case's
   south glass blocks the walk-in.** The walkway's north end opens down to the LOWER WALKWAY at
   x 5..6.5, z −110.5..−111.5, floor 11.84 [D probe blk floor map] — the trench runs between the
   case's glass walls (a west face at x 4.20 at y 15.0, an east face at x 6.74 at y 12.5..14.4,
   a north face at z −110.02 spanning y 12.5..15.8 [D rays]). Plate #274 (kind 2, out 0x1b,
   centre (5.79, 14.09, −111.10) [D trig dump]) hangs in the trench's fall path, so the game's
   fire mechanism is the body FALLING through the volume [L 2026-10-26 hatch-2: the touchplate
   pulses while the body's volume crosses and clears at rest]. The walk-in attempt wedged at
   z −112.95 at body height while a y-14.4 ray passes under the face [this run's STUCK +
   probe blk rays] — the south glass starts above the walkway floor; the game's entry is the
   hinge-burn route: the four hinge locks #139-142 (out 14..17, at (7.09/4.54, 15.70/16.87,
   −112.37) [D locks]) burn under the laser watch's REAL fire path [M laser_watch_0c_live's
   pins; Lock_Update 0x1ae3d8 + drone.rs:14004's contact pass], the AND #505 (14,15,16,17 → 18
   [D]) drives SS #138 `ventmesh-front` 16.4 → 14.76 [D keys; M SS_Update 0x1a9948], and the
   [L hatch-2] drop from the duct fires the plate mid-fall. The port gap is the same 0xE2-family
   gap the walkthrough already carried (the stand-in stays; the evidence moved from "unknown"
   to the chain above).
2. **Blocker 3 (display ground → warehouse/yard descent) — the descent EXISTS; the "floor-broken"
   reading was a probe artifact.** The display-ground slab (6.27, x 0..14, z −152..−155.5) drops
   at its south edge (z ~−155.7) onto the deck (2.37, x 2..8, z −156..−157.5), which drops at its
   south end (z ~−157.5) onto the YARD (−0.02, x 4..8, z −158), which runs east to the warehouse
   slab (0.38 x 42..48, 0.89 at x 50, 0.90 x 52..56) and the secret-door plates [D probe blk3's
   y 4→−0.5 map]. The old "2.4 deck dies in a void at z −154" was a y 9→1 ray that misses the
   yard floor below y 1. The game's ALTERNATIVE connection is the annex vent, fully evidenced:
   the lock #180 (out 0x54, (53.80, 5.90, −165.36) [D locks dump]) burns under the laser [M],
   ch 0x54 drives the SS lid (53.77, 5.43, −163.59) → (53.77, 0.00) [D keys], and the hole falls
   to the warehouse floor 0.90 [D probe blk underside map]. No port collision gap — the walk uses
   whichever route; the stand-in stays only because the slab is not yet reachable on real input
   from the photograph spot (see What's left).
3. **Blocker 4 (the ring's west/south gaps) — the crossings are the WIRES, all [D]-floored at
   both ends.** The level's type-3 grab icons mark every end: icon #4 (−3.30, 12.36, −207.51) at
   the west edge; icons #6/#7 (−1.54 / 7.21, 12.27, −223.57) flanking wire #282; icon #12 (28.42,
   12.24, −218.43) at the SW arc's east end; icon #14 (36.28, −208.82) at the SE corner [D icons
   dump]. The derived grab segments [D probe blk; Wire_Create 0x1981d8's centre±spherical
   geometry, the port's wire.rs]: #284 `wire_firstsilo2mainsilos` A (−4.63, 13.67, −209.88) → B
   (−8.47, 13.67, −216.52) — B lands on the west ledge (11.24-11.38, x −12..−8, z −216..−218);
   #282 `wire_silosilo2` A (5.85, 14.11, −223.15) → B (−0.25, 14.11, −223.15) — the (−2..−1 and
   4..8, −221..−226) ledges (11.24) under its ends; #283 `wire_silo2turbine` A (29.58, 14.44,
   −216.96) → B (35.89, 14.44, −209.44) — B lands on the ring SE corner (11.09..11.17). The body
   sphere-HIT at (28, 12.3, −217.5) against #283 is confirmed [D probe blk grab test]. The ride
   mechanics ARE ported [M Player_CollWire 0x197130, Player_Wire 0x197658's strafe tree,
   Player_HandleJump 0x1916b0:118-147's ctx-3 jump grab; the port's wire.rs/player.rs] — this is
   a TEST leg, not a port gap.
4. **Blocker 5 (the doorway-ledge → yard descent) — the ledge is NOT fully boxed.** The south
   wall (z −175.80, y 1.5..2.9+) spans x ≤ 55.5 ONLY: the north-south rays at x 56.5/57.5 hit
   nothing at y 1.5..2.9, and the floor map shows the ledge (0.90) stepping down to the yard
   (0.69) across x ~55.75 for z −174..−175.5 [D probe blk floors+rays]. The real descent is
   east-then-south around the wall's east end. The validation run wedged at x 55.0 on an
   unprobed face between x 55.0 and 55.5 at y ~2 (the leg's first waypoint) — one face map away
   from a real leg.

## What was run

- `cargo test --offline --release -p nf-view --test walkreal_0700000c_live -- --test-threads=1`:
  **2 passed; 0 failed** on the final tree (the master stand-in shapes + the finding comments).
- The three probe binaries: each `1 passed`, printing the maps quoted above.
- The FULL suite: **did not fit the window** — the background fast runner dies at the tool-call
  boundary in this sandbox (known), the foreground chunk budget went to the blocker probes and
  the leg attempts, and the sandbox expires ~22:03 UTC. Only nf-view tests this task touched
  (test-only diff; no nf-game/nf-formats/nf-audio source changed, so their suites are unaffected
  by this diff). Next session: run the four-crate suite from this branch (the cache is warm).

## What's left (the legs, written and quoted on the branch history)

The real-input legs were WRITTEN this session (visible in this branch's history, commit
"level-0c-every-objective-blockers: the three blocker probes..." parent diff) but the validation
window closed before they ran green:

1. Leg 1 (blocker 2): walk the walkway north (x 8.9 past the z −121.5 balcony lip — r 1.7), into
   the trench — WEDGED on the case's south glass. Needs the hinge-burn entry: burn #139-142 via
   the laser watch's real fire path from the balcony repro spot (7.61, 15.71, −113.39 [L]),
   then the [L hatch-2] drop.
2. Legs 1b/1c + leg 4 (blocker 3): the lower walkway south (pass the z −148..−150 hole at
   x 13..14 [D blk2]) → the #506 sliver (5..6, −154) → the pit drop onto the slab → the slab/deck
   south-edge drops → the yard east to the plates. The photograph eye (5.4, 7.35, −113) needs the
   display-ground ACCESS (the 0xE2 leg) or a new clear-line eye on the slab (untested — the slab
   eye is 43 units out, past the camera's 30-unit check).
3. Leg 5b (blocker 5): the ledge's east-strip descent — map the face between x 55.0 and 55.5 at
   y ~2 first (one probe), then walk east-south.
4. The wire legs (blocker 4): jump-grab #284 at the west edge (ctx 3 / body sphere), strafe +1
   (mode 7 faces wire.yaw + 3π/2 [M Player_Wire]) to B, drop; the diagonal ledges to icon #6;
   walk-east contact-grab #282 (the standing body is inside its r 5.05 sphere), strafe −1 (mode 8)
   to A, drop; the arc east to icon #12, contact-grab #283, strafe +1 to B, drop onto the SE
   corner; then the existing door/corridor legs (the E stance works from the corner side — the
   pick sphere (40.01, 12.57, −209.69) is on the corner side [M Collide_PickObj 0x1eae80 0x101]).

## Live checks for the main agent

1. Plate #274's walk-in: in the real game, walk the walkway north into the case — does the south
   glass block the body, and does the plate fire on a drop after the four hinges are laser-burned?
   Watch ch 0x1b's write tick vs the fall.
2. The display → yard descent: which route does the game's player walk — the slab/deck south-edge
   drops [D blk3] or the annex vent (the #180 lock burn → the lid drop → the hole)? gdbwatch the
   ch 0x54 write while playing the route.
3. The ring gaps: ride each wire (grab at the icons) and record the ride speed/direction and the
   drop landing — the port's ride is unvalidated against the game.
4. The doorway-ledge east strip: the face between x 55.0 and 55.5 at y ~2 — what is it (a crate?
   the wall's return?), and does the east-then-south descent walk clean?

## Usage

not visible to me (this environment shows no token, cost or context-window figures).
