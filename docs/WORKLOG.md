# WORKLOG (branch running copy)

## glass-pass-every-level-2 (started 2026-11-04 12:56 UTC)

Branch `s/glass-pass-every-level-2` cut from origin/s/glass-pass-every-level == origin/master
(8deea246 — round one is already absorbed; the prior report read from the branch history).
Task: land the per-level guard-through-glass pins (06 pane 376, 08 panes 170/176/177, 14
windows 97/86/156) and the charger-impact half of the 01 pin (the freshly-woken waiter's
pickable gate, wake+7..11 measured, [M Collide_PickObj 0x1eae80:26-30 drops obj+0xF0 bit
0x20; NDrone2_Enable(0) sets it]). A pane must break AND the round must reach the guard;
fists and laser stay stopped. Fix any level where that fails. Every walkreal_*_live; the
full suite total line.

Progress:
- [12:58 UTC] Branch + WORKLOG started; the prior report's leads (the recon coords, the
  walker-side shoot_guard_through_panes fn, the ignored 14 stand-walk test) are the map.
- [13:20-14:16 UTC] The per-level pins. 06: the walked route reaches the court; the pane 376
  filter 0xff07 [D, printed] drops the pistol's 0x0008 word by [M Collide_FilterBullets
  0x1ece30] — the pane is smash-only by the game's own law; the world line also blocks at
  68.8% (the frame). Ignored with the reason. 08: the crossing stand (105.0, 35.0, −124.5)
  measured on the recon coords; the spawn nav does not connect — needs the walkreal-08 hall
  legs; ignored with the geometry. 14: the corridor stands measured ready (#5 1 u and #2 16 u
  inside pane 97's crossing from (22.9, 1.2, 33.3)); the nav walk cannot cross the drops from
  the spawn mezzanine; ignored with the route note. The 01 impact half: the blocker is the
  raw step_bond harness — the NPC globals never advance, and [M 0x140968:62] (the NIS freeze)
  makes the guards undamageable by the game's own law inside the harness; the taps + the
  plain post-smash path stay in the test; the full-scene-step drive is the next session's
  opener. The recon example now prints the spawn + the guard coords. The binary whole:
  3 passed / 0 failed / 3 ignored. The suite + the walkreal sweep do not fit this window
  (no src changes — the branch is test/example/docs only).
