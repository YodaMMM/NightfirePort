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

Final (2026-10-28): `walk_the_infiltration_to_the_delta_exit` un-ignored and GREEN (70993
ticks, 0 stuck, 0 falls). Three discoveries: (1) the 0x85 doorway seals completely — the
"gap past the leaf" premise was refuted by the crossing probe (the leaves' plane x≈−52.35
blocks both ways; the frame jambs seal z 38.6..38.9); (2) the Kiko scene's scripted body
carries Bond WEST past the closed door (release (−55.49, 1.10, 42.02)) — the test stops
steering at the touch and lets the scene deliver the west side; (3) the Laser left in hand
from the cuts cannot kill Rook (198 bloodless point-blank taps) — the kill switches to the
hand weapon 0x2 first. cut_fusebox now aims at the VISIBLE cable slab with the miss-feedback
(the translation sits 1.5 m low) and picks the group-0 wire. Report:
docs/reports/level-14-delta-exit-walk.md (the switch wall art + the M.Hatch indicator noted
as undrawn, live checks listed).
