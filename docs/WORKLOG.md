
## level-13-warm-chain-margin — branch cut

Branch cut from origin/master ef7822e8b. Task: Chain Reaction's (07000013) ignored walk
walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage dies on
master at tick 2448 at (4.03,-368.39,33.35) with 3 hp after a 3.50 hit from drone #5 (weapon
0x19); the same route passed on d09f8b2b3. Bisect the walk across d09f8b2b3..master for the bleed
change, judge the new damage against the decompile [M], then re-route or re-tune the walk on real
input only (no top-ups, never lower damage or health) so it passes with a real margin; un-ignore
it. Run the whole walkreal_07000013_live binary and interactables_07000013_live.

## level-13-warm-chain-margin — bisect done (first bad 7876b7fe), the margin work mid-flight

The bisect (git bisect run over d09f8b2b3..master, the walk itself as the probe, every verdict a
real FAILED not a build error) pins the first bad commit: 7876b7fe "Ally deaths blame the right
killer; level music machines; turrets take rounds; laser cuts bolts only" — its 12-leg keypad
change (tower2.rs: the laser burn replaced by the typed code) re-times the 12 leg, and the 13
lobby then meets a different spawner-wave state: the wave lands ON the east lane and a shotgun
guard unloads 7 pellets (−0.5 + 6 × −6.0 = −36.5 in one tick) point-blank. The damage itself is
the game's own math [M player_hit_damage (drone.rs 0x.../DroneWeap family): pellet damage × the
Agent pain scale × DroneFiring_TooClose_Damage at d 2.6] — the walk's exposure is the bug, not the
numbers. Re-tune shipped on the branch (real input only, no damage/health writes): the opt-in
tower2::PRE_CLEAR_NORTH_LOBBY (the 12 north lobby's pursuit walked Bond down at master), the east-
lane sweep at 13 (takes the wave before the re-anchor), the west-mid stand moved one leg west (the
stand window itself was the exposure), the sweep stands capped at 700. The walk now reaches 13's
west legs and dies at tick 2242 (−12.59,−368.39,39.91) inside the north-west sweep's failing #9
stand — the un-ignore is NOT shipped. The d1/d2 sisters pass on every run (2 passed). Report:
docs/reports/level-13-warm-chain-margin.md.
