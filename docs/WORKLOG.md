
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

## homing-copter-2

Branch `s/homing-copter-2` from `origin/s/port-map-bullets` (70d0fbf04). Task: finish the
arm — (1) the game seeks a copter body only while its obj+0xfc byte is non-zero; trace
what writes it (SP_HideObj, the gate's first ON tick, Copter_Update 0x17dbb8), carry that
state, and never let a hidden or gated copter be a homing candidate; red test for the
dormant case. (2) Un-ignore `homing_copter_07000012_live`: drive the gate channel on the
real level, equip through the path the level allows (pickups #2aa / #2f0), and show a
fired round turning onto the body, red without the arm. Plus an audit of `Bullet_homing`
0x12f358 past the target pick. Entry started.
