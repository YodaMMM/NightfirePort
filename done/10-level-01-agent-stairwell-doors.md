# level-01-agent-stairwell-doors — the 01 Agent walk with the guards using their door routes

On master (after level-14-cascade-servers-4: one +0xBA8 door slot, OpenDoor ENTER deferral) the 07000001 guards take their door routes: door 2 at (-15.0, 6.3, 14.1) opens around frame 3460 and the SMG guards reach the stairwell head. walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent is ignored for it: Bond arrives at the pre-stairwell clear on ~38.7 health, and the pair #3 (skin 0x5000038) / #12 meet him at 1-2.5 u (TooClose doubling [M Drone_ModBulletDamage 0x139f50]).

Tried and not enough: the sweep at every difficulty (now on master; the Operative walk is green), back_off 6.5 during the sweep, no approach walk. Levers: arrive with more health (the pickups, the hall phase's settle gap), fight the pair from the hall's west side before they reach the head, or catch them at door 2 as they come through. Real input only; no top-ups. Remove the ignore when green; run every walkreal_*_live; full suite total line.

---
claimed-by: sandbox7 at 2026-11-03T22:34Z

---
## Report (done, 2026-11-03T23:52Z)

