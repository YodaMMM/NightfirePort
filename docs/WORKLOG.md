
## spawn-pick-and-dart-stamps (2026-11-06, s/spawn-pick-and-dart-stamps)

Task: port two drone-AI divergences found reading the decompile, each with a fail-first test:
1. The spawner pick: the game draws Rand_Rand over ALL templates and spawns nothing when the
   drawn template is used or too close [M 0x137cb0:21, 0x137aa0:32]; the port draws
   rand(usable.len()) and always spawns (spawners.rs:194). The port also makes none of the
   game's spawn-chain draws [M 0x14b300:382-461, 0x137cb0:58]. Port both; live PCSX2 wave-frame
   comparison on 0700001b goes to the report as a live check.
2. Darts on the Tower levels: on_dart_impact skips the stamp block the tase runs
   [M 0x1469b8:322-348 vs :89-128]. Port and pin.

Cut from origin/master c4c00c7ec (same tree the civ-unignore tasks ran). Plan: read the four
decompile sites + the port's spawners.rs/drone.rs dart path, write the fail-first tests, port,
then re-pin any walk the new draws shift BY ROUTE (never by removing draws), and run every live
binary that touches spawners.

### spawn-pick-and-dart-stamps progress (2026-11-06, ~03:55 UTC)

- **The spawner pick is ported** [M 0x137cb0:21 + 0x137aa0:31-32, :41-56, :88-96]: one
  Rand_Rand over the WHOLE template list (+0x2E == +0x8, both the census count), drawn BEFORE
  the walk; only usable records decrement; a rank past the last usable spawns NOTHING this
  call. The old rand(usable.len()) always spawned. Fail-first unit tests in the module (the
  modulus assert + the missed-rank assert both failed pre-fix; a_rank_zero passes both models).
- **The spawn-chain draws are ported** [M 0x14b300:382-461 + 0x137cb0:58]: the param-14 class
  switch re-rolls at every spawn (the same block the constructor runs at load — 2 draws, case 4
  draws none), the +0xF4 tail Rand_FRand(0) consumes the stream [M :435], the 0x7000011/14
  engage/combat overrides, and SpawnDrone's Rand_Rand(10000) → obj+0xEC re-rolls the wobble
  (the "unidentified writer" the port's wobble doc carried — it is the spawner path's own stamp).
- **The dart's Tower stamp block is ported** [M 0x1469b8:331-357 — byte-identical to the tase's
  :103-128]: fail-first live pin `the_dart_stamps_the_tower_alert_block_like_the_tase` in
  walkreal_07000009_live.rs (the challenger is sighted; the pre-fix run printed sighted 1,
  alt_mode 14, return_state 0x10, first_attack 1 — all wrong).
- **Walk re-pins by route**: walkreal_07000009's alarm-switch approach now beat-splits and
  accepts the press reach (1.4) — the measured failure was the old single hop WEDGING 3769
  ticks at the switch's 1.08 floor (its "arrival" was the level-exit latch passing the assert
  truthy) while a woken sleeper pressed.
- **Whole-binary results (quoted, --test-threads=1)**: nf-game lib 686/0/5 (incl. the 3 new
  unit tests); walkreal_0700001b_live 17/0/2 (124.16 s — the Drake spawner waves survive the
  new draws); walkreal_07000009_live 9/0/0 (13.72 s); walkreal_0700000a_live 13/0/2 (48.01 s);
  npc_doors_live 6/0/1; fix_guard_misc_live 6/0/0; drake_fight_live 3/0/0;
  walkreal_07000005_live 13/1 — the 00agent 05→06 chain died at t9526 (−35.02,38.49,−43.14,
  13 hits for 100.0): the shifted stream made the 00-fights costlier (40.0 hp at t2174, 23.5 by
  t4321) and the courtyard descent finished him — a real-damage 00 re-pin, not started.
  walkreal_0700000c/0d/12 built but not run in the window.

### spawn-pick-walk-05 started (2026-11-06, ~15:40 UTC)

- Branch `s/spawn-pick-walk-05` = origin/master a6665d115 + merge of origin/s/spawn-pick-and-dart-stamps
  (59072b88c, clean). Task: settle the one red the reviewed branch left — walkreal_07000005_live's
  00agent 05→06 chain — by finding WHY the chain moved; re-route the walk test-side if the new spawn
  rank merely changes who spawns where, fix the change if it is itself wrong (cite the decompile).
- Plan: reproduce the death (t9526, −35.02,38.49,−43.14), trace the shifted stream into 05's
  proximity-spawner attempts, then read the decompile of the spawner pick [M 0x137cb0:21,
  0x137aa0:31-32,41-56,88-96] again on the 05 templates to classify moved-vs-wrong.
