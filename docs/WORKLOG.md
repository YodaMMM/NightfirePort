
## level-0a-ring-sweep-timing (started 2026-11-04 09:16 UTC)

Branch s/level-0a-ring-sweep-timing cut from origin/s/trigger-plane-records-2 (9c5406bc), origin/master
(dd78d755) merged. Four tests red, one cause: walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch
and its three real-damage sisters — the laserbeam glists ride key motion (#614 sweeps between z 13.17 and
10.73) and the walk's jump lands in the swept beam, so ch 0x22 fires; a player times the crossing against the
sweep. Task: re-time the ring's crossings with real input (sweep-aware press windows from this branch's
"sweep-watch route experiments" history, or pin the beams' key paths [D] and cross in the measured gaps).
Never weaken the trigger law or the "no laser/alert channel fired" pin. Done = the four green,
low_beam_621_fires_standing, fix_truck_live, interactables_07000011_live green, every trigger/beam/laser test
and every walkreal_*_live green, then the full suite total line.
