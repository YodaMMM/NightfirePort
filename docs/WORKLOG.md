## level-0a-band-settle-6-b (2026-10-27)
Branch s/level-0a-band-settle-6-b from origin/s/level-0a-band-settle-5 (tree = the settle-3
state), merged origin/s/level-0a-band-settle-6 (the plan-only session) and origin/master.
Task: bisect settle-5's five radial-tail deltas against walkreal_07000001_live's plate walk
(one env flag per delta), decide the 01-mover from the decompile [M 0x109da0 /
0x10a53c-0x10a5e4], re-derive the 01 plate pins from the game's behaviour if the skip is
real; pin the faces/ticks with the [L 2026-10-26] jamb push table; un-ignore the three
walkreal_0700000a tests, keep every walkreal test green.

## level-0a-band-settle-6-b (2026-10-27, the bisect verdict)
Bisected settle-5's five radial-tail deltas against walkreal_07000001_live's plate walk (one
env flag each): only D3 (the Q2 re-run) broke 01 — the mis-encoding was the caller's
flags-99/return gate [M 0x10a510/0x10a528] not covering the SECOND Dist2Tri call; with the
re-run's behind-plane query leaving the face inert, 01 is green. Landed the settle-5 radial
tail unconditionally (D1 radial-arm removal, D3+gate, D4 dot gate, D5 |v|==0→0); held D2 (the
first-query flags-99 skip, [M]-faithful) env-gated — its landing wedges 01's resupply pocket at
(5.48,6.67,11.60). The glide test's route = the [L] through-line; the doorway slide reproduces
the game's rate (155 ticks to (-9.27,1.12,16.60) [L 2026-10-26]). 01 battery 3/0, 0a 5/0+3i,
nf-game lib 578/0. The trio stays ignored (the y-settle climb; the corner float) — the report
names each test's one remaining mechanism.

## level-0a-band-settle-6-c (2026-10-27)
Branch s/level-0a-band-settle-6-c from origin/s/level-0a-band-settle-6-b, merged origin/master.
Task: the full suite broke 21 tests across many levels (Bond settles ~1 unit high on the
radial-tail landing, e.g. interactables_07000015 south walkway y 41.315 vs floor ~40.35) —
find which part of the change applies outside the case the game gates it to
[M Player_FeetOnPoint 0x194188 and the walk's vertical], restrict it to exactly that, get the
whole suite green with the 0a walk tests. Work in progress.
