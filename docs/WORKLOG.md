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
- (15:00) the cherry-pick d9095e28b (071294809) landed clean; both named reds reproduced
  (escort_follow 0/1, fix_npc_doors 0/1).
- (15:12) the DIAGNOSIS: STREAM SHIFT, not a talk hold. The bark-trace (added to the escort
  test's stall dump): the lead parked at (3.1, 6.3, 6.8) in AllyLeadWait 0x20 with talks 0 /
  voice None at his own route node (goal (2.91,5.74,7.57)); two bark lines had landed and
  FINISHED (last_voice 861, 3390). Game side [M]: Drone_BrokenObject 0x138588's speech is
  NDrone2_SpeechSFX 0x141b00 — sound + the +0x4d0 handle, no state write, no walk stop; the
  port's speech() matches. The park is the game's own wait law; BOND was the stuck one — the
  shifted follow steering wedged him on the meet hall's west wall (x≈1.93, z 11.1-13.1 [D])
  west of the hall, and door #1 (7.9,5.3,11.1) auto-closed between the pair.
- (15:20) the walk-side fix (no draw touched): the walkreal 01 north-lane recovery
  ([D nav path 0 nodes 8/23/7]) + the E-open of the closed door hoisted into the
  ALLY_LEAD_WAIT branch (it shadowed the closed-door branch). escort 1/0 (10.65 s, the
  contact led to (−28.3,14.3)), npc_doors 1/0 (7.94 s).
- (15:32) the sweep: broken_object_bark_live 4/0, walkreal_07000001_live 8/0 whole,
  mp_death_hands_online 2/0, escort_cover 3/0, escort_hide_findrun 3/0, fix_guard_misc 6/0,
  ally_stuck_sweep 1/0, ai_traces 4/0, campaign_flow 5/0, nf-game lib 678/0+5i. Other
  levels' streams are untouched (the bark walk is level-gated to 0x07000001 [M the
  cherry-pick's gate]).
- (15:35) the report.
