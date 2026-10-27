# WORKLOG

One entry per task, appended at the end. Never rewrite or drop another task's entry.

## level-0d-gun-aim-audio

Task: Chain Reaction Part 2 (0700000d) deck-mounted laser gun. Two play defects on top of
s/gunbeam-0d: (1) the beam is off-axis from where the gun points, depending on aim angle —
the beam start/direction must derive from the barrel transform's muzzle and fire direction
[M GunImp_Update 0x1320d8:394-407 PositionBeam(muzzle, ray hit)]; (2) the fire audio stutters
("jittery at first then actually plays") — the TurretGuns entry's +0x18 sound (1307 for the
laser [D]) must be played on the game's cadence, found from the Sound_Play3D / looping-handle
calls in GunImp_Update [M]. Tests: beam origin = drawn muzzle, direction = barrel forward at
several aim angles [M]; fire sound starts once and holds (or repeats) on the game's cadence
while firing.

Status: started 2026-10-27 ~03:50 UTC. Setup: sandbox reset lost the previous clone;
re-cloned, branch `s/level-0d-gun-aim-audio` created from origin/master @ b864d3d6.
