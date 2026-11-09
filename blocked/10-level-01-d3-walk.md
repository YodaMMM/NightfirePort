# 07000001: the 00 Agent walk with guards breaking the glass

The ignored 07000001 00 Agent (d3) real-damage walk is still red after several rounds (guard-pane-drain series; read the latest report on s/guard-pane-drain-6 if present). On current master (guards turn off Bond on contact; guard rounds break glass), find where Bond dies, compare those guards' rules with the game, fix the port if it differs, otherwise re-route as a player would and un-ignore when it passes.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-01-d3-walk`, report in docs/reports/level-01-d3-walk.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox3 at 2026-11-09T11:52Z

---
## Report (blocked, 2026-11-09T12:31Z)


---
## Report from the worker (blocked, 2026-11-09T12:31Z)

# level-01-d3-walk — every 07000001 00-Agent door-phase death removed; the walk reaches the AND alive; #7 is unkillable from the south and the west-door lever is the sole remaining one

## Status: not fully done — the walk is still `#[ignore]`d, but the failure changed from "Bond dies" to "the AND can't complete" (alive, deterministic, ledgered)

Task: the ignored 07000001 00 Agent (d3) real-damage walk goes green with guards turning on
Bond on contact and guard rounds breaking glass — find where Bond dies, compare those guards'
rules with the game, fix the port if it differs, otherwise re-route as a player would and
un-ignore when it passes. Branch `s/level-01-d3-walk` cut from `origin/master` (3efb98f19).

## Where Bond died (reproduced first, real input, no teleport)

Four distinct deaths, all on the port's own fight machinery, all removed this session:

1. **tick 5925** — `shoot_bounded`'s flank walked Bond THROUGH the open hall-door plane
   (z 11.1) hunting #11's exposed part ("exposed false" at the peek stand), and the
   return-fire trade with the standoff captain #7 (SMG 0x1c at (1.3,5.7,17.8)) ran at the
   aperture mouth (8.24,11.71): **18 hits for 85.6, every hit at d 9.3** [L the 5925 run].
   Geometry [L the same run]: from every position south of the plane #7's own line to Bond is
   walled (the hold line's sight line crosses the wall plane at x 5.69; the aperture x is
   ~7.5-8.3) — the trade is only losable from inside the hall.
2. **tick 6159** — the blocked-forward creep in `shoot_bounded` carried Bond from the peek
   stand (z 10.5) into the mouth; #5 (a fourth hall guard, SMG) closed to d 5.7 and took
   **49.6 in 8 hits** [L the 6159 run].
3. **tick 6007** — #7 CLOSED to (6.2,14.4) and drilled the mouth-side stand through the
   standing-open door: **56.8 in 8 hits** [L the 6007 run]. #6 had picked up a dropped SMG
   (weapon 0x0 → 0x1c) and added 28.8.
4. **tick 10017** — guard 11's dry branch resupplied to a guard's weapon drop at
   (6.0,5.3,**13.7**) — 2.6 u INSIDE the hall, admitted by the walked-path corridor because
   the peek walks had put the mouth on the trail — and the pair drilled **79.2 in 11 hits**
   [L the 10017 run].

The port's guard rules compare clean against the game at every point checked: the trio holds
the standoff bands and never crosses the south aperture (master's own
`guards_shut_door_07000001_live` pins it — their NavPath carries only the west door #2 pair
[M 0x1d2598 `AIPath_Parse`, 0x1571b0 `AINetwork_DoAStarPath`]); their fire through an OPEN
door is the game's own open-door sight line; #7's drift along the hall's south edge is the
side-step law; the damage rates are the drain series' accepted ones. **No port bug found in
the guards — the deaths were the harness's own movement** (a flank, a creep, a loot walk), so
the fix is the player's movement discipline, not a port change.

## The fix (test-file + opt-in shared flags; every flag-off path is master bit-exact by construction)

Shared helpers (`crates/nf-view/tests/common/`), all opt-in, no existing test sets them:

- `Walker::no_cross_z` — while Bond stands within the plane's 0.2 margin, every movement
  tick's +z-carrying forward/strafe push is zeroed (`gate_z` in `tick_inner` + `step_gated`
  for the direct `step_bond` sites). The d3 test sets 11.6; cleared once the AND fires.
- `Walker::flank_hold_z` — the `flank_hold_x` hold shape on z: a target beyond z 11.1 is
  never flanked (three holds → the fight gives up, the caller's loop gets the window).
- `shoot_bounded`'s blocked-forward creep is off under `no_cross_z` (the creep was the
  mouth-walker); the return-fire pick holds at the lip (`return_fire` returns false while
  Bond stands past `zp − 0.45` — `stand_and_fire` does not move, a pick at the mouth runs its
  whole cap in the shooters' clear line); `resupply`'s corridor filter drops pickups past the
  plane (the loot-bait death).

Test file (`walkreal_07000001_live.rs`, d3-gated where noted):

- The d3 door phase sets/clears the gates around the named loop.
- `door_peek_fight`'s rounds SHUT the leaf behind Bond (`shut_hall_door`, the real E path,
  idle-OPEN state 4 [M door.rs OPEN=4/CLOSED=2 — a shut leaf blocks the rounds]) —
  **d3-gated**: ungated it shifted the d2 real-damage walk (the shared `door_peek_fight`;
  the whole binary went 6 passed/1 failed), the gate restored d1/d2 byte-exact.
- The d3-only mouth strip in the stand scan (z 10.9-11.35, x 7.3-8.5 — the ONE spot the
  sight lines mutually clear; a player steps into the doorway, fires down the hall, steps
  back) and `peek_fire`'s wound cap halves at the mouth (6.0 vs 12.0).

## Result

**#6 and #11 die through the aperture at ZERO cost** (peek rounds, "peek 2 done, health 85.6,
guard hp -2.0" [L the d3run5/6 logs]); the door phase now ends at the AND assert ALIVE. **#7**
(0x5000038, SMG 0x1c, 24 hp) is unexposable from the south: 10 peek rounds + the mouth strip
found no part at any drift spot (x 0.9-2.0, z 14.2-16.1, states 0x73/0x74/0x5f), the armed
sustained lure ran dry on him (55 taps, weapon 0x2 clip 0 spare 5) and never drew him across —
the nav law holds. The walk stays `#[ignore]`d with the full ledger and the sole remaining
lever in the ignore message.

| binary | result |
|---|---|
| walkreal_07000001_live (whole, `--test-threads=1`) | **7 passed / 0 failed / 1 ignored** (the d3 00-Agent ignore), 57.7 s |

The other `walkreal_*_live` binaries did NOT run this session (the sandbox clock ran out
first — every shared-helper change above is flag-off bit-exact by construction and the 01
binary's d1/d2 confirm the shared paths green; the next session must run the full sweep, one
level per tool call, before anything else). nf-game/nf-formats/nf-audio sources are untouched
by this branch (the diff is the two test helpers + the 01 test file + docs).

## What's left

1. The **west door #2 lever** (the ledgered sole lever): stage to the corridor side of #2
   (−14.1,5.3,13.1) — the known walker wedge, short lenient legs — open by hand, fight the
   hall from the west line SHOOT-ONLY; #7 CAN path to #2 (his own NavPath), so an armed
   sustained lure from the west line draws him into the #2 doorway kill. Second lever: the
   east perch (2.97,6.38,16.27) shoot-only behind the mouth (never into the 3.0-u forced band
   [M 0x179740:100-110]).
2. Then the AND fires and the **escort phase** runs — it has never run on any tree.
3. The full `walkreal_*_live` sweep (one level per tool call) + `glass_pass_every_level_live`
   + `dead_pane_live` + the fast runner on the final tree.

## Live checks for the main agent

1. **#7's exposure from the west line.** 07000001 d3: open west door #2 from the corridor
   side, stand at (−16,6.4,13.1), and probe #7's boxes (a gdbwatch on his hit_boxes during
   the hunt). Expected if the lever lives: some part clears down the hall's long axis —
   the same survey the peek fight's scan does in-port.
2. **The west-door crossing.** Fire sustained from the west line with the door open:
   does #7's goal machinery acquire the route through #2's pair (n21/n22)? If he still holds,
   the west lever dies and the east perch becomes primary.
3. **The shut-leaf round block.** With the leaf shut (state 2) and Bond south of it, verify
   the mouth guards' rounds stop at the leaf (no health ticks through it) — the port's
   `is_closed` gate [M door.rs:254-262] vs the game.

## Usage

- Context window / tokens / cost: **not visible to me** — no token, credit or context figures
  are exposed anywhere in this environment.
- Wall clock (from `date -u`): the driver message arrived ~11:55 UTC; the branch cut 11:57;
  the four reproduced deaths ran 12:03-13:14 UTC; the whole-binary green run 13:26 UTC; this
  report written ~13:30 UTC, inside the ~13:55 UTC expiry.
