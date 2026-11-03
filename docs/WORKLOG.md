# WORKLOG

## level-41-every-objective
Started 2026-11-03 04:10 UTC. Branch s/level-41-every-objective cut from origin/master
6f16146e. Task: no walk pins every 07000041 objective in order — extend walkreal_07000041_live
or add a sister test that plays the whole mission on real input from the fresh start, every
objective completing in the game's order, each checked on the channel or objective state the
game reads (objective records, channels, scripts [D the level file, M the objective handlers]),
not on Bond's position. Bond reaches the level end alive, no teleports, no forced fields. Fix
the port against the decompile where a step can't be done the way a player does it, and fix the
whole class. Stay off the exit event (pending 30-level-41-exit-event-2) and name it in the
report if it blocks. Run walkreal_07000041_live, interactables_07000041_live and nf-game.

Work log:
- (04:10) branch start, WORKLOG entry; fast suite runner next.

## level-41-every-objective
Started 2026-11-03 04:10 UTC. Branch s/level-41-every-objective cut from origin/master
6f16146e. Task: play the whole 07000041 wrapper on real input with every objective completing
in the game's order, each checked on the channel/objective state the game reads; stay off the
exit event (pending 30-level-41-exit-event-2).

Work log:
- (04:10) branch start, WORKLOG entry; fast suite runner started (build ok 1089 binaries in
  1517 s, run phase next).
- (04:35) the objective-list finding, re-verified row by row from the snapshot: the MissionData
  table (ee_start_05.bin @0x2a4350, 24 × 0x28) has NO 0x07000041 row → Mission_Init 0x182270
  takes the no-record path (BaseMap = −1, InternalState = State = 5) and Mission_Update's case 5
  is a no-op — zero objective records; nothing can announce or complete. The pins ride the
  mission layer's own state instead.
- (04:50) probes: the chain trigger #99's E spot and the movie trigger #81's north volume are
  BOTH reachable on real input from the airlock (the old teleport-only claims were convenience,
  not geometry); the north deck's mid-floor jams straight steers (dumped at (22.8, −4.1) with
  clear segments — the port's own collision [S]); the proven north route wedges once
  deterministically at (17, −4) and lands at (6.3, 4.1).
- (05:00) walkreal_07000041_live: +the_chain_order_played_from_the_fresh_start (E → ch 1 → the
  real-path chain consumers (turret, voices) → the plate 0x6e → the DynoRod deletes), +the_
  plate_first_walk_locked_e_dyno_rod_tether_and_the_far_south (+real-damage sister), +the_
  north_detour_walks_to_the_movie_trigger_and_its_e_is_a_no_op. 11/11.
- (05:15) DECOMPILE CORRECTION: Trigger_Activate 0x1b2318's case-1 arm (mask&4 → out write) has
  NO lock read; Trigger_Locked 0x1b2fc8's only caller is Check_Target 0x1a7810:197 inside the
  type-10 aim-hint branch. The interactables "locked out after the plate" sister contradicted
  the decompile and passed vacuously (no aim → the use sphere passed over the 0..0.75 y-band).
  Fixed: the_chain_trigger_answers_the_e_before_and_after_the_plate — the aimed E fires ch 1
  with the plate's ch 0x6e on. 18/18.
- (05:25) nf-game lib 655 passed; checkpoint push next; report + final full suite after.
