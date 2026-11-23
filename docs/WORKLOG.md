## retune-nb-08 (2026-11-18, s/retune-nb-08)

Task: commit 3412a89a3 made every guard run the game's spacing scan and the close-range alt
attack from the fire states; the 08 whole-mission walks
`walk_the_whole_mission_on_real_input_real_damage_agent` and `_00agent` now die. Read both
ignores, find where the new fights cost the health, and change the route, stance or fight
order there. Helpers stay gated to the level and the difficulty. Real input only, no top-ups,
never lower damage or health, never change guard code. Judge each lever on its own real run.
Remove each ignore when green; if still red, push the measured ledger and the next lever into
the ignore message. Run the whole walkreal_07000008_live binary.

### retune-nb-08 progress

- Reproduced both ignored deaths [L runs, this branch]: Agent d2 t31483 at (112.72,13.49,-177.24),
  4 rockets landed (401.8 of 448.5), dead at the 5th launch — the ledger: 59 machine drips
  (-1.20/-1.40) for 72.6 world damage + the killing blast, and the upstream guard cost only
  ~15.4 (drone #2/#9/#5/#10). 00 Agent d3 t28298 at (112.25,13.09,-179.16): 32.2 in 38 world
  hits (gun chips -2.00 x5 + the -7.78/-8.22 missile blasts), ONE rocket launched (t1269), the
  upstream y-46 hall trio (#14/#11/#2) + secure-room #5 cost 48.0.
- Measured the root shift: the nb-scan-all upstream walk is ~6500 ticks faster (the old green
  fight started ~t35500; now ~t25700-28900 [L the runs — rockets 1/2 identical t49/t229/t275/t428
  across all runs]), so the machine's route phase differs and its passes line up with the
  doorway lane on d2 and fly HIGH overhead (y 25-29) on d3.
- Lever 1 (both sisters, difficulty >= 2): the dodge SIDE bias and the drift walk-back open from
  d3 to >= 2 (the d<2 else branches unchanged — the d1 real-damage Operative timeline byte-exact
  by construction; its test passed in the whole-binary run).
- Lever 2 (Agent, difficulty 2): the fight opens CROUCHED at the slit anchor (110.98, -179.31);
  the movement inputs stand Bond back up into the lintel-window launch geometry (the green run's
  launches leave from the standing eye y 14.22). GREEN: five rockets, 502.5 damage vs hp 448.5,
  best single 101.0, end health 79.0 armour 24.1, 14 hits for 21.0, mission 31070 ticks — the
  ignore REMOVED.
- Levers tried and retired on d3 (each on its own real run, the code reverted): the crouched
  slit anchor (t29087, ZERO launches — the crouched eye's sight stops on the lintel t0.106,
  clip 4 intact), the standing slit + the aim gates relaxed to 0.25 (t27383). The d3 fight code
  is back to its original measured state (death t28298 reproduces exactly [L the whole-binary
  run]); the full ledger + the next lever live in its ignore message.
- Whole walkreal_07000008_live: 6 passed / 0 failed / 1 ignored (the d3 ledger), 153.6 s.
  Only this test file and docs/ changed; no crates/*/src/ and no tests/common/ files touched.
## retune-nb-08-5 (2026-11-19, s/retune-nb-08-5)
- Round 5 of the 08 whole-mission 00-Agent real-damage walk. Branched from origin/master
  (origin/s/retune-nb-08-4 does not exist on the remote; master already carries the round-4
  shipped tree per its ignore-message ledger). Round-4 remaining bug: the d3 cabin fight runs
  CROUCHED on the lower floor (eye 13.3 blindfolded by the exterior ledge; the steep rising
  rays self-blast). This round: stand Bond up on the lower floor before the launch gates.
- Round 5 result: still red at the machine fight, master's shipped run-10 state kept (death
  t21070, 55 hits — the byte-exact control verified). The round-5 ledger: (a) the run-11
  "crouch latch" mystery SOLVED — it is the stand/crouch LOCKOUT, BL+0x960 = 60 armed on
  every substate change [M Player_ChangeSubState 0x190630:48; SSCrouch 0x1924c0:45-105];
  the stand press after waiting the lockout out FIRES (crouched false, lockout 59) but the
  standing capsule re-bridges the hole (feet 13.47) and the launch self-blasts earlier
  (t20944/t20942) — TRIED and DEAD. (b) The pickup scan [L]: armour 0.0 health 14.0, rocket
  clip 4 carried 0 vs the machine's 598 hp — UNWINNABLE on carried ammo; the cable car's
  three rocket pickups (launcher + 2 boxes, 4 each [D 0x0600004C streams 37-39]) are UN-GONE
  1.2-2.4 u away — 12 rockets win. (c) Collection walks TRIED and DEAD (both routes climb the
  carriage's exterior into the hurt volume, deaths t21016/t21286). Next lever (in the ignore
  message): collect the pickups DURING the ride (they are parented to the carriage and ride
  with Bond), then the idle-wait sweep before the y-46 trio/#16 (the upstream bleed 84.0
  leaves 14.0 hp at fight entry). Only crates/nf-view/tests/walkreal_07000008_live.rs and
  docs/ changed; no crates/*/src/, no tests/common/.
## retune-nb-08-6 (2026-11-19, s/retune-nb-08-6)
- Round 6 of the 08 whole-mission 00-Agent real-damage walk. Branched from origin/master
  (SniperAim ENTER landed since round 5), merged origin/s/retune-nb-08-5 in. First:
  re-measure the death on the merged tree, then take the next untried lever from the
  ignore message (skip measured-dead levers): collect the cable-car rocket pickups DURING
  the ride, then the idle-wait sweep before the y-46 trio/#16.
- Round 6 result: still red, but the walk now dies t21773 (was t21070) — 700 ticks deeper, in
  the RIM fight (feet 13.47, eye 14.18). The re-measure on the merged tree (SniperAim ENTER)
  reproduced master's run-10 ledger exactly (t21070) — the guard change did not move this walk.
  Ledger corrections: the round-5 "carried 0 / unwinnable" was a wrong-index read (the rocket
  pool is ammo type 0xe, carried 4 — the fight already has 8 rockets); one cable-car ammo box
  ALREADY auto-takes at ride start (the boarding spot is inside the take radius) and the cap
  (ammo_data type 0xe = 4) refuses the other two forever — round-5's ride-collection lever
  measured DEAD. The lower-floor stand-off lever measured DEAD: the standing capsule pops to
  13.470367 at every lower-plate spot ((112.8,-179.15), (113.08,-179.02) — the crouched walk
  holds feet 13.0856 east past x 113, the trench is narrower than the standing capsule). The
  shipped tree (ride walk + stand-off) fights from the rim: nearly zap-free (one -1.73 row in
  ~800 ticks), launches FIRE (t377/t585, both miss at the mid passes y 18.4/20.8) — and the
  closer is the machine's MISSILE salvo (the -15.00/-3.27 blast lands at the same frame with
  and without a launch [L the 14.5-band no-launch run]). Next levers: answer the salvo at the
  rim (dodge arm / car-body cover), the upstream idle-wait sweep (the salvo takes 18.3; the
  tree entered at 20.0 vs master's 14.0), then the low-band launches. Whole binary 6/0/1,
  151.65 s. Only crates/nf-view/tests/walkreal_07000008_live.rs and docs/ changed.

## retune-nb-08-7 (2026-11-20, s/retune-nb-08-7)

Task: round 7 of the 08 whole-mission 00 Agent real-damage walk. Branch from origin/master,
merge origin/s/retune-nb-08-6 in first (done: merged cleanly onto the 08-6 tree). Re-read the
08-6 report and the walk's ignore message; skip every lever the ledger marks measured dead;
take the next untried one from the ignore message. Re-measure the death on the merged tree
(master moved SniperAim ENTER). Helpers stay gated to level+difficulty; real input only, no
top-ups, never lower damage or health, never change guard code. Run the whole
walkreal_07000008_live binary.

### retune-nb-08-7 progress

- Round 7: re-measured t21773 exact on the merged tree; the per-10-tick bullet diagnostic split
  the death in two: the killer missile (w62, kind-98 radius 4.0, straight 1.0 u/tick) closed
  30 -> 0.3 u while Bond RODE his own t585 rocket (the missile cam zeroes the whole input [M
  scene.rs; Player_Update 0x193600 case 10] — the dodge is gated off during the ride), and the
  aimed strafe wedges at the rim because Player_Move zeroes the forward axis while aiming [M
  player.rs] — the same law made the walk-back's forward-1.0 a no-op forever. Shipped (all
  d3-gated, sisters byte-exact): the aim-releasing diagonal dodge with the per-salvo latched
  side and the miss<2.0 drive cap (a pass at miss >= 1.6 wall-detonates for 0.0 to -4.8 instead
  of the contact -15.00), the walk-back pin while any w62 round is live, and the aim-releasing
  full walk-back (0.35 u). Launch band -> the proven landing top cp[1] <= 13.7. The walk now
  survives two salvos and reaches fight t~1500-2110 (tick 22649-23246, was 21773); the first
  low-band launch fired at t2041 (missed — a state-2 fast pass). The wall is the entry health:
  the dance costs ~4-9 + zaps per salvo from 20.0 hp; next lever = the idle-wait sweep before
  the y-46 trio / #16 (+10-20 entry hp), then the wall-exit-aware dodge side and the
  slow-pass launch gate. Only the walk file and docs changed; whole binary 6/0/1, 107.63 s.

## retune-nb-08-8 (2026-11-20)
Round 8 of the 08 whole-mission 00 Agent real-damage walk. Branch s/retune-nb-08-8 from
origin/master with origin/s/retune-nb-08-7 merged (walk test + reports only, no src changes).
Plan: read the 08-7 ledger and the ignore message on
walk_the_whole_mission_on_real_input_real_damage_00agent, skip levers marked measured dead,
try the next untried lever on its own real run; Operative-side rules unchanged (real input
only, no top-ups, never lower damage or health, no guard-code changes, helpers stay gated to
level 08 + difficulty). Status: setup; levers not started yet.
  Round 8 result: the baseline re-measure reproduced the shipped state exactly (death t22649,
  fight entry armour 5.0 health 20.0) and the NF_DMG_TRACE run attributed the whole 80.0
  upstream bleed for the first time: the y-46 trio 36.0 at the wedged nav pin (149.6, 46.1,
  −93.1) during the dry-pistol return-fire stands, #16 28.0 on the pit-excursion descent legs,
  #9/#10/#4 16.0 later, world 20.0 in the Hind fight. Ledger lever (1) the idle-wait sweep was
  executed as the crouch sweep on the two exposed legs (the shipped hall-crossing/briefcase
  precedent): BOTH halves measured DEAD on their own real runs — the crouched deck nav wedged
  the walk at the closed hall door for the 6000-tick cap (zero progress), the crouched
  excursion descent dropped Bond's eye into the pit pair's line (death t6918 on the ramp, was
  t22649) — both reverted with dead-lever notes at the sites; the ignore message carries the
  full round-8 ledger and the next levers (the crossfire geometry, the pre-raised descent, the
  armour-order swap). No src changes; whole walkreal_07000008_live binary 6 passed / 0 failed /
  1 ignored, 152.57 s on the final tree.

## retune-nb-08-9 (2026-11-20)
Round 9 of the 08 whole-mission 00 Agent real-damage walk. Branch s/retune-nb-08-9 from
origin/master with origin/s/retune-nb-08-8 merged. Plan: read the 08-8 report and the ignore
message on walk_the_whole_mission_on_real_input_real_damage_00agent, skip levers the ledger
marks measured dead, try the next untried lever on its own real run; rules unchanged (real
input only, no top-ups, never lower damage or health, no guard-code changes, helpers gated to
level 08 + difficulty). Status: three levers judged on their own real runs. (1) the deck stand pull west 2.5 u —
  DEAD (the trio's lanes cover the whole deck line; deck-head fight armed 12.0 was ~52.0,
  death t7313 was t22649), reverted with the note at the site. (2) the aim press before the
  excursion legs — DEAD (the hip cone fires during the raise [M Player_SetWeaponAnimObj case
  0xf]; the holds wobbled the rolls, death t18365 to #26 point-blank), reverted. SHIPPED: the
  pit fight's legs under level_fights (the existing ±2.0-y opt-in pick filter, this one call,
  d3-gated — sisters byte-exact): the blind below-level stands never open, #16 28.0/5 -> 8.0/1,
  mid-game health 44.0 was 20.0, death t26803 was t22649 (4154 ticks deeper; the boarding
  chunk drains only the armour, and the +24.0 gain is re-spent on the pre-fight walk-down
  zaps (12 x -2.00 at (111.7,-179.2) t25666-25793) before the crouch drop, so the fight
  entry health is the same 20.0; the fight runs ~600 ticks from it). Whole
  walkreal_07000008_live binary 6 passed / 0 failed / 1 ignored, 151.76 s on the final tree;
  the full suite.py did not fit (the background runner killed by the sandbox between tool
  calls, the ~200-binary build does not fit the window); nf-game --lib 796/0, nf-formats 98/0
  except the allowed 3 slim-ISO disc_golden failures, nf-audio 35/0. Report:
  docs/reports/retune-nb-08-9.md.

## retune-nb-08-10 (2026-11-20)
Round 10 of the 08 whole-mission 00 Agent real-damage walk. Branch s/retune-nb-08-10 from
origin/master with origin/s/retune-nb-08-9 merged (08-9 was not on master). Plan: read the
08-9 report and the ignore message on walk_the_whole_mission_on_real_input_real_damage_00agent,
skip levers the ledger marks measured dead, try the next untried lever on its own real run;
rules unchanged (real input only, no top-ups, never lower damage or health, no guard-code
changes, helpers gated to level 08 + difficulty). Status: two levers shipped, each judged on
  its own real run. (1) the pre-fight walk-down zaps — the whole d3 drop sequence (the anchor
  walk_to + the drop press + the crouch drop + the lockout + the stand-off walk + the stand
  press) moved ABOVE the pane shoot-out, right after the bottom-of-mountain cinematic: the
  baseline's zaps' anatomy is 8 during the shoot-out (the volume arms with the machine's approach
  ~106 ticks after wait_enabled returns, first zap t25666) + 4 during the anchor walk; the drop
  (~65 ticks) now lands the zap-free plate BEFORE the arming and the shoot-out runs from the rim —
  the pre-fight zaps drop 12 → 1, the fight entry returns to health 41.5 (was 20.0), death t27636
  (was t26803, 833 ticks deeper). (2) the d3 fight anchor moves to the shipped stand spot
  (113.08, −179.02): the lever-1 run measured the old anchor (112.5-112.6, −179.39) at rim height
  zapping the fight away (13 −2.00 rows in the last ~830 fight ticks); the stand spot measured ONE
  row in ~430 ticks and the round-6 launches fired from it — the lever-2 run: the anchor zaps
  gone (Bond held 13.6 health for 981 fight ticks), the fight reaches ~2000 ticks (the round-7
  low-band window), death t28256 (1453 ticks deeper than the shipped tree). Still red — ZERO
  launches in both lever runs (line_ok FALSE at all 105 samples, the machine's hp 598 untouched):
  the machine's route phase at fight t0 flew the whole window on the southeast far legs and the
  high overhead passes, whose bearings exit through the south-east corner post / the lintel; the
  proven launchable window (the machine's LOW approach dive, frames ~25620-26320) ran UNDER the
  shoot-out + the moved drop block. The next levers are in the ignore message. Whole binary
  6 passed / 0 failed / 1 ignored, 111.74 s [L the final-tree run]; the affected-binary set is
  walkreal_07000008_live alone (no src/ or common/ changes). Two eprintln-only diagnostics also
  shipped (the timeline byte-exact): the approach-flight dump (the machine's launchable dive is
  MEASURED at f25710-25800, y 12.84-13.61, dist ~49 SW — the fight loop opens f26248, ~450 ticks
  late) and the tail stamps (the drop block f24872-25309, 437 ticks with ~140 wedged in the east
  walk; the shoot-out f25309-26196, 887 ticks for one pane round). Report: docs/reports/retune-nb-08-10.md.

## retune-nb-08-11 (2026-11-20)
Round 11 of the 08 whole-mission 00 Agent real-damage walk. Branch s/retune-nb-08-11 from
origin/master (ccd79b5be) with origin/s/retune-nb-08-10 merged in. Setup done; reading the
round-10 report and the walk's ignore message next, then evaluating the next untried lever on
its own real run. Report: docs/reports/retune-nb-08-11.md.

## retune-nb-08-11 (2026-11-20)
Round 11 of the 08 whole-mission 00 Agent real-damage walk. Branch s/retune-nb-08-11 from
origin/master (ccd79b5be) with origin/s/retune-nb-08-10 merged in. The tail compression SHIPPED:
the shoot-out's 887 ticks were arm_fullest's 12-cycle scan (762 — the direct inventory pick is
the same 0xa/35 in 5 presses, tail-only; the mid-game calls keep the shipped scan after the
first lever run's t24113 wobble) + the blind 90-tick reload wait + the east walk's 142 wedged
ticks (the wedge break parks byte-identical). The shoot-out ends f25713 (was 26196), the fight
loop opens ~25714 INSIDE the measured dive (f25710-25830), and rocket 1 LAUNCHED at fight t36
dist 49.9 — the first dive-window launch of the retune. The dive-window fight dies t25822: 10
dense approach zaps (−20 of the 44.0 entry) + the machine's reply missile ON the west-band
corner d 3.4 (−26.00) MID-RIDE (the camera-locked ride cannot dodge; the d3 dodge miss band
2.0→4.0 tried, no effect on this death). Still red — the refreshed ledger and the reply-line
geometry lever (the launch chest clears the doorway gap only at z ≥ −178.35; the anchor is
0.67 short) are in the ignore message. Whole binary 6 passed / 0 failed / 1 ignored, 108.67 s
[L the final-tree run]; the affected-binary set is walkreal_07000008_live alone (no src/ or
common/ change). Report: docs/reports/retune-nb-08-11.md.

## retune-nb-08-12 (2026-11-20)
Round 12 of the 08 whole-mission 00 Agent real-damage walk. Branch s/retune-nb-08-12 from
origin/master (ccd79b5be) with origin/s/retune-nb-08-11 merged in (08-11 was not on master).
Setup done; reading the round-11 report and the walk's ignore message next, then evaluating
the next untried lever on its own real run. Report: docs/reports/retune-nb-08-12.md.

## retune-nb-08-12 (2026-11-20)
Round 12 of the 08 whole-mission 00 Agent real-damage walk. Branch s/retune-nb-08-12 from
origin/master (ccd79b5be) with origin/s/retune-nb-08-11 merged in (08-11 was not on master).
The reply-line lever landed as the SE pane-hole anchor (113.3, -179.3) + a d3 pre-fight anchor
walk (the lever runs measured the fight's walk-back releasing the AIM and walking THROUGH the
dive's launch window — the fine [08 gates] dump names every gate): rocket 1 launched mid-dive
(fight t49) and LANDED -101.0 — the retune's first dive-window machine damage (round 11 died
0.0/598) — and the round-11 -26.00 reply wall is down (the reply line crosses the west band
through pane 2's shot-out centre, zero reply-blast rows). Still red: the hurt-volume zaps
drained the whole 44.0 entry (22 x -2.00 rows; the standing capsule re-bridges onto the hole's
rim at every spot on this latitude incl. the park), death t26276 (frame 26278), 454 ticks
deeper, machine at 497/598. Tried and dead on their own runs: the north strip (launch line
cannot open, second zap window) and the park-plate anchor (byte-identical round-11 reply
death). Whole binary 6 passed / 0 failed / 1 ignored, 149.84 s [L the final-tree run]. The
refreshed ledger + the next levers (the off-rim zap-free standing spot; the crouch-window
plan; the reply-cadence live check) are in the ignore message. Report:
docs/reports/retune-nb-08-12.md.

## retune-nb-08-13
2026-11-21. Round 13 of the 08 whole-mission 00 Agent real-damage walk
(walk_the_whole_mission_on_real_input_real_damage_00agent). Branch s/retune-nb-08-13 off
origin/master (6a600cb02) with s/retune-nb-08-12 merged in (clean). Shipped the ledger's lever
(2), the crouch-window stance plan (d3-gated): the sustained hurt-volume burst now taken as 3
rows (-6.0) instead of 12 (-24.0), every stalemate volume row after it crouched through, rocket
1 byte-exact (launched t49, landed -101.0). The ledger's lever (1) skipped: round 6 measured the
lower-floor standing class dead ("no standing spot exists on the lower floor", three lever runs).
Tried and dead on their own runs: the east-dip stand (t28257, the state-2 cannon pass chewed the
standing capsule) and the salvo override (t29352, the dodge side pick walked Bond onto the round-11
mullion corner). Shipped state dies t29801 at (113.08, 13.09, -179.88) crouched on the trench —
3525 ticks deeper than round 12 — to one -6.00 w62 falloff (the crouched strafe cannot out-run the
missile's falloff; the machine 497/598, clip 3, no second launchable pass measured — the
west-high rays stop t0.748, 0.052 short). Whole binary 6 passed / 1 failed, 176.86 s [L the
final-tree run]. The refreshed ledger + the next levers (the launch starvation; the trench-pinned
standing dodge; the reply-cadence live check) are in the ignore message. Report:
docs/reports/retune-nb-08-13.md.

## retune-nb-08-14
2026-11-21. Round 14 of the 08 whole-mission 00 Agent real-damage walk
(walk_the_whole_mission_on_real_input_real_damage_00agent). Branch s/retune-nb-08-14 off
origin/master (b69bacbc9) with s/retune-nb-08-13 merged in (clean); fresh sandbox (repo, game
files, toolchain re-setup). The baseline reproduces round 13 byte-exact: death t29801 (frame
29803) at (113.08,13.09,-179.88), 62 hits for 100.0, health 6.0 entering the killer salvo
[L the 2026-11-21 baseline NF_DMG_TRACE run, 24.71 s]. The ledger's lever (1) THE LAUNCH
STARVATION (the west window: stop 0.7 + band 21.0 for west passes only, rockets >= 1,
stalemate-only) TRIED and DEAD on its own run: a second launch fired t1802 at the machine's
west-high pass (53.07,18.54,-174.05) but the death moved t27583 — 2218 ticks WORSE (the window
held Bond standing through the volume's burst rows −2.00 × 10 and the 60.3-u ride froze him
mid-ride standing; the reply kind-44 100.0 blast landed d 2.8; also measured: the fired pass's
own ray stops were t0.862-0.875 ≥ 0.8 — the 0.7 stop widening contributed nothing, the 21.0
band was the whole lever). Reverted. SHIPPED lever (2) THE TRENCH-PINNED STANDING DODGE
(d3-gated, test-file only): the driving dodge stands Bond from the crouch (the shipped flip
press under the 60-tick lockout [M Player_ChangeSubState 0x190630:48]), burst_hold clears and
the fresh-zap arm is gated off while the dodge drives (prefix-byte-exact: all 112 baseline w62
rows are the t3160-4030 crouched stalemate [L the baseline run]); the latch's both-open pick
scores the trench side (the east-walking diagonal wins; a west-pointing chosen diagonal drops
its forward). MEASURED: the FIRST killer salvo is now SURVIVED — the miss grows 0.58 → 3.24 →
2.41 standing (the crouched strafe held 0.56-0.58) and the falloff drops −6.00 → −3.98 (health
6.0 → 2.0); but the east-side diagonal walked Bond 1.4 u NORTH off the plate (z −177.65) where
the east lane wedges the standing strafe (0.05 u/30 ticks) and the walk-back south crawls
~0.016 u/tick, and the SECOND missile (north-flying, dir (−0.194,-0.109,0.975)) arrived
mid-walk-back with the miss stuck 1.40-1.41 for 40 ticks — its falloff −2.02 closed the
2.0-health walk t29924 (frame 29926) at (113.08,13.47,-178.98), 123 ticks deeper than round 13.
Whole binary 6 passed / 1 failed (the target walk), 176.48 s [L the final-tree run — the d1/d2
sisters, the plain walks, the d2 Agent sister and both ally tests byte-exact]. The refreshed
ledger + the next levers (the second-salvo lane; the west launch starvation measured dead from
the rim; the reply-cadence live check) are in the ignore message. Report:
docs/reports/retune-nb-08-14.md.

## retune-nb-08-15 (2026-11-21, round 15)

Branch `s/retune-nb-08-15` from origin/master, origin/s/retune-nb-08-14 merged in (the only
conflict was the walk's ignore line; kept the 08-14 ledger — master's run-12 census finding is
the ledger's round-5 lockout answer, already recorded). Continuing the lever ledger on
walk_the_whole_mission_on_real_input_real_damage_00agent from the round-14 state (death t29924,
the first killer salvo survived): the next untried lever is (1) THE SECOND-SALVO LANE — score
the dodge diagonal on keeping z inside the pocket band −179.6..−178.9 while preferring the east
x, judged on its own real run. Helpers stay gated to 08 + d3. Progress appended below as it
lands.

## retune-nb-08-15 (2026-11-21, round 15) — continued

Sandbox reset this round: the repo, the game files and the toolchain were re-set-up (clone,
work-slim + ISO, rustup 1.100, the extracted libasound2 debs, cargo fetch). A stale remote
`s/retune-nb-08-15` (an earlier session's setup-only branch, based on a master 16 commits old)
was superseded by a force-push of the task branch rebased on current origin/master (ae5fc9219)
with origin/s/retune-nb-08-14 merged in — the only conflict was the walk's ignore line, kept
as the 08-14 ledger (master's run-12 census finding is the ledger's round-5 lockout answer,
already recorded).

- **The baseline reproduces round 14 byte-exact** on the merged tree: death t29924 (frame
  29926) at (113.08,13.47,−178.98), 63 hits for 100.0 [L the baseline NF_DMG_TRACE run,
  24.66 s]. The killer-salvo chain anatomy is now measured: 8 w62 rounds t3160−t4150 from 8
  rotating bearings, the first ~6 pass CLEAN, the last two close on the wedge/lip rows.
- **Ledger lever (1) variant (1a), the band-scored latch pick — TRIED and DEAD** (death
  t29917, 7 ticks worse; the north-wedge row avoided but the exits flipped south, −1.34 +
  −4.66, the same −6.0 total). **The upstream stalemate cannon dodge — TRIED in two steps and
  DEAD** (step 1: the hold latched but the shipped d3 drive is w62-only, byte-identical
  timeline; step 2: the crouched stalemate strafe drifted Bond onto the south-west rim,
  death t27885, 2039 ticks worse). Both reverted; the shipped state is the final tree.
- Fresh measurements in the ledger: the chain's detonation ring sits 2.5−3.5 u out on several
  bearings around the 0.7-u pocket — two rows (−1.3..−4.7) are unavoidable from the pocket;
  the chain is un-survivable at the 6.0 entry health, survivable at ~10−12.
- Whole `walkreal_07000008_live` binary on the final tree: **6 passed / 1 failed (the target
  walk), 177.54 s** — the d1/d2 sisters, the plain walks, the d2 Agent sister and both ally
  tests byte-exact green [L the final-tree run]. The refreshed ledger + the next levers (the
  walk-back family (1b)/(1c) with the per-pass drift cap note; the sustained-burst 3-row live
  check; the carried trio/reply-cadence live checks) are in the ignore message. Report:
  docs/reports/retune-nb-08-15.md.

## retune-nb-08-16 (2026-11-21)
08 whole-mission 00 Agent real-damage walk, round 16. Branch s/retune-nb-08-16 from
origin/master with origin/s/retune-nb-08-15 merged (d1373ac73). Round-15 ledger read: the
death has sat at t29924 for two rounds, so this round tries a lever upstream of that fight
(route, order, cover) per the task line, taking untried levers from the ignore message and
skipping everything measured dead. Sandbox reset: full re-setup (clone, downloads, build).

- **Round 16 (2026-11-21): went UPSTREAM of the killer chain per the task line; the death
  stays t29924 but the ledger now carries the FULL upstream budget and two measured-dead
  levers.** Baseline reproduced byte-exact on the merged tree (master d57c3fdac + 08-15):
  death t29924, 63 hits/100.0 [L the NF_DMG_TRACE run]. The budget: the station hall −48.0
  (12 × −4.00, the trio + #9 + #10 at the y-46 deck), #16 −8.0, the machine-fight volume
  zaps −38.0 (approach 9 standing + sustained flip 3 + CROUCHED tail 7 at the anchor), the
  chain −6.0; chain entry 6.0. **The ledger's item (2) pre-crouch is DEAD BY DATA** — the
  baseline shows SEVEN −2.00 rows landing on the CROUCHED capsule at the anchor
  (t26318-27902) — the volume reaches the pocket on both stances, pre-crouching saves zero.
  **The round-6 note's zap-free park spot (111.99,−179.51) TRIED and DEAD on its own run**
  (the PARK walk-back, d3-gated post-ride band-closed-only): 12 × −2.00 rows land AT the
  park (both stances) and the walk died t29795 on the south rim during the chain, 129 ticks
  worse — no zap-free park exists near the pocket. NEW LAW: line_ok never reopens after the
  dive launch (0/210 gate samples; the eye sweep stopped 0.02-0.12 u out in all azimuths) —
  the fight's only machine damage is the t49 launch; from the anchor it cannot be won, only
  survived. Whole walkreal_07000008_live on the final tree: 6 passed / 1 failed (the target
  walk), 171.09 s [L]. Next levers: the station-hall −48.0 cover stand (route/order/cover,
  upstream of the machine fight entirely), the volume's machine-powering live check, the
  chain-entry math. Report: docs/reports/retune-nb-08-16.md.

## retune-nb-08-17 (2026-11-21)
Round 17 of the 08 whole-mission 00 Agent real-damage walk
(walk_the_whole_mission_on_real_input_real_damage_00agent). Branch s/retune-nb-08-17 from
origin/master (b5278f0d1, which carries walks-1121-08-cabin's fusebox-pit sight-abort fix) with
origin/s/retune-nb-08-16 merged in (08-16 was not on master; two conflicts in
walkreal_07000008_live.rs resolved keeping both sides' complementary content — the excursion
loop keeps master's leg_abort_on_sight with 08-16's dead-lever comment, the ignore line +
ledger block kept as the 08-16 round-16 ledger). The driver's new direction: enumerate every
hit from the walk's start (tick, source, amount), find the earliest segment that costs the
most — even far upstream — and change it so Bond reaches the end fight with more health
(the t29924 death has sat for three rounds; the 08-16 ledger measured the killer chain
un-survivable at the 6.0 entry and survivable at ~10-12). Setup next, then the upstream
hit-ledger measurement on the merged tree. Report: docs/reports/retune-nb-08-17.md.
Round 17 measured and shipped the ledger only (both levers reverted). The merged-tree
baseline (master's fusebox-pit sight-abort + 08-16) RESHAPED the death: t28183 (frame
28185) at (113.01,13.47,-179.26), 59 hits, 22 return-fire fights — the full hit ledger from
the start is in the ignore message (trio -28.0 / #15 -12.0 / #16 -4.0 / #5 -6.0 / #26 -4.0 /
the PRE-FIGHT approach burst -28.0 / #25 -18.0 during the dive ride). Lever (a) the JIGGLE
FIGHTS (the accuracy-law motion factors [M 0x179740:52-54; ACTION.ELF statics 0.25/0.75 [D]])
tried on its own run and DEAD (the deck took 8 rows and the re-rolled timeline wedged the
boarding descent t26700); lever (b) the post-launch survival crouch tried and DEAD as a
byte-identical no-op (the standing zaps are pre-fight, the killer rows are #25 during the
ride). Both reverted; the final tree is the merge itself. Whole binary 6 passed / 1 failed
(the target walk), 165.28 s [L the final-tree run]. The next lever (the PRE-FIGHT crouch +
the ride-free survival path) and the fight-order item are in the ignore message. Report:
docs/reports/retune-nb-08-17.md.
## retune-nb-08-18
Setup from scratch (sandbox reset): clone, work-slim + the verified 369823203-byte ISO
archive, rustc 1.100, the user-prefix libasound2/libasound2-dev debs, cargo fetch.
08-17 was not on master: merged it first. Lever (1) the pre-fight crouch + the first-pass
west-gate hold-off TRIED and SHIPPED on its own NF_DMG_TRACE run: death t28183 -> t28907
(+724 ticks, the approach burst crouched), zero launches, the Hind damage 0.0 of 598, the
machine never re-dove within 989 fight ticks (the route flies y >= 14.6; the arrival dive
is the only y <= 13.7 window), #25 hp 20 intact through 22 return-fire fights (the armed
launcher cannot fight a guard) — 12 un-gone 0x44 dart pickups at 1.6-2.8 u from the anchor
are the next cheap lever. Whole binary 6/1, 162.00 s [L]. The next levers (the fight-order
common/ savings, the dart scoop-and-arm, the dive-cadence read) are in the ignore message.
Report: docs/reports/retune-nb-08-18.md.
## retune-nb-08-19
Branch start: sandbox reset, redoing setup. 08-18 not on master: merged it first
(fast-forward, master had not moved). Round 19 targets: the machine fight cannot be won
from the current anchor (round-18 finding) — try a different place or approach for that
fight; levers from the ignore message, skipping measured-dead rows.
## retune-nb-08-19
Setup from scratch (sandbox reset): clone, work-slim + the verified 369823203-byte ISO
archive, rustc 1.100, the user-prefix libasound2/libasound2-dev debs (incl. the runtime
libasound2t64), cargo fetch. 08-18 not on master: merged it first (fast-forward). THE
DIVE-CADENCE READ DONE AS [M]: Copter_Move 0x17d628:63-113 advances/loops the route-path
chain by damage/(0.9*hp) — zero machine damage = no advance ever, the round-18 siege was
unwinnable by law; Copter_Update 0x17dbb8 case 1-2 pins the attack cadence. Lever (1) the
DIVE-LAUNCH SHAPE shipped (d3-gated, the 08 test file only): the dive-stand + the
sag-band launch gate (launch_line_clear_far, t >= 0.8 — the dive rays stop t0.936 on the
gondola cable) + the pre-loop aim on the switch ticks + the post-ride crouch. MEASURED:
the dive launch fired t43, LANDED -101.0 (the first dive-window machine damage since
round 12), death t28907 -> t29335 (+428), zero machine rows on the crouched pit in ~1100
ticks; the machine's path never advanced (path-t 1154 at death; the loop's dive ~700-800
ticks past the death). Lever (2) the #25 sub-fire TRIED and REVERTED (two variants): it
KILLED #25 but the rotated yaw flipped the dodge's side pick and a salvo blast closed the
walk 271 ticks earlier — ledger material. Whole binary 6/1, 167.46 s [L]. Next levers in
the ignore message. Report: docs/reports/retune-nb-08-19.md.
## retune-nb-08-20
Branch start: cloned fresh (sandbox reset), merged origin/s/retune-nb-08-19 into
origin/master tip (stream batch 3, b60b1ff79) as instructed. Round 20 goal: the
whole-mission 00 Agent real-damage walk green again. Re-measure the baseline on the
merged tree first (stream batch 3 moved guard aim/body boxes), then take the next
untried lever from the ignore message, skipping measured-dead rows.
Round 20 (2026-11-22): branch from master (stream batch 3) + 08-19 merged. THE MERGED-TREE
BASELINE FIRST: the batch-3 running jump vs the d3 lip+hop conflict wedged the walk at the
hall east lip t6540 (6000-tick cap; the in-place hop is dead under the one-sided wall push —
the hop jump parks ON the lip step, the running-jump comment's own law). SHIPPED: the d3
east-deck-head crossing switched to the running jump itself (d1/d2 byte-exact). Lever tried
and shipped: the MOST-HITS-FIRST retarget (opt-in thread-local, 00 Agent only) — the fight
retarget samples EVERY tick and reads the worst shooter over the WHOLE set: death t16372 ->
t25783 (+9411), the walk reaches the d3 machine-fight anchor. A first variant (the wide-ring
pick most-hits-first) measured byte-identical no-op and was reverted. Whole 08 binary 6/0/1
on every push. Next: the fall 23.0, the secure-room #6 re-deal range gate, the machine fight
from the anchor. Report: docs/reports/retune-nb-08-20.md.
retune-nb-08-21: 08 whole-mission 00 Agent real-damage walk green again, round 21.
Round 21 (2026-11-22): branch from master 9fff71e1c with 08-20 merged (08-20 was not on
master; clean merge). Setup from scratch (sandbox reset; freed 5.6 GB of pre-seeded caches
to fit the four-crate target tree). THE MERGED-TREE BASELINE FIRST: death t25783
byte-identical to round 20 (48 hits for 77.0 + other 23.0, 17 fights) — and THE DEATH READ
IN FULL: the machine's own rocket blast (kind 44, d 3.5, radius 8.0) after the high-phase
salvos drained the 50.0 armour; health 2.0 since t5020, the armour carried ~20000 ticks.
Levers: the fall CLOSED as accepted stream cost (no game ladder at the rail; the d1/d2
sisters run the same jump and pass — the game's own player pays the 23.0); the secure-room
#6 re-deal TRIED TWO WAYS and DEAD (the aperture-align +0.45 stand kept the same -15.0, the
lane covers the walkway; the line-break -0.45 stand went blind and left #6 advancing — the
resuming walk died t5260 at d 4.9). Both reverted. Whole 08 binary 6/0/1 on the final tree.
Next: the machine fight from the anchor at health 2.0 + armour 50.0 (the salvos are the
wall), fed by the upstream banks (the deck fight's 60.0). Report:
docs/reports/retune-nb-08-21.md.
## retune-nb-08-22
Branch start: cloned fresh (sandbox reset), branch from master 9fff71e1c with 08-20 and
08-21 merged (08-21 was not on master; clean fast-forward, 08-20 is its ancestor). Round 22
goal: the whole-mission 00 Agent real-damage walk green again. The task's law: the lever must
SAVE HEALTH BEFORE THE ANCHOR (an earlier fight skipped, taken from cover, or done in a
different order) — Bond reaches the machine-fight anchor at 2.0 health. Skip every lever the
ledger marks measured dead; take the next untried one from the ignore message. Judge each
lever on its own real run. Whole walkreal_07000008_live binary at the end.

Round 22, second session (sandbox reset, all setup redone from scratch): the round's first
session had already shipped the WEAVING fight stands (the anchor reached at 37.0 health +
armour 50, was 2.0) — the walk still red at t21154. This session took the ledger's item (3),
the excursion legs' stop discipline, and judged FOUR own real runs: variant 1 (stops between
the legs) WASH — leg 1 paid x0.75 unlatched (the arrival ease held +0x1DC non-zero), a row
landed DURING a stop, the 120-tick first-moved window expires inside the 330-tick crawl legs
(death t22980); variant 2 (+ the pre-leg stop) banked the pit (24 -> 12) but the ~100-tick
pace shift re-rolled the boarding ledge's unseen pair — #27 burst 28.0 where the shipped
stream paid ZERO (death t20785); VARIANT 3 SHIPPED — the pre-leg stop ALONE (6 no-input
ticks before the first descent leg, the stream shift 6 ticks): pit 24.0 -> 12.0, the
mid-game transit clean, the machine-fight entry armour 0.0 health 49.0 (best ever), death
t25843 — the deepest walk yet (+4260 over the merged-tree baseline t25783), killed by ONE
kind-44 reply blast -45.00 (d 3.5) into exactly the 45.0 health left. A fourth run (the
anchor shifted 0.35 u NE) TRIED and REVERTED: the dive launch window closed (ZERO launches,
hp 598 untouched) and the fight re-dealt into a 19-zap stalemate (death t27890) — and it
pinned that the kind-44 reply is the machine's response to Bond's OWN rocket (suppressing
the launch is self-defeating). Whole walkreal_07000008_live binary on the final tree:
6 passed / 0 failed / 1 ignored, 272.28 s. nf-game lib 1080/0/4, nf-audio all green. The
refreshed ledger + the next levers (the kind-44 reply answer that keeps the launch, the
crawl legs' in-walk stop discipline, the armour-entry timing) are in the ignore-message
comments. Report: docs/reports/retune-nb-08-22.md.

## retune-nb-08-23 (2026-11-22)

Task: 08 whole-mission 00 Agent real-damage walk green, round 23. Branch = master 7562f1e70 + s/retune-nb-08-22 merged (mirrored files took master, the 08 test took the rounds line + master re-times).

Progress:
- merged-tree baseline: death t21550, killer = Bond own rocket self-blast (owner -1 kind 0x2c, 3.7 u, -14.00) after the l 44.7 far launch; fight entry 21.0/23.0 armour.
- SHIPPED: the launch distance cap (l<=20, d3) -> t21708; the dive-stand weave (the round-22 strafe pattern in the fight loop held window) -> t22500 (+950, deepest on this tree).
- next levers in the ignore ledger: the salvo close band vs the 6.3-miss wall-band pairs, the nav-stand weave (shared helper), the armour entry.

## retune-nb-08-24 (2026-11-23)

Task: 08 whole-mission 00 Agent real-damage walk green, round 24. Branch = master 7562f1e70 + s/retune-nb-08-23 merged.

Progress:
- branch created, round-23 tree merged (death t29609 baseline, three levers shipped in 23: launch distance cap, dive-stand weave, 6300/60 re-roll stop).
- reading the 08-23 report + ignore ledger to pick the next untried lever.
