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

Main 11-14: five switch-band stands (radius 15/20, was 3) - death t27276 in the Kiko hold, worse than t29197. Not shipped.

Main 11-14: crouched from the node-618 corridor leg - dies at the door at t25407 (worse). Crouch hurts 14 in every placement tried (holds, corridor).

---
claimed-by: sandbox5 at 2026-11-15T00:06Z

Main 11-14: armour pickups (kind 3) live at the Kiko hold (t26154, Bond 27.9/0 at (-51.3, 37.9)): (-41.3, 1.5, 36.5) at 10.1 u and (-27.3, 0.9, 31.0) at 25 u. A walk_nav to the near one routes SOUTH through the squad (via (-46.6, 28.4)): return fire kills #31/#29/#20/#23/#22 on the way but #61 (0x5000093, 0x14) ends it at t26950 at (-45.0, 28.0) - worse than t29197. Not shipped. The stock room at x -32.2 (z 31-36) holds weapon 0x1a/0x34/0x3a and ammo 0x15/0x1a/0x1b.

Main 11-14: the escape-room armour (-27.3, 0.9, 31.0) right after the escape cut-scene: Bond arrives there with ~3 health, #63 (0x5000090, 0x14) at 4.7 u ends it at the room door (-30.6, 29.9) at t28980 - worse. The health is spent in the holds before the touch.

Main 11-14: the biggest single bleed is the south loop corner (nodes 42-44, t11354-11395): Bond walks into #14/#15 (0x14) at 1.0 u, 35.0 in 40 ticks. Measured stops: at node 41 (-21.4, -21.1) four 400-tick rounds never land on #14 (11 u, nothing exposed), the gun runs dry and the walk dies in the Kiko hold (t40789, a stall-inflated tick, NOT further along); at node 42 (-22.48, -19.97) both die (#14 at 9.8 u, #15 at 4.5 u) at 50.4 health (master 37.9) - but the Kiko corridor is reached at the same 27.9 and a -22.91 blast from the already-dead #28s explosive round ends it at t26516. Not shipped; the corner fight saves 12.5 that the later stretch loses again.
