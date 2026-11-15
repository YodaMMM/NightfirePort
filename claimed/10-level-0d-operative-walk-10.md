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

Main 11-14 (667be11e3, crouched ladder top merged, death t5532): a dedicated pass on the bay sniper #10 (214.7,-289.5) +-(3,3) before the bay duo kills him (+487) but the time lets #14 (south roof (166.3,21.2,-271.4), 16.0 x2) and #27 (west roof, x3) finish Bond at t4399. The ladder top faces ~5 0x1e snipers at 55-65 u, each kill costs 200-500 ticks while the others land 16 every ~200: the stand order alone cannot win; it needs cover between kills (crouch helped one hit) or the snipers thinned before the climb. Second west pass before the bay: t4025.

Main 11-14 (667be11e3, crouched): a second west-roof pass before the bay duo does not land (pass false, 38.0) and the walk dies sooner, t5245 vs t5532. Not shipped.

Main 11-14: a ground scan before the climb finds the snipers in clear line from most of the yard (186..216, -240..-222): #27 63 spots, #28 71, #29 66, #16 59, #10 49, #18 30 (#17 none). West-roof passes from the ground at (210,-237)/(204,-237) kill one then die on the open ground (t3261/t3198) - the whole yard is in the snipers lines; the crouched ladder top (t5532) stays the best stand.

Main 11-14 (merged 272ec32c8): the yard armour (kind 3 at (205.4, 0.1, -205.3)) from the blind strip before the climb (Operative real damage only) - 50 taken, 34 back at the strip; death t5532 -> t10148 on the deck at (202.0, 16.0, -255.1): #14 (0x5000087, 0x1e) from (166.3, 21.2, -271.4) 16.0 x4 at 39.6 u, then the bay pair #18/#10 from (206-208, 12.1, -288.5) at 34 u. Other live armour: (192.8, 10.8, -272.0) 42 u, (147.0, 21.8, -252.8) on the west roof, (107.7, 6.8, -275.2).

Main 11-14 (merged afec3f768): the south-roof pair act at 2 x 900 (Operative) lands #14 at +889; Bond reaches the gantry ride alive at 29.8 and dies ON it at t18516 at (144.1, 14.6, -296.1) - two sniper hits (16.0 at t18156, 12.6 at t18516). The ride loop steps the sim directly, so its death was silent before (the walk then failed as a stuck slot hop with health 0); it now asserts by name. Next: the ride-window sniper (the pre-ride #19 box act is d2-only; #19 skin 0x5000087 at (172.0, 19.6, -292.5)).

Main 11-14 (on afec3f768, not shipped): the gantry-ride killer is #15 (skin 0x5000087, 0x1e) on the GROUND at (176.2-178.0, 1.0, -280.5..-281.1): 16.0 at t18156 + 12.6 at t18516. A box act on him from the boarding spot before the ride: the ray dies 0.102 along (the gantry floor), and the rifle is EMPTY there (0+0 - the 2 x 900 south-roof acts spent it); death t18317. The ladder-base rifle ammo (221.5/221.6, 0.5, -232/-233) is refused at the climb (spare already 30) and the detour costs the climb (t7376). Next: rounds for after the roof acts, or a ride-time answer to #15 with another gun.

Main 11-14 (merged d1491c571): the rifle (switched back from the Q-Claw) aims and taps at #15 on clear lines during the gantry ride - #15 dead in 30 taps, ride ends at 26.2, the slot drop lands, death t19289 in the yard at (137.7, 4.6, -296.1) to the SMG trickle (0x500006d #2/#3/#6/#8, 0x500007e #1, 0.6-1.2 a hit, 44 hits for 83.6 total + 16.4 other).
