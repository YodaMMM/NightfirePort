# level-14-full-walk — leg 1a rewritten onto the east route; the Q-Claw shaft entry works to the hang over the east vent lane; the release-onto-the-lane leg is the one red step; legs 2..6 untouched

Branch `s/level-14-full-walk` = `origin/master` (1a3ba0b4) + this round. Checkpoint pushed at
the leg-1a rewrite.

## What is done

1. **Leg 1a is rewritten** in `crates/nf-view/tests/walkreal_07000014_live.rs`
   (`walk_the_infiltration_to_the_delta_exit`, UN-IGNORED) onto the pinned east route, and the
   walk runs GREEN through: the crossing stance → the SkyBridge Q-Claw latch/reel/hang/release →
   the trench floor (46.00, 0.015, 63.22) → the trench east (48.17, −0.05, 62.90) → the corridor
   south → the pit-band jump west → the east stairwell foot (45.14, −0.10, 53.25) — the pinned
   chain verbatim.
2. **NEW Leg 1d — the z-51 WEST pit jump** onto the 0.60 stairwell-corridor segment
   (x 44.4..46.4, z 50.5..51.25): walked GREEN in the test
   (landed (46.29, 1.26, 50.75)). Evidence: the segment floors [D probe14_shaft2 grids A/B,
   committed as `crates/nf-view/examples/probe14_shaft2.rs`]; the jump is the pinned pit-band hop
   pattern [M the run-hop cadence], +1.2 rise over the −2.40 pit band.
3. **NEW Leg 1e — THE SHAFT ENTRY, solved as a Q-Claw reel**: the task note's "east shaft up to
   the east vent lane" is NOT a climb — the shaft throat (x 44.75..45.75, z 49..50) is a
   floor-less well open past −20 ([D probe14_shaft3 rays A/B — committed
   `probe14_shaft3.rs`]), the lip ramp (z 48.0..48.6, top 2.01..2.40 [D probe14_shaft4 —
   committed]) is +1.41 above the 0.60 strip and the sim's jump apex is 0.78
   ([M Player_HandleJump: vel = −GRAVITY·0.4, GRAVITY 9.8 — crates/nf-game/src/player.rs:1541]),
   and the level has ZERO ladders ([D probe14_ladders — committed:
   `sim.ladders` empty]). The game's own nav web has no node above y 1.0 in the vent band
   ([D probe14_nav — committed]) — the vents are Bond-only. The WORKING entry: from the segment
   stance the Q-Claw bolt latches the mouths' grapple **#626 (45.30, 4.79, 42.86) [D
   placement]** — the ray from (45.09, 1.23, 51.0) skims the lip with 0.03-0.09 clearance and the
   bolt's 1.0 sweep catches it [M Bullet_Delete 0x12e660:50-55; 0x12c5a0:92] — and the reel
   [M Player_SSGrapple 0x192860:79-121] pulls Bond up OVER the lip, hanging at
   (45.37, 4.59, 43.91) directly above the east vent lane (2.40). This is GREEN in the test
   (`the mouths hang`), and the refuted-from-the-corridor ray (level-14-qclaw-climb) does not
   apply from the segment.
4. The lane/mouth/mezzanine geometry is mapped and committed as evidence: the east vent lane
   floor is a thin plate (top 2.40, underside ~2.0-2.3) over a −2.40 trench with entry stairs at
   its south end ([D probe14_tunnel A/B/C — committed `probe14_tunnel.rs`]); the lane runs
   z 48.5 → 42.16 (the flush lid: 2.38→2.40 at z 42.3..42.2 [D probe14_mouthfin A — committed
   `probe14_mouthfin.rs`]) and continues to z 38.25, where a continuous sloped connector
   (2.15..2.40, z 39.5..38.25, x 41.5..45) joins the west lobe; the #301 drop hole is the open
   band z ≤ 38 [D probe14_mouthfin C/D]. Leg 1f walks the lane south and the connector west to
   (40.7, 2.3, 39.0) with `try_walk_to` (not yet reached — see below).

## What is left (the exact next step)

**The release-onto-the-lane leg is the only red step.** The walk reached the hang
(45.37, 4.59, 43.91) over the lane; the last release attempt (turn-only north 45 ticks →
north-forward 50 → idle) ended with Bond STILL on the wire at (45.37, 4.22, 49.09). The release
rule is [M gadget::grapple_tick / Player_SSGrapple 0x192860:
crates/nf-game/src/gadget.rs:490]: phase 1 (the reel) reads NO fire input and cannot be
interrupted; phase 2 (the hang, d < 2.0) detaches on the FIRST no-fire tick (`!fire`), zeroing
velocity — so after the phase-2 break the idle ticks SHOULD detach and drop Bond straight onto
the lane (2.40) at (45.37, ~2.95, 43.91). The observed no-detach at y 4.54 with `grapple`
apparently still `Some` contradicts that reading — the next session should print
(grapple.is_some(), phase, grounded, substate) every 10 ticks inside the release loop and
reconcile; if the wire truly holds, check whether the reel loop's last `tick_fire` left the
phase at 2 with the fire edge semantics differing (the Walker's tick_fire holds fire every
tick; the detach needs one clean no-fire tick). After the lane drop: leg 1f (the lane south
over the flush lid, the connector west to #301) and the unchanged ch 0x47 wait, then legs 2..6
EXACTLY as they were (they were green on the old route and are untouched). Then the full suite.

## What ran

- `cargo test --offline --release -p nf-view --test walkreal_07000014_live
  walk_the_infiltration_to_the_delta_exit -- --nocapture` (NF_ISO set, release): the walk legs
  through the mouths hang all green (the log: the crossing landed, the corridor east, the
  stairwell foot, `seg true`, `the mouths hang (45.37, 4.59, 43.91)`), failing at the release
  assert. The other two tests in the file were not re-run this window.
- `cargo test --offline --release -p nf-view --test scratch_shaft_entry -- --ignored` (scratch,
  teleports): the segment stance → the #626 latch (pitch 0.191) → the hang (45.25, 4.52, 44.08)
  — the shaft-entry mechanics proven before the test encoding.
- The full `tools/test/suite.py` did NOT fit: the sandbox kills background processes between
  tool calls (the started run died with a 0-byte suite.out — the same failure the
  east-walkway/qclaw sessions logged), and the foreground window went to the leg iterations.
  The next session must run it foreground-sliced on this tree (the probe examples compile
  clean; nf-view's walkreal_07000014_live is the touched binary).

## Live checks for the main agent

Fresh boot 07000014:

1. Stand Bond on the 0.60 stairwell-corridor segment (≈ (45.3, 0.60, 51.0)), aim the Q-Claw at
   grapple #626 (45.30, 4.79, 42.86) and hold fire: does the REAL game latch from there (the
   lip-skimming ray), and where does the reel carry him — the hang over the lane, or past it?
2. At the phase-2 hang, release fire: does the wire detach on the first no-fire tick and Bond
   drop straight down (the port's reading of 0x192860:64-66), or does the game hold him until a
   stretch/pull limit? gdbwatch the grapple state word during the release.
3. From the hang over the lane, what input does the game's Bond use to land ON the east vent
   lane (a south drift shortens the wire and never releases; north drifts run off the lane's
   north end in the port)?

## Usage

not visible to me (this environment exposes no token, credit or context-window figures).
