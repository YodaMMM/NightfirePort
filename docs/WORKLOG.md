# WORKLOG

One entry per task, append only (never rewrite or reorder others' entries).

## mp-bot-scan-others

2026-11-07 04:12 UTC, branch `s/mp-bot-scan-others` (from origin/master 22d5836b7).

Task: make MP bots choose targets from the game's per-player scan — check whether
`BotBrain::scan_others` (port of `BOT_setOtherPlayerInfo` 0x125390) is called per tick, find what
actually fills `OtherPlayer::flags`/`dist2`, pin with a real-match LOS test (no target behind wall,
target in the open, no teammate in team games), run every mp_*_live + mp_world_sync_live + nf-game.

- [x] setup: cloned repo, branch created and pushed.
