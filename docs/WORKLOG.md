# WORKLOG

## level-4a-interactables (branch s/level-4a-interactables)

- start (master 76081986). Task: model on `interactables_0700000b_live.rs` — read what the
  existing interact_/walk tests for 0700004a already cover, census every player-touchable
  placement on 0700004a ([D placements + docs/levels/0700004a.md]), add the missing kinds as
  `crates/nf-view/tests/interactables_0700004a_live.rs`, fix every port gap the tests expose.
- outcome (master 76081986 merged with origin/master 89dab718-line): the census re-run accounts
  for ALL 294 m41 placements — the two rows the §7 table missed are the kind-0x31
  "OBJECT_Lift_At_Floor_67" (the lift platform entity 0x020005b4, flags 0x2031, no params →
  FuseBox_Create's script gate skips it, never a fusebox [M 0x1b0448; D census]) and 2 kind-0xE9
  AI points (the AI network, not player-touchable). New `interactables_0700004a_live.rs`: the
  platform-0x31 pin + the rider's real-E door legs (the unlocked −48 pass opens, the locked −52
  pass refuses before t1610, the landing-scene sisters: ch 0x83 never rises — the twin SP
  #273's out-channel trigger can't arm). Doc §7 corrected (the two rows + the E-operable
  paragraph: the rider's pass-E IS the level's use-key census). No port gap exposed: the door
  use-pick, the lock refusal, the SP out-channel trigger and the scriptless-0x31 skip all match
  the decompile. Parked (unchanged): the drone fire leg from the ride line (0/2400 clear rays,
  the existing file's blocker).
