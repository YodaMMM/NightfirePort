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
- 10:12 UTC setup finished after full redo (sandbox was reset: repo/work/ISO gone, rootfs
  disk quota full — repo now at /home/z/my-project/NightfirePort, cargo home+target on
  /tmp/my-project PolarFS, ISO extracted there, ALSA pc via user prefix). CENSUS FIRST:
- 10:30 UTC census run over all 34 archives (crates/nf-view/tests/census_unread_params.rs,
  assertion-free, ignored probe; 6.7 s). Placements: 0x0f 588, 0xf1 112, 0xf5 257, 0xf8 33.
  Ignored-param verdicts: 0xf1 spawner params 0-8 all read by port — closed. 0xf8: every
  param zero on every level — closed. 0xf5 p3/p4/p5 zero-everywhere — closed; 0xf5 p1 only
  value 4 (the handled lift case); 0xf5 p6 nonzero on 217 records but Drone_AIVolume_Create
  reads only level_tag+0x2c..+0x3c (params 0-4) [M 0x1376c0:76-87] — game-ignored, closed.
  OPEN: 0xf5 param 2 (stored DI+0x28 [M 0x1376c0:85]) nonzero on 203 records / 12 levels,
  reader not yet traced. 0x0f param 7: nonzero 15 records / 5 levels; game stores DI+0x148
  [M 0x136be0:83], NDrone2_DefaultInit writes it too [M 0x14b300:473],
  BOTWEAP_InitWeapon zeroes the byte [M 0x129e48:37]; decomp grep found no reader — looks
  write-only, port ignoring matches; needs the report's Live check. 0x0f param 22 sits in
  the stats-tail region (max 0x005b0003 mirrors param 17's count word) — confirm the stats
  machinery consumes it. AI paths: params[8] all zero — closed; node disc words +0x1c.. are
  load-time slots (in-memory node keeps +0x04) — closed; path extra area present on 49
  paths / 28 levels (parser only skips it [M 0x1d2598]) — reader unknown, follow-up; link
  +10 flags 0x20/0x40 on 14,496 links / 26 levels — port folds them into node->door
  binding (nav.rs), behavior re-verify left.
- 10:45 UTC freeze: committing the census + report; no code change shipped (the two open
  gaps need reader traces first — nothing pinned without a trace).
