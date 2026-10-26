# WORKLOG (task branches; append-only)

---
Task ID: 20-level-16-audit
Agent: level-16-audit worker
Task: 07000016 full behaviour audit — every placement/script/mission record against the game's code, table in docs/levels/07000016-audit.md, small gaps fixed with tests, large gaps documented.

Work Log:
- 09:46 UTC: branch s/level-16-audit created from origin/master; sandbox carries the level-15 setup (work/, ISO, nf-queue) — no re-setup needed. Background suite.py killed between tool calls (known); foreground per-crate runs are the substitute.

Stage Summary:
- Branch live; audit doc not yet written.

---
Task ID: 20-level-16-audit
Agent: level-16-audit worker
Task: 07000016 full behaviour audit — every placement/script/mission record against the game's code.

Work Log:
- 09:46 UTC: branch s/level-16-audit from master; sandbox reuses the level-15 setup; ALSA prefix rebuilt from the cached debs.
- 09:50-09:58: census re-run (707 all-map / 360 m63 rows, 25 kinds + 7 room refs); mission record 22 read byte-for-byte (0x2a46c0; objtable 0x2a42f0) — matches c07000016.rs::DEF; the level doc's "[L] record 16" corrected to 22.
- 09:58-10:30: dispatch 0x1d04f0 re-verified case-by-case + the 0x1d1b68 walker gate (0x31/FuseBox unreachable on this level); DynamicObject_Create/Update 0x18dbf8/0x18d3b8 decoded (the kind-0x3b channel-gated draw family); new dump_07000016_scripts probe — 14 scripts (07000016.bin) + 1 (07800016.bin); the only camera op is 06000631 (keys 0..114); 0600090a writes 240/241/242/243×2/0x50/0xb0..0xcd.
- 10:30-11:05: the 360-row audit doc written (docs/levels/07000016-audit.md); the 0x3b swap pinned by a new worldobj_live test (6/0); the level doc's stale 0x3b row/U-051/ch-0x50 corrected; GAP-16-1 (the per-tick draw swap) documented.
- 11:05-10:34 UTC: per-crate suites (the 100%-disk linker bus error cleared): nf-game lib 553/0/3 + tests green, nf-formats 3 allowed disc_golden fails, nf-audio green, nf-view touched tests 6/0.

Stage Summary:
- Deliverables: docs/levels/07000016-audit.md (360 rows + scripts + record 22), docs/reports/level-16-audit.md, the new worldobj_live test, the corrected docs/levels/07000016.md.
- Gaps: GAP-16-1 the kind-0x3b per-tick draw swap (Sim done + pinned; the scene hook is the view-side remainder); GAP-15-1/15-2 inherited; the 06000631 camera render check for the main agent; U-051's data side settled (0600090a writes 243 twice).
