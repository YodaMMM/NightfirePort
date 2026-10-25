
## level-14-lid-and-climb

- 2026-10-25T14:20Z sandbox8: branch created from origin/master (c528468b). Task: 07000014 — (1) how the game opens and clears the trench lid [M] (script/SP, key frames, post-open collision — the glist pick vs the script entity's pose), port it so the east-duct walk drops past it, un-ignore `the_east_duct_route_reaches_the_stairwell_slot`; (2) the Q-Claw climb: aim at grapple #626/#657 (45.30/40.78, 4.79, 42.86), fire, reel, E-open the mask-4 lid, onto the mezzanine; rewrite leg 1a of `walk_the_infiltration_to_the_delta_exit` onto the east route and un-ignore when it walks; (3) every ride test stays green; full suite on the final tree. First: read docs/reports/level-14-qclaw-climb.md on origin/s/level-14-qclaw-climb. Downloads + fast suite starting.
