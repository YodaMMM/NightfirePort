
## level-0c-yard-climb-real-health (started 2026-11-02 16:48 UTC)

Branch cut from origin/master edd5748b. Task: walkreal_0700000c_live's real-health run (ignored)
fails the yard climb — "never mantled past climb_top 12.139 [M 0x196118/0x196310]" at
(9.95, 7.95, -186.93) after Bond took 80 damage; the top-up walk mantles there. Plan: (1) find
whether damage (pain/flinch/knock-back/speed) gates the port's mantle and whether the game's
climb has the same gate [M 0x196118/0x196310 + the pain path], (2) fix the port where it differs
(cited) or route the walk to the climb after the fight if the game blocks it too, (3) run
walkreal_0700000c_live both variants, every climb/mantle test, the full suite.
- 16:50-18:00 UTC: the mechanism answer is complete; the walk stays ignored. (1) The port's climb has NO pain gate and neither does the game [M Player_Climb 0x196310 - the stick advances the climb, the only up-block is the 0x20 head-hit flag; Player_HandlePain 0x1902c8 writes no substate; Player_CollLadder 0x196118:17's sole health read is the grab gate (hp > 0 to grab)] - the port MATCHES the game: the real-health run's Bond climbed at 19.8 -> 3.8 hp (the climb ticks advanced normally through two -16 hits) and froze at y 7.95 only when hp hit 0.0 - the death sequence [M Player_CheckForDeath 0x190810 -> the dead branch: no substate arm, no collision pass] correctly ended it. (2) The kill chain named [L the damage log + the yard census]: #18 (skin 0x5000090, weapon 0x1e, (36.4,12.1,-208.8), hp 10, a STATIONARY sentry) dealt 51.8 of the 100 in 4 hits from d ~34 u, outside the plain 20 u shooters window the whole run; #1 (13.0,12.1,-196.0, d 9.6 from the base, level 7.6 from the ladder top) killed Bond within ~110 ticks of the mantle. (3) The walk-side stands (real damage only, the helper yard_sniper_stands): the aimed-0x1e direct switch [M Player_WeaponChange mode 0; the aimed spread is zero [M player_spread]] + the fight machinery - the stands SUPPRESS #18's beat (the climbs with a stand survived to the mantle; without one, death on the ladder) but the UPWARD rounds never land: the aimed 0x1e at a stationary box d 33, perfect aim (err 0.000, blocked false, gunrays true), three clips dry, tgt hp 10.0 - while the LEVEL-geometry deck fights land. The scoped player-bullet flight vs the eye-ray/gun-ray predictor disagreement on upward shots is the port gap to chase next (the predictor uses the flat trace, the flight the cel-scoped walk). lane_fights tried on this route and reverted (the 48 u fights stall and trade worse - death at the staircase tick 6412). (4) Tests: walkreal_0700000c_live 2 passed 1 ignored (the sister re-ignored with the measured reason), the climb/mantle group fix_ladder2_live 4, fix_ladder_live 4+1i, fix_ladder_water_live 4, ladder_02_live 3, ladder_underwater_02_live 6 - all green; nf-formats + nf-audio 136 passed 3 failed = the 3 allowed slim-ISO disc_golden. The branch touches ONLY the two test files + the WORKLOG (nf-game/nf-formats/nf-audio/nf-view src untouched - the full 4-crate build cannot fit this window; nf-game's state is master's own).

## level-0c-yard-climb-real-health-2 (started 2026-11-03 10:50 UTC)

Branch cut from origin/master 63e76f06 (player-upward-shot is already fully merged into master:
`git log origin/master..origin/s/player-upward-shot` is empty; `DamageLog::aimed_fights` and the
two upward_shot_0700000c_live proofs are on master). Task: finish the 0c real-health walk —
opt `yard_sniper_stands` into the aimed fights (`DamageLog::aimed_fights`), kill the deck
sentries (#18/#15-class, weapon 0x1e, hp 10) aimed from the strip, then climb and mantle;
un-ignore `walkreal_0700000c_live`'s real-health sister (~line 409) on real input only, never
lowering damage or health. Then run walkreal_0700000c_live, upward_shot_trace_live, the
climb/mantle group (fix_ladder*_live, ladder_02*_live) and nf-game, and the full suite.

## level-0c-yard-climb-real-health-2 (continued 2026-11-03 11:35-13:25 UTC)

Sandbox had reset: re-cloned, rebuilt work/ + ISO + rustup from scratch; the branch (a679176f) was
already pushed, continued on it. Merged nothing (player-upward-shot is master already).
- The opt-in machinery: `with_yard_aimed_stands` (thread-local, 0c-local module) sets
  `dmg.aimed_fights` for the WHOLE real-health walk's fights + the yard stands
  (`yard_stand_aimed`/`yard_sniper_stands`): the emplacement-pair sweep at the door (before the
  crossing) and at the ladder base, the recent-hitter stands, the pre-climb suppression pass over every
  alive 0x1e within 48 u (#15 the patroller + #1 the walkway deck acquire late), and the ladder-top
  stands after the mantle. No health/armour/ammo writes anywhere; flag off = master behaviour (proven:
  the top-up walk + ladder495 green).
- Measured (6 real-input runs, NF_RF_TRACE): the aimed fights cut the en-route damage 71 -> 54 hits and
  the door sweep kills #36 in ONE aimed round; but #18's hp 10.0 never moves under 7+ CONVERGED aimed
  rounds (err 0.000, blocked false) from BOTH the door and the ladder base — the twin #36's body/ corpse
  meets every round first [M Drone_CollisionHandler 0x140968]; and the health budget is negative (yard
  arrival 33.6 hp, the crossing/base cost 2 x -16, the fatal graze from #15 (18.4, 12.2, -216.7) at
  1.6 hp). The sister was un-ignored mid-session and re-ignored with the full measured reason + the next
  levers (the report carries the details + 3 live checks).
- Runs on the final tree: walkreal_0700000c_live 2 passed 1 ignored; upward_shot_0700000c_live 2;
  upward_shot_trace_live 1; fix_ladder_live 4+1i; fix_ladder2_live 4; fix_ladder_water_live 4;
  ladder_02_live 3; ladder_underwater_02_live 6; nf-game 655 passed 0 failed 5 ignored. The fast runner
  did not fit the window (the sandbox reset ate the setup time); the touched crates ran explicitly.
- Report: docs/reports/level-0c-yard-climb-real-health-2.md.

## level-0c-yard-climb-real-health-3 (started 2026-11-03 14:14 UTC)

Branch s/level-0c-yard-climb-real-health-3 cut from origin/s/level-0c-yard-climb-real-health-2 (9cf3d8ca),
merged origin/master (35dca502) clean. Plan: (1) reproduce the sister's state on this tree; (2) wall 1 —
find a yard-floor stand whose azimuth to #18 clears #36's body/corpse (sweep spots, trace the drone pair's
real positions), or handle the twin by kill order/a different approach; (3) wall 2 — cut the en-route
damage (the #30/#37 300-tick stands) and suppress #15/#1 before the crossing; (4) un-ignore the sister on
real input only, then the full walkreal_*_live sweep + the fast runner.

## level-0c-yard-climb-real-health-3 (progress 2026-11-03 14:14-15:35 UTC)

Both measured walls of the -2 report are SOLVED on this branch; the sister now survives the yard,
mantles, and dies/fails later (the wire-ride leg — a master regression, below).
- Wall 1 retired with measurements: the -2 "twin #36 corpse shield" does not hold on this tree —
  #36 is present false, enabled false (probe dump, both variants); the PAIR census does not hold.
  The real mechanism: the deck sentry #18 wiggle-steps ±0.1 u, and the zero-spread aimed act
  [M Bullet_init 0x12c5a0:139-170] has NO LEAD — converged rounds (err 0.000) die in flight while
  it steps, land the run it pauses (probe A: hp 10.0 -> -50.0). Fix: the STILL GATE — the 0c-local
  yard stand taps only after the target holds still 18 ticks, and aims with the STRICT gun-ray
  set (the plain eye ray read Some on #15 while every round died on the deck structure — the
  predictor-vs-muzzle-flight gap, measured 5 rounds no land).
- Wall 2: the emplacement dies MID-CROSSING (from the door it is not exposed at all — aim None
  [L]; the old at-the-door sweep never fired a round), the east-ledge snipers #20/#37 (2 x -16
  from d ~59) get an at-the-door sweep, the pre-climb sweep suppresses #15/#1, the ladder-top
  sweep does not wait for a recent hit (#1's first beat lands ~110 ticks after the mantle —
  before any hit window), and the west-edge sweep clears #13 before the wire legs. Measured run:
  #18 killed mid-crossing, #1 killed at the top, #15 silent, the climb mantled, health 17.6 at
  tick 12300 (the -2 runs died at the base at 1.6).
- REMAINING (next session): the #284 wire ride stalls at (-7.447, 12.794, -215.275) d_B 1.6 —
  byte-identical in the TOP-UP walk too, i.e. a master regression, not this branch's change:
  master 6d1ef8e9 (level-0d-real-health-crossfire) made the wire rider retain the last walk's
  move-quad lanes; the rider now hangs 0.6 u OFF the A-B line and is pressed against the
  x=-8.0 plane (+ the 0.55 body radius), 0.14 u above the documented hang y 12.65. The -2 base
  (9cf3d8ca) had the top-up walk green. Needs: the wire substate's move-quad law re-checked in
  the decomp (0x193600's wire case vs 6d1ef8e9's climb-only citation) or the ride leg adapted.
- The sister is un-ignored on this branch (it must not ship red — re-ignore if the ride is not
  fixed this session).
