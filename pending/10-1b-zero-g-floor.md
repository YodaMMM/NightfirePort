# 1b: the port lands Bond where the game keeps him floating

Branch from `origin/s/aim-console-phase`, merge current `origin/master`, push to `s/1b-zero-g-floor`. Read docs/reports/aim-console-phase.md: the snap is innocent on console 8, and the float wedges at (44.7, -8.66, 1.68) with walk mode pinning Bond, thrusters dead.

Measured in PCSX2 on 0700001b, frame 2251 on: Bond put at (46, y, 0) with velocity zeroed, y from -7.5 down to -8.7 in steps of 0.2. Every one held its height for 0.8 s, substate unchanged, anim+0x60 flags 0: no ground contact and no fall. So there is no zero-g floor boundary at -8.3 / -7.9 as the report assumed. The test ran with no thrust input and did not go below -8.7 or try (44.7, -8.66, 1.68) itself.

1. Treat this as a game-side bug first. Put Bond at those spots in the port and see what switches him to walk mode. Read how the game decides floating against walking on 1b: `Player_ZeroG` 0x192ab0, the zero-g volume or switch it reads, and what ends it. Cite it, and fix the port so a player cannot get stuck there. Test it on real level data, proved red on the old code.
2. Then re-run the 1b walks. If consoles 7 and 8 latch, the two 1b reds are closed by a game fix, not a route lever.
3. Run the `[ring]`/`[gate]` probe on 08, 0c, 14 x3 and 16, one at a time, and say which are snap faults and which are route or raw-aim tuning. 14's three are a nav stick at "the corridor south to the leg-2 spot": look at what stands there.

Keep the [L] 08 pins and the 0x30000000 fix. This branch changes where rounds land, so it may join `s/stream-batch`; say what it does to the walks.
