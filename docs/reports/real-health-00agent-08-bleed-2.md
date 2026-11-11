# real-health-00agent-08-bleed-2 — report

**Branch `s/50-real-health-00agent-08-bleed-2`, base origin/master `784ba2d99`.** (The driver header
says branch `s/real-health-00agent-08-bleed-2`; the task body and finish.sh say the `50-` prefixed
name, matching the queue item, so that name is used.)

## What's done

**The upstream bleed lever (1) from the -1 ledger is taken and proven — the Hind fight now starts
with the margin it asked for — and the mission endgame's next blocker is measured and ledgered.**
The ignore on `walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage_00agent` is
NOT removed (the walk is not green yet); it now carries the updated measured ledger and next levers.
All changes are d3-gated; the sisters' timelines stayed byte-exact.

1. **The east-hall clear** (the ledger's lever 1, first option): the six d3 pistol guards hold
   their east placements (152.5-161.5, 46.0, −87.7..−101.5) and the walkway legs' fights ran
   inside their bursts — the retarget churn ([rf-trace] fight#11 t30 retarget, fight#3 t210
   retarget) killed nobody while 23 × −4.0 = **92.0** drained (ticks 684-1683, [L the 2026-11-11
   run2 NF_DMG_TRACE run on master]). The walk now crosses to the deck vantage the churn stands
   themselves measured (150.0, 46.1, −96.4) and kills the men one at a time with the shoot
   machinery's single-target stands (`shoot_bounded`, nearest first, no visibility filter — the
   visible-only pick let #9/#10 live past the loop and the next leg crossed their lane at 1.0 u
   [L run3]). Hall cost: **16.0** [L run4].
2. **The drops scooped** after the clear and at the stairwell foot (d3): the fights' ammo pool —
   the run2 boarding legs opened at clip 7 spare 1 and the next fight's out-of-ammo resupply walk
   went to a drop 3.4 u from the burst man; the bursts ate the last 46.0 (death tick 27334 [L
   run2]).
3. **The fusebox doorway re-fought after the laser burn** (d3): the burst man #6 walked in during
   the burn — 18.0 at ticks 9586-9662 [L run2]; the pre-burn check alone missed him.
4. **The boarding lip ring 12 → 16 on d3**: #26 burst from 8.5-8.6 u then retreated past the 12 u
   ring, so no lip stand ever opened on him (the ledger's own "outside the 12 u scan" note) —
   24.0 per run4.
5. **The Hind east-phase answers** (d3): the machine's path-0 circuit was EAST at the fight start
   (parked (141.0, 19.2, −212.6) [L run4/run5 flight samples]) — the opposite phase from the -1
   ledger run — and its sight through the east PANES (the masked sight drops mat-0x10 [M
   0x1ebaa8:314-316]) fired continuously: the wall-blast strafe bled **47.0 in 450 ticks** while
   line_ok never opened. Built and measured: the dodge window 30 → 60 u + a sustained-fire jink;
   the east-parked walk-out (a 16-ray solid-cast azimuth sweep — the flight-cast version walked
   Bond INTO the east panes and died closer [L run9]); the terminal crouch (the bleed 47.0/450 →
   17-22 over 3300+ ticks [L run12]); the phase-crouch (crouched under the glass on the high
   phases, standing for the machine's LOW passes). **Rocket 1 LANDED for 101.0 at t2668** through
   the east panes at the machine's low-east dip (y 11.8-15.1 at x 162-170) [L run14].
   Bond still dies at fight-t ~3800 to the high-phase wall blasts (the machine looks DOWN through
   the glass — the crouch does not hide from above) before the ~6 rockets 598 hp needs.

## How it was verified

All runs on the slim ISO, release, `--test-threads=1`, 2026-11-11, with NF_DMG_TRACE (and NF_RF_TRACE
for the fight-churn diagnosis):

| run | result |
|---|---|
| walkreal_07000008_live whole binary, final tree, `--include-ignored` | 5 passed / 1 failed — the failure is the ledgered d3 walk itself; the d1 real-damage, d2 Agent, both plain walks and both ally tests GREEN (twice: before and after the fight edits) |
| interactables_07000008_live | 15 passed / 0 failed (twice) |
| the d3 walk, per iteration | run2 death tick 9592 (25 hits) → run4 27334 (19) → run5/6 25533 (57) → run12 28523 (38) → run14 29039 (40, rocket 1 landed 101.0) |

The full suite did not fit this sandbox (the same ENOSPC ceiling the previous sessions hit; the
`target/` cache plus the ISO leave ~0.3 GB free and the four-crate build needs ~5 GB). My changes
touch one test file only; the whole affected binary + interactables ran green.

## What's left

- The Hind fight's endgame: the machine's circuit passes low-east every ~2400 fight ticks and each
  pass launches 1-2 rockets, but the high-phase wall blasts (the machine at y 16.8-20 looking down
  through the east glass) kill Bond before ~6 rockets land. Next levers are in the ignore message:
  a spot whose sight the machine truly lacks (the west band's shadow — the only inside candidate),
  or the machine's fire-state window (st 1 transit vs st 2 shooting). The walk needs ~4 clean
  passes.
- Live checks for the main agent (07000008, 00 Agent):
  1. At the Hind fight with the machine east: does the REAL game's machine see/fire at Bond
     crouched inside the terminal through the east panes (the masked sight drops mat-0x10 [M
     0x1ebaa8:314-316]) — read the machine's fire cadence vs Bond's stance each second.
  2. The machine's path-0 phase at the fight start: record path/t for the -1 ledger's timeline vs
     this branch's (the fight's outcome is phase-dependent; the phase is set by the upstream
     timeline ~±700 ticks).
- The suite on a roomier sandbox.

## Usage

- input tokens: not visible to me
- output tokens: not visible to me
- context used: not visible to me
- cost: not visible to me
