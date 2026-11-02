# sniper-range-accuracy-0c-0d — weapon 0x1e guards hit too hard from too far

Branch from origin/master.

The new real-health walks on 0700000c / 0700000d (ignored with these reasons) die to weapon 0x1e drones firing from 34-41 units: on 0d drone #31 (skin 0x5000087 at (160.9, 6.8, -235.1)) lands 14-16 per shot from 41.2 u; on 0c drones like #18 (skin 0x5000090) deal 80 total by tick 8700 from ~34 u.

Do:
1. For weapon 0x1e, pin the game's drone fire at range [M]: the accuracy/spread law with distance (DroneWeap_*), the fire rate and burst, the per-hit damage and any range falloff, the drone stats that scale it (skin's accuracy/difficulty), and the max engagement range.
2. Compare with the port at 34-41 u: hit probability per shot, damage per hit, shots per second. Fix every difference; cite each number.
3. Un-ignore the 0c and 0d real-health walks if they now finish; if a walk still dies with a faithful port, give it cover the level offers (no health writes). Run walkreal_0700000c/0d_live, enemy_accuracy_live, every real-damage walk, then the full suite.
