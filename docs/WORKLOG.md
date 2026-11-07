# WORKLOG

One entry per task, append only (never rewrite or reorder others' entries).

## mp-human-pain

2026-11-07 04:38 UTC, branch `s/mp-human-pain` (from origin/master 22d5836b7).

Task: route every multiplayer hit (bullet, blast, melee) on a human and a bot through the
ported `Player_HandlePain` 0x1902c8 multiplayer arm (`player_pain_mp` / `bot_pain` in
crates/nf-game/src/mp/botweap.rs): compare scale, locational multipliers, the x3
`location_damage` option (+0x1BC row), armour-first, the 1.0 floor and pain types line by
line against the live master path; add the settings the doc comment says are missing;
damage deterministic across machines, no shared-Rand draws; pin through a real match
(armour first on an armoured player, body-part scaling with the option on, no scaling with
it off); run every mp_*_live + mp_world_sync_live + the net hash binaries + nf-game + full
suite.

- [x] setup: repo/work/ISO/queue worktree reused from the sandbox, branch created from
  origin/master 22d5836b7 and pushed.
