# WORKLOG

## level-0a-interactables-2 (branch s/level-0a-interactables-2)

- start (master 3d97ed08, 2026-11-01 ~11:52 UTC). Task: interactables_0700000a_live.rs has 7 tests;
  using interactables_0700000b_live.rs as the model, run the census of 0700000a [D every placement
  kind] and for every player-touchable kind add the live test that drives it through the real path
  (walk/E/shoot) and asserts the game's result, with a sister; fix any port gap the tests expose,
  with evidence. Setup reused from the prior session (repo, work/, ISO, nf-queue, toolchain present).
- The kind set (census per docs/levels/0700000a.md + the audit): 6 new per-kind live tests added —
  TouchOnce plates (0xEA, fire-once + the #757 early-approach arm), doors (0xDB, E-open + the
  keypad-locked refusal), keypad locks (0x2E, the #648 code menu through the snapshot KeyCodes
  [L 0x2beff8] → ch 48 → door #724 auto-opens), the PC kind (0x35, the K5 sister + the real
  gadget-cycle install + the decoy E arm unit-pinned — no standable floor near #684, live check),
  the keyboard SP (0xD9, the script's tick-15 entity + AND #741), the hatch LOAD_LEVEL (0xE8, the
  blocked-E message 0x2000021 + the real-chain load). Test-side fixes with evidence: the door's
  closed state reads bit 0x2; the locked-exit message's observable is the INFO pane's info_msgs
  (the same-tick flag→labels→bind pipeline); the hatch E stands on the plate's own volume centre
  (the settle pushes out of the plate's thin z band). No port gap surfaced — nf-game untouched.
  interactables_0700000a_live 13/13, walkreal_0700000a_live 7/0 (+3 pre-existing ignores),
  nf-game lib 635/0 (+5 ignores). Report: docs/reports/level-0a-interactables-2.md.
