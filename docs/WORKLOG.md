## level-12-start-damage-gap (2026-11-22, s/level-12-start-damage-gap)

Task: the_idle_start_costs_bond_what_the_games_does (ignored on stream batch 1121d): the port
reads 80.75 health at tick 480 and 56.25 at 840; the game reads 53-72 and 0-37 [L].
start_rounds_07000012_live traces every round: no rolled hit passes the 120-tick NewSighting
gate before t296, and each stand/crouch swap holds fire about 90 ticks. Find which gate or
timer differs from the decompile (NewSighting, ChooseCombatMove 0x150c18, the stand/crouch
swap hold) and fix it with citations; un-ignore the test.

### level-12-start-damage-gap progress

- Setup: the sandbox kept the previous session's repo/ISO/work; re-pointed origin at the task token,
  fetched, created `s/level-12-start-damage-gap` from origin/master (c229c7780), pushed with the
  WORKLOG entry. nf-queue reset to origin/queue. cargo fetch ok (alsa via the user prefix).
- Read the task's [L] frame: the port 82.50@480 (the ignore message's 80.75 was the pre-batch-3
  tree) and 28.25@840 — the 840 side of the assert (<= 40.0) now PASSES on master, the 480 side
  (<= 72.0) still fails. The whole gap is the first 480 ticks (game -28/-47/-32, port -17.5).
- Measured the port chain (probe examples/probe12_sight.rs, scratch): the start pair alerts at
  t169 (the script sound 634 at g7's spot; sound 615 at t109 too far/wrong state), los t170/171,
  ack at 10 sighted ticks (reaction 10 [M 0x14ac60 default arm]), fire states t179/180, first
  shots t180/181. 33 rounds by t480 (17 g6 w0x19 + 16 g7 w0x18), 21 real rolls after the 120-tick
  NewSighting gate (first real roll t294), 9 rolled hits, 6 landed (17.5 lost).
- The 3 unlanded rolled hits (t294/t375/t431, all g6 standing) die as World events on the counter
  edge at (17.4-18.4, 1.3-2.0, ~19.0) between the guards and Bond; the crouched rounds (muzzle
  y 0.84) clear under it and g7's all landed. The aim line from g6's standing muzzle (14.43,
  1.42, 18.83) passes y~1.36 at that x — into the edge.
- Verified the captain assignment is NOT the bug: [D] plc 13 (south) carries prop 9 (the d1
  captain bit [M 0x14b300:504-508]) → the port's south guard is the game's 3.5-per-hit shooter
  [M 0x164210:31-52 swap in Attack 0x56]. The prior task report's A/B ↔ north/south reading was
  crossed with the disc (both guards' prop 0x40 read 0 through bitDescs — my first naive bit
  decode of the placement words was wrong; the game maps props through bitDescs/bitMasks
  [M 0x1c7d98]).
- Landed the [M]-verified AimStandFire ENTER fix: the captain 0x19->0x18 swap [M 0x166328:36-49]
  and the prop-0x40/movement-locked diversion to CombatNoMove 0x59 [M 0x166328:30-49] (the port's
  ENTER had neither; both are no-ops on this level's pair — 0x40 clear [D], the captain swapped in
  Attack already).
- Verified divergences recorded for the next round (not landed): (a) the game's per-round guard
  spread is weapon+0x24 x Drone+0xc0, +0xc0 = the accuracy byte [M Bullet_init 0x12c5a0:139-148,
  DoModeSettingsNEW 0x14c600:108] — the port passes the raw +0x24; (b) the port's north guard
  never re-picks inside AimStandFire 0x68 (zero ChooseCombatMove picks t179-840) while the game's
  north guard hits from 0x70/0x74/0x68/0x6f [L] and ends at (13.9, 17.1-17.5) [L] — the port's
  ask window is the ~1-tick burst_done drain tick, and the probe caught the aim anim ending into
  class 7 (which refuses Ready2Fire [M 0x178c48:47-54]) between bursts (t179-260 class 7, then 2);
  (c) the port's 0x74/0x73 step states hold fire while the game's 0x74 logged hits [L] — the
  game's +0x3b clear at the 0x74 ENTER [M 0x1676a0:24] is re-stamped by the weapon pass, the
  port's `firing = false` blocks until the next 0x68 entry.
- Test state: `the_idle_start_costs_bond_what_the_games_does` stays ignored; the ignore message
  now carries the measured ledger and the next levers. start_rounds_07000012_live and
  the_start_pair_keeps_apart_and_reaches_bond still pass (82.50@480 trace unchanged by the ENTER
  fix, which is a no-op on this pair).
