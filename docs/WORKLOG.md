
## level-08-hind-flight-pin
Started 2026-11-04 19:01 UTC. Branch s/level-08-hind-flight-pin (pre-created at origin/master
357255525, no prior commits — continued there). Task: on 07000008 the attacking helicopter must fly
its attack, not hold one spot — log the copter's position/state/path index every 30 ticks on the real
walkreal_07000008_live route from its arrival to its death; compare with Copter_Move 0x17d628 /
Copter_Update 0x17dbb8 (crates/nf-game/src/copter.rs) and the disc paths [M+D]; fix any stall with the
address; pin the travel distance and case order in a real-input test. Every walkreal_*_live; the full
suite total line.
- (19:01) branch checked out, WORKLOG started.
