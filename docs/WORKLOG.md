# WORKLOG

## level-01-every-objective (2026-10-29)

Branch `s/level-01-every-objective`, task: make `crates/nf-view/tests/walkreal_07000001_live.rs`
walk the WHOLE mission 07000001 from a fresh load on real input, every objective [D] MissionData
/ [M] Mission_MonitorObjectives 0x1827d8 asserted complete in player order, ending on the level
exit firing.

- 03:57 UTC: sandbox intact (ISO + work/ + nf-queue). Branch created from origin/master 113dcdcd.
  Background suite starting next.

## level-01-every-objective — 06:45 UTC 2026-10-29
Sandbox reset mid-task; re-setup (clone, work-slim, ISO, user-prefix ALSA with pkg-config libdir fix, rustup). Merged origin/master (a213e841) so the suite runs on the final tree. Test fixes from d9c51ceb stand: walkreal_07000001_live pins obj0 announced-but-not-folded (fold belongs to 07000002 intro stream 7 [D 07800002 0x06000084]); walkreal_07000002_live asserts the cross-level fold after two settle ticks [M 0x1827d8 case 3]. Next: full suite on this tree, then report.

## level-01-every-objective — 08:25 UTC 2026-10-29
Final tree (merged origin/master at a213e841): walkreal_07000001_live 4 passed / 0 failed (33.1s), walkreal_07000002_live 1 passed / 0 failed (50.6s, cross-level obj0 fold pinned). Chunked suite (sandbox kills background jobs): 265/895 binaries, 975 passed / 0 failed / 89 ignored, zero real failures; mp_stress_explosives_live SIGKILL is a sandbox resource kill (reproduced standalone, 3.5G free). Remaining 630 binaries need ~10 chunk-hours — next session reruns tools/test/chunk_suite.py from scratch on this tree. Report: docs/reports/level-01-every-objective.md.

## level-01-every-objective-2 — 10:35 UTC 2026-10-29
Sandbox reset; fresh clone. Branch from origin/s/level-01-every-objective (f468735c); next: merge origin/master (1b2309ea), demote the `!channels.on(0x47)` assert in walkreal_07000001_live to an [S] unverified log (an assertion may not rest on a missing writer — the port may simply lack it), keep every other assert incl. the cross-level fold on the 02 walk, full suite, report.

## level-01-every-objective-2 — 13:35 UTC 2026-10-29
Final tree (merged origin/master 1b2309ea): the 0x47 never-raises assert demoted to an [S] unverified log (an assertion may not rest on a missing writer — the port may simply lack it; no production writer of 0x47 exists in crates/nf-game/src), every other assert kept, cross-level fold on the 02 walk stays. walkreal_07000001_live 4 passed (27.5s), walkreal_07000002_live 1 passed (52.2s) on the merged tree. FULL suite on this tree (chunked foreground — sandbox kills background jobs): **3454 passed / 20 failed / 213 ignored, 876/894 binaries**; all 20 environmental (3 disc_golden allowed, identity_disc slim-ISO, pylon ×3 no-GPU, 11 OOM CRASHes, 2 runner-cap TIMEOUTs incl. campaign_flow which passed solo at 570 last session); 18 slow walkreal_* binaries unrun. Report: docs/reports/level-01-every-objective-2.md.

## level-14-every-objective — 16:56 UTC 2026-10-29
Sandbox reset then restored; fresh clone from origin/master (f40be760). Task: `walkreal_07000014_live`
walks Countdown Part 1 to the Delta exit — list every objective the mission data lists [D / M 0x1827d8],
add the missing ones on real input (no teleports, no channel pokes), incl. the capture cut-scene path and
the sabotage cascade, ending on the exit. Setup (downloads, toolchain) running; this entry pushed first.

## level-14-every-objective — 18:12 UTC 2026-10-29
Census done: mission 0x07000014 part 0 lists 5 objectives (ch 0x47/0x48/0x4A/0x54/0xCE [D
c07000014.rs / ee_start_05 @ 0x2a4260]); the walk asserted 1-4 complete, obj5 had NO completion
assert. Added: obj5 announce (state 3) after Rook's death [M 0x1827d8 case 1] and the fold
(state 4) during the exit hold [M 0x1827d8 case 3]. Port gap found and fixed: the exit hold
skipped the whole sim, but Game_Run's Mission_Update sits at 0x1c9790:17 OUTSIDE the GS_IsPaused
gate (:30-38) — the game keeps folding objectives through the 300-frame fade. Fix:
Drones::tick_mission_during_hold on the scene hold paths. My first ch-0x64 assert contradicted
the decompile (the tail AND spans all three parts [M 0x1827d8:26]; D DEFs 07000015/16) — test
fixed to the negative pin. Capture (Kiko) path + sabotage cascade verified real-input in the
walk. Walk binary 5/5 green; mayhew transition walks 01 (4), 02 (1), 05 (9) green. Suite:
background killed (sandbox), foreground chunks 240/1/6 across 43/904 binaries (the 1 =
probe_pylon_render, known no-GPU); state /home/z/suite14_state.json for the next session.
