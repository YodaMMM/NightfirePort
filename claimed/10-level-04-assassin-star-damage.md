# 07000004: the assassin's thrown hits cost 0.75 in the port, 1.25 in the game (difficulty 1)

**Branch:** `s/10-level-04-assassin-star-damage`.

Level 07000004 (Double Cross, the courtyard fight). Each of the assassin's ranged hits on Bond costs 1.25 health in the game and 0.75 in the port at the same difficulty (1) and range (5-6 u, outside TooClose 3.0).

Evidence:
- Game [L PCSX2 07000004 d1, drone 0x1b45420, Bond health BLData+0x894, no input]: 0xae @1327 -> 0xaa @1461 at (1.76, 5.31, 30.09); hits of **1.25** at frames 1474, 1477, 1538, 1605, 1607, 1661 (Bond at the hand-back spot (-3.87, 5.46, 30.30), so dist ~5.6).
- Port (master, after the hand-back fix "Guards woken by a cut-scene's end see Bond where the scene put him"): `load_with_carry_difficulty(iso, "07000004.bin", false, None, 1)`, no input. The ninja (dtype 0xf) has `weapon` 18 (0x12, the Storm M32 record) and `damage_mul` 1.0, and her first ranged hit is **0.750** at tick 1920 (state 0xa9, dist 6.11). The level's difficulty-1 factor is 0.5 [M Player_HandlePain, bullet.rs damage_factor], so the port's raw round is 1.5; the game's raw is 2.5 if the factor is the same.

Find which input differs: (1) the ninja's weapon id in the game (live read of her Drone weapon field; the throws are DroneWeap_Fire(1), maybe a star record rather than 0x12); (2) the record's raw damage weapon_data+0x0C; (3) Drone+0x100 (damage_mul) for mode 0xd from NDrone2_DefaultInit 0x14b300:744-753; (4) the Player_HandlePain factor on 04 at d1. Port the difference and pin it: a hit from her at 5-6 u costs 1.25 at d1.

Also note (not a bug unless proven): in this port run she goes 0xae -> 0xa7 -> 0xaa -> 0xab in one tick (ShortRange's 1/3 ENTER roll), while the game stays in 0xaa for ~200 frames. That is an RNG draw; only chase it if the stream position before it differs from the game.

Run tests: `NF_ISO=<abs path> cargo test -j 8 --release -p nf-view --test assassin_07000004_live` and the 04 walks (`walkreal_07000004_live`, `replay_parity_07000004`). Run the full suite in the background (tools/test/suite.py) before you push and report its total line.

---
claimed-by: sandbox3 at 2026-11-12T16:05Z
