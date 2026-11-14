# 10-level-0d-operative-walk-10

0700000d's Operative real-damage walk (`walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage`) is still
ignored. Master 36db1e785 carries rounds 7-9 (the walkers' crossing sweep hoisted before the roof retarget): the
death is still in the ladder-top phase, t4161 at (213.07, 17.37, -234.51), 10 hits for 100.0 - the trio's first
64.0 lands before the sweep opens (~3800: #29 32.0 at 3042/3222, #27 at 3398/3578). Read the ignore message's
round-9 ledger and take its next lever (answer #29/#27 before the climb, from a line the scans already found).
Real input only, no top-ups; the plain replay and the d2 twin stay byte-identical.
Tests: walkreal_0700000d_live, interactables_0700000d_live. Branch: s/level-0d-operative-walk-10.

---
claimed-by: sandbox4 at 2026-11-14T18:26Z

Main 11-14 (master 06db4a156, NF_DMG_TRACE): the ladder-top bill is the 0x1e SNIPERS - #28 14.0 mid-climb (t2425), #29 16.0 x2 (3042/3222), #27 16.0 x2-3 from the west roof (151.4,22.7,-245..-256) at 62-65 u, #16 16.0 from the deck, #10 16.0+6.0 from the bay; the climb itself takes 864 ticks. A west-roof sniper pass FIRST at the ladder top (kill_guard_at_present (151.2,-248.0) +-(6,12), 900) kills one (+224) and the death moves t4161 -> t5052 (#27 still alive, 6.0 health idling); three passes die inside them (t3664). Next: all three west snipers before the deck/bay acts, or from the ladder base before the climb.

Main 11-14: the west-roof snipers from the LADDER BASE (kill_guard_at_present (151.2,-248.0) +-(6,12), 600) - no line (pass false at 100.0); the walk then dies at t3947 (one fewer hit absorbed... same phase). The base has no sight of the west roof.
