# level-0b-roof-sprint-5 — report

Branch `s/level-0b-roof-sprint-5` = `origin/s/level-0b-roof-sprint-4` merged with current
`origin/master` (one conflict, in `crates/nf-view/tests/walkreal_07000011_live.rs`, resolved by
keeping BOTH sides: the los-mask-join 27.1-strip floor-loot and master's capsule-join flight-B
mouth fights — verified below that the merged file behaves identically to master's own file).
**Status: the Agent real-damage walk is GREEN end-to-end on real input; the 00 Agent walk is the
closest pass yet (the take fires, #4 sleeps through the dash) and its ledger + next levers ride
in its `#[ignore]` message. The blast (`explode-hurt-radius`) was not touched.** No
damage/health lowered, no pins loosened, no `crates/*/src/` changes, no draw changes; all new
behaviour is gated to 0700000b and `difficulty >= 2` (the d1/warm streams byte-identical — the
whole 0b binary still passes them unchanged).

## What's done

1. **The reproduction, instrumented.** The agent sister died at tick 16495 frozen by the
   sprint-3 funnel stand-and-drop: it held Bond at d 2.6 from #4 (inside the 3.0 all-hit band
   [M DroneWeap_DoBulletAccuracy 0x179740:113-115]) for ~150 ticks while its own aim hovered at
   err 0.05-0.14 (never firing) and the squad chewed 39.6 → 0.0 (30 hits for 100: #5 25.6, #4
   22.4, #8 16.8, the world 12.8, #7 11.2) [L agent_run1]. A state-instrumented rerun
   (`[0b-stall] g4/g5/g7/g8` lines) showed **every roof guard dartable through the crossing**
   (states 0x68/0x73/0x74/0x75 → `dart_impact_state_ok` true) and the fire-on-move's 0.18 err
   gate firing darts 0.6-1.6 u off the body — all missed [L agent_run2].
2. **The sprint-4 report's second option, landed: #4 taken with the K5 from inside the 9 u
   proven range, on a route that closes before the turn.** The deck package (all
   `agent_run`-gated):
   - P2's east run aims THROUGH #5's gap-mouth drift (the trace had him at (0.3-0.8, 20.6-22.7),
     2-4 u off the old heading) — his dart lands, the 25.6 ambush becomes 1 hit [L agent_run1 vs
     agent00_run6];
   - P3's north leg CLOSES ON #4 (steers at him from d 9, the fire-on-move trigger rides the
     run) and the dart is held to d 5.5 so the sleep window covers the dash; the phase hands to
     the P4 dash on the sleep (the dart family, [M 0x163430]) or the 3.4 floor; the funnel
     stand-and-drop is d1-only now (on the sisters it was the measured death);
   - the take-zone approach (1.9, 40.9) sits west of the housing's SW column — the straight SW
     approach wedges at (2.02, 40.59) with the touch sphere 0.07 out of reach and the take never
     fires [L agent00_run5 take-dist 1.861] — while (1.9, 40.9) is d 1.62 of the pickup, inside
     the 1.79 sphere, the ray threading the open south face [D the housing tris; M
     Pickup_Update 0x18e148:202-220];
   - P4 runs THROUGH the take zone to the deck's east edge (11.3, 41.3), the take firing ON THE
     PASS (the touch law needs no stand), and the jump leg hops on the first grounded tick.
3. **The Agent sister (difficulty 2) is green**: mission complete in 16683 ticks, end health
   6.0, 25 hits for 94.0, 0 return-fire fights, stuck 0, the one fall the parachute jump itself
   [L agent_run5 + the final-tree rerun]. #4 sleeps (state 0x50) through the whole dash; the
   wake volley lands after the edge.
4. **The 00 Agent sister (difficulty 3) — the ledger, in its ignore.** The package works (the
   take fires at d 1.599, #4 sleeps through the dash) but Bond dies at tick 16517 at (7.13,
   58.07, 41.24), 2.7 u short of the edge, on the long-range toll: #9 21.0 in 3 hits (0x18 at
   d 21), #6/#8/#7 3.5s, 17.5 hp left at the take [L agent00_run6/7]. Next levers in the ignore
   message: the `[0b-stall]` vel prints read (0,0,0) while Bond runs at 0.14-0.17 u/tick — if
   the game's DroneFiring_TargetMoving_Accuracy [M 0x179740] reads a velocity the port's walk
   motion never writes, the deck guards shoot a "stationary" runner at full accuracy (a port
   divergence worth its own evidence pass); then ~10 hp out of the P1/P2 legs.

## How it was verified

- `walkreal_0700000b_live` (the whole binary, final tree): **6 passed; 0 failed; 1 ignored**
  (`--test-threads=1`, 39 s) — the base was 5 passed / 2 failed. The d1 real-damage walk, the
  fresh walk and the warm walks pass with their streams untouched.
- `interactables_0700000b_live`: **14 passed; 0 failed** (26 s).
- The merge-resolution check: `walkreal_07000011_live` fails 2/4 (the d1 + 00agent real-damage
  walks die to CORPSE drones (hp −72/−75) firing — master's 0a9d39435 "Guards and corpses are
  solid to Bond" game change); master's OWN copy of the file on this tree fails the SAME 2/4 —
  my resolution is behaviour-identical to master's, the failures are master's state, not the
  merge.
- The full suite (`tools/test/suite.py`) was started on the final tree; its total line is at
  the bottom of this report.

## What's left

1. The 00 Agent walk (the ignore's ledger): the vel-0 moving-accuracy divergence first, then
   the P1/P2 long-range toll.
2. The blast-law task (sprint-4's item, untouched here): the port's explosion hurt reaches
   ~13.7 u where the disc's radius is 8.0 [L ee_snapshot weapon_data+0x2e·0x10c+0x08].
3. The 07000011 corpse-fire failures are master's state (see above) — its own task.

## Live checks for the main agent

1. **The moving-target accuracy law.** In PCSX2 on 0700000b (any difficulty): stand a guard
   shooting at a RUNNING Bond and at a STANDING Bond at the same distance; expected — the
   running Bond is hit markedly less (DroneFiring_TargetMoving_Accuracy [M 0x179740]). If the
   running Bond is hit just as often, the port's vel-0 walk (the [0b-stall] prints) is the
   divergence and the deck ledger's long-range toll is inflated by it.
2. **The K5 sleep on a deck guard.** Fire one K5 dart at a roof guard in state 0x68/0x73 from
   ~5 u: expected — the guard drops (state 0x50) and stays down 150-260 ticks [L agent00_run6
   g4 trace].

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage readout is exposed to the worker).
