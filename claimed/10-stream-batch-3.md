# Stream batch: behaviour first

Branch from `origin/s/stream-batch-2`, merge current `origin/master`, push to `s/stream-batch-3`. Full suite on stream-batch-2 merged with master a6665d115: 4405 passed, 28 failed, 353 ignored.

These do not look like route drift. Treat each as a possible real regression until shown otherwise, and name the cause with a citation before touching the test:
- `hold_npcs_live`: "the unheld guard should move, moved 0.003". Guard arrival radii may have broken a guard's first move.
- `fix_pickups_ammo_live`: "the take plays 0xF5 at the box: []". A pickup sound did not play.
- `mayhew_hold_live`: "the corridor plate never raised ch 0x33".
- `fix_npc_doors_live`: "door #2 opened with Bond too close (2.2)".
- `guard_drops_flashbang_live`: "the deaths added NO pickup".
- `smg_close_range_live`: "pistol rounds at 6 u damaged no live guard".
- `walkreal_0700000d_live`: "the ~10 m fall is survivable", got 18.99.
- `walkreal_07000016_live`: the weapons assert at the chaser fight, on both the chain and the cold load.
- `walkreal_07000002_live`: "geisha 1 saved".
- `upward_shot_0700000c_live` x2, `enemy_accuracy_live` pin matrix, `start_corridor_guard_07000011_live`.
`goggles_draws_live` needs `--test-threads=1` here, so run it that way before calling it red.

Then the walks: 01, 06, 08, 0b, 0c, 0d, 11, 12, 13, 14, 15, 16. "walk_to ... 6000-tick cap" failures need a look at what stands there; a guard parked on a route is a behaviour question. Do the plain walks first.

An ignore with a measured reason is allowed for 00 Agent walks only. Do not loosen pins tagged [L] and do not change game code to suit a walk.

---
claimed-by: sandbox1 at 2026-11-06T16:13Z
