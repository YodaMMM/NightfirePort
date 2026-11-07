# WORKLOG

One entry per task, append only (never rewrite or reorder others' entries).

## mp-bot-traits

2026-11-07 04:14 UTC, branch `s/mp-bot-traits` (from origin/master 22d5836b7).

Task: MP bots use their 360-degree vision and health regen traits — read every reader of the
bot-record flag byte (+0xac of the +0xd1c record copy): BOT_init 0x1249e8:165 (0x10 arms the
regen timer BOT_vars+0x738), NDrone2_FindOpponent 0x1424e0:288 (0x08 vision arm), and the other
sites (0x141fe8:146, 0x1458e0:43/49, 0x16e060:187/209/317/376, 0x194458:127); state each bit's
meaning, compare with the port, port the regen tick and the vision arm plus any missing bit,
list which Bot Setup bots carry which flag [L], pin with real-match tests (regen rate, behind-back
acquisition), run every mp_*_live + mp_world_sync_live + nf-game + full suite.

- [x] setup: repo/work/ISO reused from the sandbox, branch created and pushed.
