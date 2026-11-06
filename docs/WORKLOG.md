# WORKLOG

## level-gates-drones
Started 2026-11-06 13:29 UTC. Branch s/level-gates-drones (from origin/master 8e72343f8).
Task: the Drone gate sweep — for every `Drone*`, `NDrone2_*`, `DroneFunc_*`, `DroneVision_*`
function except the talk functions: read the decompile, trace what it does, find the port's
code, report PORTED (with the port location) or MISSING one line each. A comment does not
count as ported; look for the code. Fix each MISSING one with a test on the real level,
proved red on the old code. No hand-built geometry.

2026-11-06 (2nd session). The sweep table landed: docs/port-map/drones.md, one line per
function — 587 ported, 30 not reached (DroneModeSettings rows + proofs), 0 missing. The
address sweep re-run independently (word-boundary hex over crates/*/src, then fold names;
test-only citations rejected) corroborates the 1st session's read of every function: the
only reached-without-port-code behaviour in the family was the Zoe escort monitor, fixed
last session. Checkpoint pushed before the test pass.
