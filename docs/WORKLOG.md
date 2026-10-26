# WORKLOG (task branches; append-only)

---
Task ID: 20-level-15-audit
Agent: level-15-audit worker
Task: 07000015 full behaviour audit — every placement/script/mission record against the game's code, table in docs/levels/07000015-audit.md, small gaps fixed with tests, large gaps documented.

Work Log:
- 08:24 UTC: sandbox reset — fresh clone of master (b604242f), branch s/level-15-audit created, worklog started. Setup (work/, ISO) in progress.

Stage Summary:
- Branch live; audit doc not yet written.

---
Task ID: 20-level-15-audit
Agent: level-15-audit worker
Task: 07000015 full behaviour audit — every placement/script/mission record against the game's code.

Work Log:
- 08:24 UTC: fresh clone (reset sandbox), branch s/level-15-audit from master b604242f, worklog pushed.
- 08:31-08:54: setup — work-slim + ISO extracted and verified, nf-queue worktree, rustup, ALSA user-prefix (patched alsa.pc prefix; pkg-config suppresses system -L flags).
- 09:00-09:10: creator dispatch 0x1d04f0 re-verified case-by-case; census re-run (968 all-map / 638 m54 placements, 37 kinds); new dump_07000015_scripts probe added.
- 09:10-09:18: 638-row per-placement audit doc written (docs/levels/07000015-audit.md) + scripts (5 shared blast scripts, no cameras) + mission record 21 verified byte-for-byte vs c07000015.rs::DEF.
- 09:18-09:38: docs/levels/07000015.md §6 stale keypad row corrected; report written; tests: nf-game 896/0/6, nf-formats 105/3 (the 3 allowed slim-ISO disc_golden), nf-audio 28/0, nf-view 171/0/2 (killed mid-run by the tool-call limit; the touched census binary green).

Stage Summary:
- Deliverables: docs/levels/07000015-audit.md (638 rows + scripts + records), docs/reports/level-15-audit.md.
- Gaps: GAP-15-1 the blast debris arm (create_debris has no caller; needs Rand_Rand order pin), GAP-15-2 through-Bond bump (shared with level-13). No other data-side gaps: no civilians, 0xc8 doors dead in both, no camera scripts, the "console" is a static mesh.
