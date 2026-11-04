# level-0a-ring-sweep-timing — time the 0a office-ring walk against the sweeping beams

Branch from origin/s/trigger-plane-records-2 (merge origin/master first). That branch lands the game's trigger contact: low_beam_621_fires_standing, fix_truck_live 2/2 and interactables_07000011_live 16/16 are green. Read docs/reports/trigger-plane-records.md on origin/s/trigger-plane-records ("The ring mechanism").

Four tests stay red, all one cause: walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch and its three real-damage sisters. The laserbeam glists ride key motion (#614 sweeps between z 13.17 and 10.73) and the walk's jump lands in the swept beam, so ch 0x22 fires. A player times the crossing against the sweep. Re-time the ring's crossings with real input: the sweep-aware press windows are in that branch's history ("the sweep-watch route experiments"), or pin the beams' key paths [D] and cross in the measured gaps. Never weaken the trigger law or the "no laser/alert channel fired" pin.

Done = those four green, low_beam_621_fires_standing, fix_truck_live and interactables_07000011_live still green, every trigger/beam/laser test and every walkreal_*_live green, then the full suite total line. List each binary with its count.

---
claimed-by: sandbox6 at 2026-11-04T07:58Z
