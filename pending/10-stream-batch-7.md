# Stream batch: pins that were one roll, then the walks

Branch from `origin/s/stream-batch-6`, merge current `origin/master`, push to `s/stream-batch-7`. Read docs/reports/stream-batch-6.md.

Decision on the 16 stale pins (c4b2d38ce, f1f37ba47): the draws are the game's, keep them. A test whose number depends on one position of the shared stream pins a roll, not a rule; a live trace of one run does not make that exact value a law. For each of those tests, state the rule from the decompile and assert the rule: the ranges, the order, what must and must not happen. Keep the live trace cited as an example. This permission covers only those 16 tests and only stream-position values; say for each what the rule is and what the old value was. A pin on geometry, damage or a fixed constant stays exact.

spawner_kill: fix the 7 tests to the whole-template draw, which can spawn nothing.

Then: the real-damage, Agent and 00 Agent walks, 04's stats tests, 14's tripbomb wedge, escort_follow, abseil, goggles, upward_shot. An ignore with a measured reason is allowed for 00 Agent walks only. Finish with a list of what is still red by name.
