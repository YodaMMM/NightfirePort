# WORKLOG

One entry per task, append only (never rewrite or reorder others' entries).

## unread-params-drones

2026-11-07 09:10 UTC, branch `s/unread-params-drones` (from origin/master 22d5836b7).

Task: unread placement params for drone placements, spawners, AI volumes and path nodes.
For each class read its `*_Create` in the disassembly, list every placement param and word
it stores, find what the port does with each. Where the port ignores a param, census
non-zero values on every level (`Level::parse` + `nf_view::dev::LEVELS`): zero on every
level closes it with the census; non-zero on any on-foot level is a real gap — trace the
reader in the game, port it, test on the real level red on the old code. Inert claims must
be checked on all levels. Timing values come from fresh-load readings, not symbol names
(p6 scale is 30, not FRAME_RATE). No draw changes, no driving levels, nothing pinned
without a trace. Report: docs/reports/unread-params-drones.md with the census table.

- [x] setup: sandbox was reset (repo/work/ISO gone from the old path); re-cloned into
  /home/z/my-project/NightfirePort (persistent volume), branch cut from origin/master
  22d5836b7, WORKLOG started, pushing checkpoint before the downloads.
