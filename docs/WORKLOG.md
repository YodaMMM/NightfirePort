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
