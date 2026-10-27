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

## level-0a-band-settle-7 (started 2026-10-27 ~06:25 UTC)

Task: port the feet-band walked-dy growth (the hall rest 1.1285 [L 2026-10-26] vs the port's
1.1201) from the game's ground/step rule [M Player_FeetOnPoint 0x194188, the walk's vertical],
un-ignore walkreal_0700000a_live::the_west_hall_glide_height_matches_the_game, keep every
walkreal green.
- Branch from origin/s/level-0a-band-settle-6-c (= 6-b + master) + merge origin/master
  (1d2651c0) = a64264ba.
- MECHANISM BOUNDS pinned this round:
  - [M Player_SSWalk 0x192310 = Player_Move + AnimSetUpdate + HandleJump + the matrix rebuild;
    Player_Move 0x191dd0 writes only the smoothed locals + turn] — the walk substate carries NO
    vertical writer; a flat walk is y-neutral in the game.
  - [M Player_Init 0x1910f8:40 sets obj+0xD4 = BL+0x740] — Collide_Update's per-tick
    Vec_Zero(obj+0xD4+0x10) clears the +0x750 accumulator every tick: the handler's
    obj+0x30 += BL+0x750 [M 0x1949a8:136-140] adds THIS TICK's DeltaP — the port's
    `pos += push` model is right; the lifetime-accumulator reading is refuted.
  - [M the handler's grounded zero-skip is jump-gated: BL+0x94c = Player_HandleJump's jump
    state (0 none / 1 rising / 2 falling, 0x1916b0:181/:69/:193)] — the FeetOnPoint hitdata
    copy's BL+0x14 (normal.y) is zeroed on every grounded non-jump tick; it is not the lift.
  - [D band_settle_probe] the whole glide route's floors are FLAT y≈2e-5 (the spawn, the
    lobby, the doorway, the hall) — no floor step anywhere.
  - [L-port band_settle_probe] the port's flat approach does NOT climb (1.0891 rock-steady
    13 ticks); the game's flat approach CLIMBS +0.02 in the f0-5 window before the press
    [L band-settle-2 brief: placed 1.12 → 1.1402 @f5]. The walked-dy growth is REAL and on
    the flat walk; its writer is not statically identifiable in this window's read
    (Player_MonitorAir 0x191a70's `surfaced` = the dry-land state, so the FeetOnPoint hitdata
    copy runs on land, but the grounded arm zeroes the move slots it overlaps).
  - [L-port NF_CAPS_LOG] the press-y residue: the port's jamb press (root x −7.754 — the same
    standoff the game's f5/f13 read) climbs +0.0025..0.0106/tick off the big edge face
    (cent (−8.304,1.094,15.601)) to peak 1.1720 and settles 1.1130; the game peaks 1.1465 and
    settles 1.1285 [L]. The press's radial-y is the port/game divergence to re-derive next.
- The test's ignore note updated with these bounds; the test STAYS ignored (the pin 1.1285
  [L] is not met: the port reads 1.1201). No src changes landed this round — the walkreal
  batteries are code-identical to the 6-c base.
