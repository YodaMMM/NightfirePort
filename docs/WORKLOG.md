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

Progress (02:15 UTC): the coverage list first — 07000012's mission data lists EXACTLY TWO
objective records, decoded from the EE snapshot [L ee_snapshot.bin 0x2a4350 + row 16*40 (level
07000012, mission 07000011, part 1) → objective table 0x2a41b8, count 2]: obj0 0x04000037
"Activate elevator override control in security center" (ch 0x49, enable 0x48) and obj1
0x04000038 "Escape through the elevator shaft" (ch 0x4D, enable 0x49). The existing
walkreal_07000012_live already covers BOTH in player order (spawn ch 0x48 → obj0 announced
state 3 → the console E → obj0 complete state 4 → obj1 announced → the REAL 0xE8 exit fires into
0700004A → its spawn touch #277 writes ch 0x4D → obj1 complete state 4, obj0 still 4). Added: the
record-count pin + mission-row helper in the test (assert LEVELS row 16, 2 records [L]). Green:
walkreal_07000012_live 1 passed (5.4 s), walk_07000012_live 4 passed (6.9 s),
scripts_07000012_live 2 passed (2.9 s).

The two documented stand-ins remain (both are position pokes past collision funnels): the door[19]
doorway pincer and the north-wall funnel pinch. This session probed the funnel for a real-input
crossing (probe12_funnel_realinput.rs, NOT committed — it fails by design and its findings are
here): driving east along the south face from the lift-call button pins at bond (4.94,1.04,7.66)
against unmapped geometry 1.35 from the opening mouth, before the measured pinch is even reached.
Both blockers are the SAME port gap — the game resolves the cylinder move ALONG the move
(Intersect_CylGeom 0x109da0) and its walk push tail is DEAD ([M 0x109da0:447's guard on the
DeltaP accumulator; L ee_snapshot: it drifts ~15 m and boots (0,0,0)]) while the port applies a
two-sided depenetration push (capsule_push) that pins the capsule; the game-accurate fix is a
move-clip walk model + a full walkreal battery re-run (the NF_WALK_GHOST experiment exists,
default OFF). Left for the next session; the live checks from the prior task stand.

Suite (fast runner, chunked foreground — background dies between tool calls here): state seeded
from the same-master-tree l04 run (203 binaries, byte-identical builds; walkreal_07000012_live
dropped and re-run fresh), then chunks on THIS tree to **788/888 binaries recorded — 3267 passed
/ 7 failed / 168 ignored** (02:11 UTC). All 7 failures are environment-only: disc_golden's 3
(the suite rules' allowed slim-ISO trio), identity_disc (pins the FULL retail disc 1,191,072
sectors; the sandbox's on-foot image has 518,435), and fix_0c_pylon_live (2) + probe_pylon_render
(1) (wgpu adapter NotFound — the headless sandbox has no VULKAN/GL device). 99 live-tail binaries
remain unrun (window exhausted).
