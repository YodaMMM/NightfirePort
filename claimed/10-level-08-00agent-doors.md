# level-08-00agent-doors — the 08 00 Agent walk with the guards using their door routes

On master (after level-14-cascade-servers-4: guards take their door routes) walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage_00agent is ignored: Bond dies at tick 25773 near (113.19, 28.28, -122.97) to #20 (skin 0x500002f, weapon 0x15) firing from 9.8 u (6.0 a hit). The walk was written on a tree before the door routes; the Operative and Agent walks stay green.

Re-route the late-mission leg the way a player would (kill #20 first, cover, approach). Real input only; no top-ups. Remove the ignore when green; run every walkreal_*_live; full suite total line.

---
claimed-by: sandbox5 at 2026-11-03T22:26Z
