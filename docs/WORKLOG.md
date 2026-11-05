
## level-12-engage-range
Started 2026-11-05 11:08 UTC. Branch s/level-12-engage-range (cut from origin/master f68a72216).
Task: s/level-12-invisible-wall-spots found that after the real alarm on 07000012, drone #6
holds at the corridor mouth with clear sight at 9-10 u and #7 stays in cover, so nobody
reaches the cafeteria. Compare `engage_range` for skins 0x5000008 and 0x5000006 with the
game's Drone+0x1A0 [M] and the combat_state advance rule. If the port holds where the game
advances, fix it. Then test the no-sight advance against the corner at (22.47,36.64): does it
stop short of the game's arrival radius? Run the 12 walks and report them.

Progress 2026-11-05 11:08 UTC: branch cut, WORKLOG started. Sandbox survived (repo+work+ISO
present, target/ 5.2G warm from the previous session; closed out level-1b-platform-touch's
ntfy-failed report section on that branch first).
