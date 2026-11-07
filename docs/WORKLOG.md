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
- [x] audit: all 8 flag-byte sites read and stated (bits 0-4; the table is in the report).
- [x] port: the regen +5.0/s heal + clamp (0x16e060:225-233, was a deadline-only stub), the
  360 fast-ack (:186-218), find_opponent's wide-arm completion (0x1424e0:364-366 — the seen-bit
  and lazy-sight exemptions), the shooter damage perk ×1.5/×1.25 (0x1458e0:43-56 /
  0x194458:120-140) at both scene.rs bot-shot sites.
- [x] tests: 5 new bot_trait_* nf-game lib tests (regen rate/clamp/no-bit, behind-back
  acquisition + range, fast-ack, damage truth table) — all pass; nf-game lib 689/2/5 (the 2
  fell_out failures are pre-existing on master, reproduced in a scratch worktree).
- [x] mp binaries whole: mp_bots_live 3/0, mp_bots_moving_live 3/0, mp_bot_weapons_live 2/0,
  mp_bot_objectives_live 10/0, mp_world_sync_live 8/0, mp_perf_16_live 2/0, mp_16_slots_live 3/0.
- [ ] full suite: 24/1142 binaries green when the window cut it (sandbox disk + background-job
  kills ate the middle of the session; report "What's left" #1).
- [x] finish: report docs/reports/mp-bot-traits.md pushed (118373c51).
