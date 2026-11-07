
## level-05-wall-hug-pin — 07000005 wall-hug plate swaps the gate guards while the combat music plays (in progress)

- [ ] 12:15 UTC: started — goal: un-ignore
  `the_wall_hug_plate_swaps_without_the_cut_scene_while_the_combat_music_is_on` in
  crates/nf-view/tests/spawner_kill_live.rs: find what kills the spawned gate pair (hp 10 at
  spawn, stray death during the yard wait), check the spawner record against placement [D]
  and drone-init health, pin the visible plate swap (no cut-scene, no freeze, game's channel
  write order), run spawner_kill_live + walkreal_07000005_live + interactables_07000005_live
  + nf-game.
- [x] 13:28 UTC: done — nothing kills the pair and the port is right: the "stray death" was the
  spawner's own kill latch firing on the plate tick, misread. Chain [M]: the plate #301 is
  TouchOnce (type 1), so `Trigger_Calc` 0x1b1900's pre-walk pass (type 2 only) skips it and its
  `Trigger_Update` 0x1b1a78 runs at its own newest-first walk slot (0x1336f8:54-68) — AFTER the
  placed pair (302/303, still WAIT_SWITCH) and BEFORE SP #298 (refuses, state 5, combat music
  live per 0x1dbda8) and the spawner #268, whose one `DroneSpawner_Control` call latches ch 0x43
  = p7 (0x137e68:50-52) and runs the kill scan (0x137e68:235-288): both spawned guards removed
  the same tick, hp untouched at the game's type-1 default 10 (0x14c600:81-95). The trace's
  frame 174328 pair-present sample is mid-walk (between the plate and the spawner); the old
  plate-tick assertions pinned a tick-boundary state that never exists — the TEST was wrong,
  fixed with the addresses; the ignore is gone and the AI-walks-on pin moved to a 120-tick
  window. spawner_kill_live 6/0, walkreal_07000005_live 14/0, interactables_07000005_live 6/0,
  nf-game lib 688/0; the fast suite does not fit this sandbox's disk (see the report).
