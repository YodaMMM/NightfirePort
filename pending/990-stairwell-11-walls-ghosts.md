## Your task: `stairwell-11-walls-ghosts`, the 07000011 stairwell guards: invisible walls, and guards that are not there

User playtest 2026-10-22 (check stairs11m): "still broken as fuck. bright entities that arent actually there and guards hitting invisible walls". F12s:
- dist/nf-shots/011-07000011.png (`--bond 14.11,34.03,0.26,2.305,-0.395`, frame 921) and 012 (`15.35,35.42,1.61`, frame 1258): two brightly lit guards in a T-pose, rifles held sideways, standing in a doorway.
- 013 (frame 3688): another lit guard standing in the air outside the railing.

1. **The ghosts:** list every drone on 07000011 at those frames: present/enabled/hidden flags, state, anim, position, and whether it is drawn. Are these drones the game keeps hidden or has not spawned yet (abseil/elevator/reinforcement spawns parked at their placement), which the port draws anyway? Port the game's hide/visibility rule for them [M]. The main agent judges the look; your job is which entities should not be drawn or should be posed, with evidence.
2. **The invisible walls:** find where the stairwell guards stop or slide against nothing (AI navigation vs collision). Print the collision hit (triangle, object, cel) they bump. Is it a static object the game does not collide with (cel flags, as in 949's Collide_Pick 0x20 rule) or a nav-path problem? Fix it with evidence.
3. Real-path tests for both, each with a sister.
