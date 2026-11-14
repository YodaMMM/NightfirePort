# 10-level-13-warm-chain-margin-7

07000013's real-damage walk (`walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage`)
is still ignored. Master 828704a46 carries level-13-warm-chain-margin-6: the walk dies at t4958 inside the security
room to #9 (skin 0x5000006, weapon 0x10) at the room's east corner (-19.6, -368.4, 25.6); he wakes on movement only,
and the fight is decided by ammo (the in-hand clip of 8 dries mid-fight and the leg's resupply fetch walks Bond back
into his line). Read the ignore message's update-4 ledger and take its next lever (enter the room with a full clip,
or answer #9 from a line that reaches the east corner). Real input only; keep the other walks byte-identical.
Tests: walkreal_07000013_live, interactables_07000013_live. Branch: s/level-13-warm-chain-margin-7.

Measured 11-13 (main): in 12 the 96.5 -> 40.5 loss lands on the walk to the north-lobby line (-8.25,19.0) - #7 (pistol, 3.5 a hit from (4.9,14.2)) plinks while Bond wedges at (-8.8,18.2), then #6 7.0 x2. #7 first from the pre-walk spot (shoot_bounded 900, not exposed) dies in 12 at t13094; from the line spot he is not exposed. Not shipped.

---
claimed-by: sandbox7 at 2026-11-14T03:04Z

---
## Report (done, 2026-11-14T05:15Z)

