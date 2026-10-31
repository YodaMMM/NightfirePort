# WORKLOG

## level-16-finale-shortcuts (branch s/level-16-finale-shortcuts)

- start (master 81da6c47 + origin/s/fix-16-finale ad5dd6a5, merged 62ac8914). Task: replace
  the upper-floor shortcut moves in `fix_finale_live` and `kiko_button_real_walk_live` (both
  teleport Bond after the second cinema 0x06000632) with what the cinema does — it drops Bond
  beside the button at t1057 [D per task] — or real input; fix kiko_button's header (it claims
  nothing on the disc clears the button lock; the clear is at t1056 in the 07900016 pack);
  keep all 16 tests green. Continue from origin/s/fix-16-finale (merged origin/master first).
  Named binaries: walkreal_07000016_live, interactables_07000016_live, fix_finale_live,
  kiko_button_real_walk_live, blast_room_shelter_16_live; full suite with tools/test/suite.py.
