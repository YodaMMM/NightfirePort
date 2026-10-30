# WORKLOG

## level-15-interactables (branch s/level-15-interactables)

- start (master b3c4e4a1). Task: one live test per KIND of player-touchable placement on
  07000015, driven the way a player drives it, each with a sister (model
  `interactables_0700000b_live.rs`); census every player-touchable placement [D placements +
  docs/levels/07000015.md §7]; add the missing kinds to `interactables_07000015_live.rs`; fix
  every port gap the tests expose.
- outcome (2026-10-30): the missing kinds added to `interactables_07000015_live.rs` —
  destroyables ×2 (the east-room BoxSatellite smashes through the room touch's ch 0xf6 via the
  tag+0x10 external-kill channel [M 0x12c318:36 disasm 0x12c3b8-0x12c3bc, 0x12c4f8:30-35]; the
  ch-0xfd oil barrel dies to the admitted handheld shot — the p4 = 0xf8 mask 0xff07 admits
  weapon word 0x0008, drops the laser/watch — and writes switch_channels[0xfd] [M 0x12bd48:35])
  and the spawner/drones squad (the group-1 templates #699/#700 are ABSENT until the touch;
  mode 3 brings ONE in [M 0x137e68's mode arm], present + ch-0xf6-armed). Census corrections:
  satellite p1 = the kill channel (not "the group"); spawner p0 = 3 is the MODE, p1 = 1 the
  group; the barrels' p4 = 0xf8 is the mask source; the drones row notes the template absence.
  No port gaps in the class code — two test-side fixes (the barrel lookup keyed on the
  level-unique death channel; the mode-3 single-spawn reading). 14/14 green
  (+ walk 2/2, walkreal 1/1); the fast runner's 4 problem lines = the documented slim-ISO
  baseline (identity_disc + 3 disc_golden). Report: docs/reports/level-15-interactables.md.
