# level-14-cascade-servers-4 — the door-spam fix without breaking guard door crossings

Branch from origin/s/level-14-cascade-servers-3 (merge origin/master into it first) and read docs/reports/level-14-cascade-servers-3.md.

Correction to that report: walkreal_07000014_live is GREEN on origin/master. The 14 wedge (Bond stuck at (30.33, 0.55, 35.44) by the open door (32.0, 2.1, 35.8)) comes from this branch's own door_bound change, not from master. Merged onto master the branch turns 11 tests red:
- npc_doors_live::the_06_guard_chasing_bond_opens_the_closed_door
- fix_guard_misc_live::the_06_guard_driven_through_a_door_pair_opens_it_and_crosses
- walkreal_07000003_live: walk_the_full_route_and_complete_every_objective (+ _real_damage)
- walkreal_0700000a_live: bond_stands_at_the_mouth_bridge_edge, walk_office_floor_qworm_keyboard_hatch (+ _real_damage)
- walkreal_07000014_live: the_e_use_switch_fires_the_sabotage_cascade_and_opens_the_west_door, walk_the_infiltration_to_the_delta_exit (+ _real_damage), walk_the_tripbomb_route_pick_up_place_and_shoot_it

So the binding/clear rules stop guards from opening and crossing doors they should. Re-check `door_bound` against [M 0x153358:151-155, 0x1681f8 case 3, DSTATE_OpenDoor's ENTER + NDrone2_DoorIsOpen]: when the game clears +0xBA8 and when a fresh door is bound. Keep no_guard_spams_the_opendoor_after_the_cascade green AND all 11 above green, then every walkreal_*_live, then the full suite total line.

---
claimed-by: sandbox7 at 2026-11-03T13:56Z
