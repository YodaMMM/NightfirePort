
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
