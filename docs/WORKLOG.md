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

## Outcome (2026-11-01, ~00:25 UTC)

- DONE: both upper-floor re-seat shortcuts are gone. fix_finale_live's 07000016 test and
  kiko_button_real_walk_live now walk the disc's own chain after the pack-cinema hand-back —
  the west blast room through both ch-0xf3 pulses (559/510 ticks [D 0600090a streams 6/9]), the
  chaser to the test's grenade (param 12 = 0 [D #261]), the pit squad to Bond's fire (a squad
  death never raises ch 0x90 — the sister), ch 0xbf [D mux #611], and then the LIFT-OUT cinema
  0x06000632 (07900016.bin) no-skip: its t1056 stream-16 clear takes ch 141 = 0x8d down on the
  walked route and its t1057 event-6 hands Bond back at key 1868 = (5.39, 1.34, −158.31) beside
  the button [D; M 0x1d94c0 event 6 → Script_Free → Player_Enable 0x195498] — the game's own
  transition in place of the re-seat; no bond.pos writes remain after the tests' telescopes.
  kiko_button's header corrected: the disc DOES clear the button lock — the old census decoded
  07000016.bin + 07800016.bin and missed the 07900016.bin pack. The walked press now COMPLETES
  (E → ch 0x90 → mux #618 → ch 100 → MissionState 6 → 0x7300017 [M 0x182c60:111-112]); the old
  "the walked press is INHIBITED" pin rested on the wrong no-clear claim and is gone. One timing
  note: the gun duel with the chaser missed the 6310 door deadline on these routes, so both
  tests keep their pre-task grenade kill for her (fuse-only explosion [M 0x12ce80]); the squad
  dies to Bond's fire (walkreal shape).
- Named binaries (final tree): walkreal_07000016_live 9 passed / 0 failed / 0 ignored;
  interactables_07000016_live 8 / 0 / 0; fix_finale_live 4 / 0 / 0; kiko_button_real_walk_live
  1 / 0 / 0; blast_room_shelter_16_live 2 / 0 / 0; nf_game 588 / 0 / 5. The 16 finale-chain
  tests are all green.
- Suite (tools/test/chunk_par_tmp.py, fresh state, 2 workers — the sandbox kills true
  background jobs; priorities: the 07000016 family + nf_game): 394/996 binaries recorded by the
  00:22 UTC stop line = 1782 passed / 1 failed / 122 ignored. The one failure is
  probe_pylon_render (rc 101, the wgpu adapter panic — no GPU in this sandbox; environment, not
  a regression). The tail was not reached.
