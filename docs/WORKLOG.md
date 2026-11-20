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
