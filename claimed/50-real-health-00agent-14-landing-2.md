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
