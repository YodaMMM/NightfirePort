
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

## homing-copter-3 (2026-11-07, branch s/homing-copter-3)

Continuing from s/homing-copter-2 (master merged). Building the report's fix: a bullet-clear in
world.rs that excludes by shape hash (the dynamic set's last_objects index-aligned with
geo.objects), fed the candidate's stream hashes (the hidden_stream_objects precedent in
nis.rs), called from the homing's line of sight; guard candidates and the copter's own spot
check [M 0x17dbb8:171-172] audited too. Then the lock test un-ignored and pinned through real
fire.
Round (2026-11-07 ~16:35 UTC): the subject-argument exclusion built and the lock test green.
`segment_bullet_clear_excluding` (world.rs) skips dynamic shapes by hash (last_objects
index-aligned, pointer-identity skip); `stream_hashes_for_placement` (nis.rs) feeds the
candidate copter's SP stream hashes; the homing scan's copter branch uses them. Guard branch
audited: guards are not script entities, nothing to exclude. Spot check [M 0x17dbb8:171-172]
audited: unported, the warrant comment pins the exclusion its slice must carry. The lock test
un-ignored: spawn dot 0.9986 -> best 0.9992, locked; the new wall half: 420 rounds behind an
occluder, none locked; the dormant half still green. Six pre-existing PlayerBullet literals
got the missing target_copter field. 25 affected binaries green; the full suite total line
did not fit (the background runner dies between tool calls; the 4-crate build exceeds the
foreground cap) — the determinism-sensitive set ran green instead.

## hurt-volume-flight (2026-11-07, branch s/hurt-volume-flight)

Branch cut from origin/master (10956badb). Task: the flying hurt volumes on 07000046 —
Hurt_Create 0x17e870 reads the placement's raw +0x40/+0x42 (u16 count, u16 rate) and
+0x44 (path reference) into ext+0x0C/+0x04/+0x08; Hurt_Update 0x17e5d0's state 1 flies the
volume along that path with Spline_Interp, hurting as it goes (docs/reports/
unread-params-props.md on s/unread-params-props: the triple non-zero only on 07000046's
seven hurt volumes, count 6, rate 0x5A, path words ending 000E/000F/0010). Say what the
seven volumes are in play and what starts state 1; plumb the path reference to the level's
path table; port state 1 with the game's rate and end behaviour, other states untouched;
pin through the real level (flies at the game's pace, Bond in the way takes the damage,
clear he does not); run the hurt binaries, the 07000046 binaries, nf-game. Entry started.

Round (2026-11-07 ~19:45 UTC): the flight ported and unit-pinned. Hurt_Create's triple
(count/rate/path = raw +0x40/42/44) plumbed through the loader's m_paths resolution
[0x1d1b68:88-92] to door::parse_keys keys; Hurt_Update's state 1 = door::interp pose per
tick, t += (count−1)/60·(60/rate) [constants 0x30d0dc/0x30d0d0], wrap at count−1, one-shot
retires. The seven 07000046 volumes identified: Icicles, damage 105, who 0x2 = the class-5
BULLET starter (shot down, never touch-started), paths 14..20 = 28.4 m vertical drops.
scene.rs ticks the clock + the bullet-segment starter (drone.rs gained a read-only
bullet_flight_segments accessor). nf-game lib 691 passed (10 hurt units); 106 walkreal
binaries 255 passed / 0 failed. The live pin (hurt_07000046_live) is red-on-old but not
green yet: the test harness cannot drive the fire ray onto the icicle (the fire dir comes
from the camera/aim pipeline, not 1:1 from bond.pitch; the directly-under shot exceeds the
~77° fire-ray cap) — the next session fixes the aim and the tests should go green. Full
suite did not fit (rootfs filled at 100%, target pruned; the background runner dies here).
