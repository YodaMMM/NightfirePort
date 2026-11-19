## retune-nb-0d-6 (2026-11-19, s/retune-nb-0d-6)

Task: round 6 of the 0d arrival-to-C5-door 00 Agent walk `walk_arrival_to_the_c5_door_real_damage_00agent`.
Branched from origin/master 72a68f828 (origin/s/retune-nb-0d-5 does not exist on the remote; master already
carries the rounds 1-5 shipped tree per the ignore-message ledger). Round-5 remaining bug: the committed
shift-30 roll rides alive (2 ride chips, clean landing, exit 16.8) but the yard-floor converged garrison's
4-chip/6-tick burst outraces the sweep's first kill; the round-5 armour walk-over reshape is measured dead
(the wake law — any pocket visit wakes the west-roof #32's sight cadence onto the climb phases). This round:
the round-6 levers from the ignore message — (A) the g18 ride lottery at the untried knob families
(NF_0D_TAP_ZONE {40,45} x NF_0D_TAP_CADENCE {15,17} x NF_0D_RIDE_SHIFT {30,60}, each config its own roll),
(C) a pre-ride corner shave that does NOT re-roll the ride. Helpers stay gated to level 0d + 00 Agent;
real input only, no top-ups, never lower damage or health, never change guard code.

### retune-nb-0d-6 progress

- Branch off master 72a68f828, WORKLOG started (docs/WORKLOG.md does not exist on master; rounds 1-5 entries
  were stripped from master on merge — their ledgers live in the queue done/ files and the test's ignore
  message).
- Lever (A) (the round-6 ride lottery) MEASURED DEAD: eight fresh real runs, every untried knob family
  (NF_0D_TAP_ZONE {40,45} x NF_0D_TAP_CADENCE {15,17} x NF_0D_RIDE_SHIFT {30,60}) dies ON THE RIDE — the
  girder health (24.0 at shift 30, 16.8 at shift 60) drains to 0 by the container-top landing (fall timer 99,
  landing pos (149.26, 8.23, -296.08) identical across all eight), the panic is the rim sweep's own guard
  ("the rim sweep must not kill Bond on the top", walkreal_0700000d_live.rs:1925); the 40/45 tap zones fire
  extra taps on the ride and the extra draws re-roll the ride's fire onto the rider — the same dice law the
  round-4 cadence hunt measured, now proven for the zone axis too. The baseline (committed defaults) reproduced
  byte for byte first: death t19772, 21 hits for 80.8, other 19.2, #22 46.0, the floor-landing burst
  (#3 10.8 / #1 9.0 / #2 6.0), 0.0 real-damage test FAILED as committed.
