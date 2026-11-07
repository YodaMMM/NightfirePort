
## level-05-wall-hug-pin — 07000005 wall-hug plate swaps the gate guards while the combat music plays (in progress)

- [ ] 12:15 UTC: started — goal: un-ignore
  `the_wall_hug_plate_swaps_without_the_cut_scene_while_the_combat_music_is_on` in
  crates/nf-view/tests/spawner_kill_live.rs: find what kills the spawned gate pair (hp 10 at
  spawn, stray death during the yard wait), check the spawner record against placement [D]
  and drone-init health, pin the visible plate swap (no cut-scene, no freeze, game's channel
  write order), run spawner_kill_live + walkreal_07000005_live + interactables_07000005_live
  + nf-game.
