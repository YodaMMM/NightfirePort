# real-health-00agent-08-bleed-6 — 07000008's 00 Agent walk: the cannon step-out lever taken and measured

Task: take the bleed-5 ignore ledger's next lever (bias the d3 dodge to step OUT of the west
doorway's z band on a cannon zap, the anchor walk stays for drift, the anchor stays the firing
spot), judge it on real runs, keep every block gated to difficulty 3, and run the whole
`walkreal_07000008_live` binary and `interactables_07000008_live`. Branch
`s/real-health-00agent-08-bleed-6`, base origin/master 4f3802517.

## What's done

The lever is TAKEN, measured on five real runs (each the full mission walk on real input from
the load, no top-ups), and the walk is still red — the refreshed ledger and the next lever are
in the ignore message on
`walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage_00agent` (update 6).
All changes are in `crates/nf-view/tests/walkreal_07000008_live.rs`, all d3-gated; the d1/d2
sisters and the ally tests stayed byte-exact through every iteration.

Shipped d3-gated blocks (all inside the Hind fight loop):

1. **The cannon step-out latch** — a cannon zap (a world hit: the Hind's cannon rounds, owner
   past the drone list — the bleed-5 class fix) latches a step-out; strafe episodes chain on
   the latch until Bond's z leaves the doorway band's north edge at the strip's z −177.6
   ([L run-bleed5-3: the cannon stopped there]), then the latch clears and the drift walk
   returns him to the anchor (112.5, −179.4). The arm is gated to inside the band (north of
   the edge the spread still lands — run-bleed6-3's −177.38 rows — and a fresh arm met the
   arrival-cancel the same tick and parked him in the fire, 21 ticks stationary at
   (112.61, −177.38), f32053-74 [L run-bleed6-3]).
2. **The step-out side** — the strafe sign that walks +z (north) out of the band: the right
   vector's z component picks it (strafe +1 moves +z·right[2]).
3. **The SPRINT law** — the crouched+aiming walk is 0.011 u/tick [L run-bleed6-1: 0.31 u in
   28 ticks], seven times slower than standing; the dodge strafes, the step-out and the drift
   walk run STANDING and Bond re-crouches the tick the movement ends (the crouch press edge,
   one tick [M Player_SSCrouch 0x1924c0:84-105, the Toggle default Input_Init 0x17edd8]),
   skipped on a launch tick.
4. **The anchor fire gate** — d3 `ready` requires Bond within 1.5 u of the anchor. The north-east
   corner is a LAUNCH TRAP: the flight-query sight passes through the east panes at oblique
   angles and the launch detonated on the pane frames −52.0 next to Bond [L run-bleed6-2,
   death t29509].
5. **The arrival cancel** — the step-out episode is cancelled when the strip is reached, so the
   strafe cannot overshoot into the corner.
6. **The cadence diagnostic** — every cannon zap logs Bond's health, the step-out state and the
   live drone bullets' weapon ids; a run summary tally prints with the shoot-down line.

## How it was verified (the five runs)

| run | tree | death t | rockets | notes |
| --- | --- | --- | --- | --- |
| bleed6-1 | the lever as prescribed, crouched | 31380 | 1 (100.2) | 60 zap ticks; the crouched strafe cannot outrun the 25-tick bursts (f27503/f27528); the cannon zapped the strip z too (−177.50/−177.34 rows) |
| bleed6-2 | + the sprint law | 29509 | 0-1 | the cannon class SOLVED: 16 zap ticks, hp held 52.0 for 2000+ ticks — but the episode overshoot walked him into the north-east corner and the corner launch self-blasted −52.0 |
| bleed6-3 | + arrival cancel + anchor fire gate | 32706 | 1 (100.2) | the BEST death — 2540 ticks past update 5's t30165; the cannon spread reaches every cabin spot (rows at −180.95, −179.4, −178.1, −177.61, −177.38 across the runs); still 1 rocket |
| bleed6-4 | + the inside-band arm gate | 32706 | 1 (100.2) | same deterministic path/outcome |
| bleed6-5 | + the phase-crouch variant | 29593 | 0 | the phase-crouch (standing for cp[1] < 15) measured WORSE — ZERO launches, 37 world hits for 34.3 — NOT shipped |

Cadence answers the ledger asked for: the machine's cannon does NOT pause while its missiles
fly (zap rows with live [98] — a missile in flight — at f29462 and f30555 [L run-bleed6-2/3]);
bleed5-3's "cannon-silent strip" was a circuit phase, not a law (hits land at the strip z
across the runs).

Binary results on the shipped tree (post-revert, commit 049f8c6c2):
- `walkreal_07000008_live` whole: **5 passed, 0 failed, 1 ignored** (the ignored one is the red
  00agent walk carrying the ledger) — 111 s.
- `interactables_07000008_live`: **15 passed, 0 failed** — 15 s.

## What's left

The Hind fight still loses: the binding constraints are (1) the launch windows are rare from
inside the cabin (1-3 per ~10000 fight ticks; 598 hp at ~100/rocket needs ~6), (2) the missile
blasts (the 24-tick dodge episodes, standing ~1.85 u, cannot beat the homing), (3) the cannon
spread covers the cabin. The next lever (in the ignore message): name the west-doorway sight
window per machine pass (log which machine positions/bearings `eye_sees` + `machine_lane_clear`
catch — the lintel y 14.31 [D the tri dump] vs the standing eye 14.24 / the crouched eye ~13.3)
and place the firing spot to catch more of them; then the missiles (pin the chase's per-tick
steer clamp [M Bullet_update 0x12e7b0:177-244] and dodge with a sustained standing sprint
perpendicular to the missile's line while it is 2-30 u out). Live checks for the main agent:
the missile's turn rate at 10-20 u, and the west-doorway sight window's y band from the anchor.

The fast suite (`tools/test/suite.py`) was started detached on the final tree; the per-binary
results above are the tests that exercise the changed file (only
`walkreal_07000008_live.rs` changed — no shared helpers, no game code).

Scratch logs (run-bleed6-*.log, whole08-bleed6*.log, interact08-bleed6.log, suite.out) are
untracked, never committed.

## Usage

Not visible to me: this session exposes no token, credit or context-window figures to the
worker (no usage metadata is available in the sandbox).
