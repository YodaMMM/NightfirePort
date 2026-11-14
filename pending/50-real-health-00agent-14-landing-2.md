# 50-real-health-00agent-14-landing-2

07000014's 00 Agent real-damage walk (`walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage_00agent`)
crosses the 1.72 landing on master 25d6fc65e and reaches the escape touch; it dies at t29197 at (-30.20, 1.16,
24.28) on the Rook's-wake legs to #30 (skin 0x5000087, 0x14) with 2.9 health. The health gap is the Kiko corridor:
27.9 into the Kiko hold, then #20 (0x5000090, 0x14, hidden at (-51.2, 32.6)) lands 15.0 in 3 at 2.4 u as Bond
passes node 1. The task file of 50-real-health-00agent-14-landing (done/) lists every lever measured on 11-13 -
do not repeat them. Ideas still open: a crouched pass of node 1, a different west-corridor line on #20 before
the Kiko hold, more health out of the west-face fight (#34 10.0). The NF_HOLD_SKIP collision fix (off by default)
also needs its discriminator so it can be turned on: the pit-band jump arc's east gate must survive it.
Real input only, no top-ups; the d1/d2 walks stay byte-identical. Tests: walkreal_07000014_live,
interactables_07000014_live, probe14_pocket_settle. Branch: s/real-health-00agent-14-landing-2.

Measured 11-14: quiet pass to node 2 then a hold there (300/600) dies inside the hold at t26887 at (-47.41,28.27). Not shipped.

Main 11-14 (master 06db4a156): at the Kiko hold Bond holds the 0x14 9+42, owns 0x34/0x37 grenades (1 each in clip, carried slots 22/25 = 4/4); a CROUCHED duel on #20 from the hold spot (300/900) still dies inside it at t26998 at (-51.42,0.65,37.88). Untried: a frag lob onto #20 at (-51.3,32.4) from the hold (5.5 u - check the frag radius first).

Main 11-14: the t26998 death in every "stay near the Kiko hold" variant is #29 (0x5000093, weapon 0x14, hp 14) at (-49.9,32.7): -5.0 x2 then a -17.91 BLAST at frame 26998 - an explosive 0x14 round, not #20. The shipped stream avoids it by leaving for the second hold. A frag lob on #20 from (-50.3,40.0) never got thrown (the 26998 blast first); dodging landed/bounced drone rounds within 8 u in the holds: t27175 (worse). Next: #29 (hp 14) is the Kiko-side threat to answer, not #20.

Main 11-14: the Kiko hold picking skin 0x5000093 (#29) first dies inside the hold at t26493 at (-46.82,42.38) (worse). Not shipped.

Main 11-14: crouching in the holds (one crouch press at each kiko_hold start) - the Kiko hold ends at 22.9 (was 27.9) and the walk dies at t27365 - worse. Not shipped.
