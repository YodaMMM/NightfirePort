# Stream batch: suspects, then the rest

Branch from `origin/s/stream-batch-3`, merge `origin/s/spawn-pick-walk-05` (reviewed, it also moves the stream) and current `origin/master` (the one-way door fix and 03 reroute are there). Push to `s/stream-batch-4`. Read docs/reports/stream-batch-3.md.

Real suspects first. Name the cause with a citation:
1. `start_corridor_guard_07000011_live` d2: guards land 6 hits then hold `lost_sight=true, seen_ticks=0` for 14000+ ticks. Idle Bond stops being attacked. Does the game do that? If unclear, say exactly what to read live.
2. 13 real_damage: a void fall inside the level, death t4169 at y -368.30. Compare with `10-void-recovery`; if that branch has landed, merge it.
3. 0c real_damage: a new red, dies t8216.

Already measured, so just fix the tests:
- `hold_npcs_live`: the unheld guard reads 0.003 at 600 ticks, 0.63 at 1200, 1.13 at 1800 and held stays 0.000. Widen the window to 1800.
- `fix_pickups_ammo_live`: 0xF5 does play at the box and is active; `sounds` is 1 before and 1 after, so `skip(sounds_before)` skips it. Search the list for the id at the box.

`guard_drops_flashbang_live`: the extra (6,6) recovered-sidearm drop looks game-law; cite it and fix the count.
Then the rest of stream-batch-3's list, plain walks first. The pin matrix in enemy_accuracy_live may be re-measured; say the harness is fixed-stream.

An ignore with a measured reason is allowed for 00 Agent walks only. Do not loosen pins tagged [L] and do not change game code to suit a walk.
