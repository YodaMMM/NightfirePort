
## port-map-bullets

Branch `s/port-map-bullets` from `origin/master` (22d5836b7). Task: the function map for
the bullets family — every `Bullet_*`, `Gun*`, `Weapon*`, `Explode_*`, `Effect_*`
function in ACTION.ELF, one line each in docs/port-map/bullets.md: ported with the
port's code location, or not reached with the proof. A comment does not count as ported.
Any function a real level reaches with no port code is a gap: fix it with a test on the
real level, proved red on the old code, each fix in its own commit. Entry started — the
enumeration follows.

Work (same entry): the 32-function table landed; an independent subagent audit of the 8
thinnest rows corrected two (the trail-smoke fn name; the heatseekable overclaim) and the
follow-up record/copter sweeps proved the class-0 arm a REAL gap — fixed in the
PlayerBullet homing block (own commit) with the placement sweeps as the reachability
proof; the live red/green harness is on the branch, ignored until the give draws the
0x2e; the class-3 Bond arm is the slice still missing.
