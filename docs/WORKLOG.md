# WORKLOG

## level-14-delta-exit-walk (2026-10-28, branch s/level-14-delta-exit-walk)

Task: 07000014 (Countdown Part 1) — un-ignore walkreal_07000014_live
`walk_the_infiltration_to_the_delta_exit`. Leg 2's `follow_nav_route` crosses
the closed west door plane (Door_Slide_11A/B, ch_lock 0x85) before the
sabotage cascade can unlock it. Give legs 2-3 explicit west-side waypoints,
cut both west fuseboxes on the real laser path, walk to the E-use switch
oDroneTrigger1b (-60.30, 0.61, 38.37; touch out 0xA0, mask 4 [M
Trigger_Activate 0x1b2318]) and press E (the green
`the_e_use_switch_fires_the_sabotage_cascade_and_opens_the_west_door` has the
exact input), then legs 4-6 east through the opened door to the Delta exit
#944 on real input; un-ignore (or re-ignore on the next exact blocker with
its evidence). Note anything the player needs that the port does not draw
(the switch wall art, the M.Hatch indicator) in the report. Full suite,
quote the total line.

Plan:
- Fast runner started in the background in the first minutes.
- Reroute legs 2-3 west of the door (the generator room is west; the door is
  crossed only AFTER 0x85 opens it).
- Iterate the walk on real input; then the full suite on the final tree;
  push; report; finish steps.
