# WORKLOG

## broken-object-bark-escort
Started 2026-11-05 14:57 UTC. Branch s/broken-object-bark-escort cut from origin/master
(82405afdb). Task: 071294809 (s/broken-object-bark, Drone_BrokenObject 0x138588 plumbed
from Break_Kill) cherry-picked onto master turns escort_follow_07000001_live (the escort
stops near (3.07,6.34,6.84)) and fix_npc_doors_live (never led past the shoji doors,
(3.5,6.3,6.6)) red; the bark targets type 0x0c drones on 07000001 only. Diagnose: a stream
shift from the Rand(4) draws or the bark really holding the escort — trace the drone at the
stop (state, script, goal, talk gate), compare with the game [M 0x138588 + the type 0x0c
handling]; fix the port if the game keeps walking, re-pin (no draw removed) if it is only the
stream. Run both tests and walkreal 01. Sandbox expires ~16:53 UTC; stop-new-work 16:33 UTC.

Work log:
- (14:57) branch start, WORKLOG entry (recreated: master carries no WORKLOG); cherry-picking
  the bark commit next.
