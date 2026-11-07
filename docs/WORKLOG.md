
## port-map-bullets

Branch `s/port-map-bullets` from `origin/master` (22d5836b7). Task: the function map for
the bullets family — every `Bullet_*`, `Gun*`, `Weapon*`, `Explode_*`, `Effect_*`
function in ACTION.ELF, one line each in docs/port-map/bullets.md: ported with the
port's code location, or not reached with the proof. A comment does not count as ported.
Any function a real level reaches with no port code is a gap: fix it with a test on the
real level, proved red on the old code, each fix in its own commit. Entry started — the
enumeration follows.
