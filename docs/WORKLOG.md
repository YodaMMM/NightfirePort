# Worklog

## level-12-every-objective (branch s/level-12-every-objective, from origin/master c0d3e92e)

Sandbox opened 00:51 UTC, expires ~02:50 UTC (stop new work 02:30). Scope: walk the WHOLE
07000012 mission from a fresh load on real input in walkreal_07000012_live.rs — every mission
objective [D records / Mission_MonitorObjectives 0x1827d8 M] in player order, each asserted
complete, ending on the level exit; blockers are port gaps to fix with [M/D] evidence.

Carried over from the previous session (same sandbox): env helper /home/z/my-project/env.sh
(PATH, NF_ISO, PKG_CONFIG_PATH + LIBRARY_PATH for the alsa link), background jobs die between
tool calls → suite via chunk_suite.py foreground chunks, stale test executables in
target/release/deps were purged once already (relink happens per chunk).

Plan: (1) read the existing test + list covered objectives; (2) decode the mission record from
work/ee_snapshot.bin; (3) docs/levels/07000012.md + prior branch intel; (4) legs for the rest on
real input; (5) suite chunks, report, finish.
