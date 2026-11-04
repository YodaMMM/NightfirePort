
## level-08-hind-flight-pin
Started 2026-11-04 19:01 UTC. Branch s/level-08-hind-flight-pin (pre-created at origin/master
357255525, no prior commits — continued there). Task: on 07000008 the attacking helicopter must fly
its attack, not hold one spot — log the copter's position/state/path index every 30 ticks on the real
walkreal_07000008_live route from its arrival to its death; compare with Copter_Move 0x17d628 /
Copter_Update 0x17dbb8 (crates/nf-game/src/copter.rs) and the disc paths [M+D]; fix any stall with the
address; pin the travel distance and case order in a real-input test. Every walkreal_*_live; the full
suite total line.
- (19:01) branch checked out, WORKLOG started.
- (19:35) the spot check + case 2 ported [M Copter_Update 0x17dbb8:138-243]: the 45° cone
  (Intersect_ConeSphere 0x1ee100 verbatim; COPT_COS/SIN = PS2Sinf(2.3561945)/PS2Sinf(0.7853981)
  [M 0x17e468:7-8]), the 240/DiffMod spot gate, the 1000-range gate, the LOS 0x2e probe from
  obj+0xC0+dir·4; case 2 = Music_Event(2,1) + the missile/gun pick via the shared attackmode_159
  (new Drones field) + the bail rules (300·DiffMod window, the 100 attack-damage bail, the
  visibility re-check); fire_missile/fire_gun_tick wired with the Bullet_init spawn (the GT
  precedent, no pad draw) and the pods' Mat_GetDir facings (new nis.rs copter_part_matrices).
- (20:00) the flight measured + pinned: walkreal_07000008_live logs the copter every 30 ticks from
  arrival to death on the real route — 4 difficulty runs, travel 123.6/125.9/148.3/168.2 u, NOT ONE
  30-tick flying stretch below 2 u (no stall), cases 1 (with a 2 on the d3 timeline — the spot check
  landed at t307.5, the copter's own round in the air) → death 4 at the ch-0x4B break. 6/6 green.
