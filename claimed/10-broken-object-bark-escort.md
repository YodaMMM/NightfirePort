# broken-object-bark-escort — does the bark stop the escort?

Commit 071294809 on s/broken-object-bark, cherry-picked onto master, turns two tests red: escort_follow_07000001_live (the escort never reached the plate, stops near (3.07,6.34,6.84)) and fix_npc_doors_live (never led past the shoji doors, settled at (3.5,6.3,6.6)). The bark targets type 0x0c drones on 07000001 only. Find out which: a stream shift from the Rand(4) draws, or the bark really holding the escort. Trace him at the stop: state, script, goal, whether a talk gate or line holds him, then compare with the game [M 0x138588 and the type 0x0c handling]. If the game keeps walking, fix the port. If it is only the stream, re-pin without removing a draw. Run both tests and walkreal 01.

---
claimed-by: sandbox3 at 2026-11-05T14:40Z
