## level-09-audit (2026-10-26, sandbox expiring ~04:21 UTC)
Branch `s/level-09-audit` off origin/master (30598272). Task: the 07000009 behaviour audit — every
placed object/character vs the game: per placement, the creator in parsemap_create_dynamic_objects
0x1d04f0 / the static path, update + use/damage/event handlers [M], the port's coverage; the level's
scripts (streams, camera, hand-backs) and the mission records. Table to docs/levels/07000009-audit.md
(one row per placement/script/record, GAP column); small gaps fixed [M] with a test each, large gaps
as report sections. Tests pin only [M]/[D]/[L]-evidenced behaviour.
Plan: fresh sandbox — clone+branch pushed at start; work-slim + ISO downloads next; suite runner early;
placement census from the disc data + the kind→creator map from 0x1d04f0 + per-kind port grep; then
scripts/mission records; then fixes. Prior art: docs/levels/07000002-audit.md, 07000008-audit.md.

### level-09-audit — progress (2026-10-26 ~05:35 UTC)
Setup redone after a tool outage (sandbox survived: repo+work+ISO+builds intact). Research by three
subagents: the full 0x1d04f0 dispatcher map (asm cross-checked), the port per-kind coverage census,
and the 07000009 disc census (288 placements/23 kinds, all 11 NIS decoded, trigger/mux graph, the
mission record parsed from ACTION.ELF @0x2a4350 — a THIRD objective record exists on 09). Deliverables:
docs/levels/07000009-audit.md (the full table + scripts + mission record + GAP list), six doc errata
fixed in docs/levels/07000009.md (laser anim 0x06000667, lift SPs 0x0600007E/7D, hint 0x01000291,
Srceen #69, record 3, the stale ch-0x7C/0x7D note), and a new test interact_07000009_live::
a_ch7d_alarm_pull_raises_the_alarm_too (the un-pinned ch-0x7D side of alarm OR mux #278; 16/16 green).
Suite: 320 passed / 4 failed / 25 ignored on the tree's test content (3 allowed disc_golden +
campaign_flow_live TIMEOUT that passes in isolation 5/5 at 503.74 s); the final-tree re-run was cut
by the sandbox window after ~34 of 811 binaries (2-core box, 9.5-min foreground caps, bg killed).
Report: docs/reports/level-09-audit.md. Left: GAP-1/2/4 draw-only (render pass), GAP-3 own session.
