
---
Task ID: level-0d-audit
Agent: worker (s/level-0d-audit)
Task: the 0700000d behaviour audit — one row per placement/script/mission record in docs/levels/0700000d-audit.md (game [M/D] chain vs the port: creator in parsemap_create_dynamic_objects 0x1d04f0 / static path, update, use/damage/event handlers, and the port's created?/updated?/drawn?/usable?/reacts?/sounds? answers), GAP column; fix small gaps [M] with a test each; large gaps → report sections.

Work Log:
- Branch s/level-0d-audit created from origin/master (30598272) at 04:43 UTC; deadline ~06:42 UTC. Sandbox intact from the prior level-08-audit session: work/, Nightfire-onfoot.iso, target/ build cache, ../nf-queue all present (skipped setup downloads).
- docs/WORKLOG.md started; pushing this checkpoint before any other work.
- The just-finished origin/s/level-08-audit (04:42 UTC) used gen_audit08.py + census tests for the same audit shape on 07000008 — checking it for reusable kind knowledge and per-row method.
- Next: suite.py in background; find the 0700000d map-bin census/port audit coverage; per-placement table; scripts (streams, camera, hand-backs) + mission records; small-gap fixes if they fit.

Stage Summary:
- Branch s/level-0d-audit exists on origin at the clean master point; no code changed yet.

---
Task ID: level-0d-audit
Agent: worker (s/level-0d-audit)
Task: (final) the 568-row per-placement audit written; the scripts + mission records decoded; the pin tests added.

Work Log:
- census_0700000d.rs (nf-view test, --ignored prints): the 568-placement/34-kind dump [D] + the 14-script stream decode across 0700000d/0780000d/0790000d; audit_0700000d.rs (2 green): the kind histogram + wait-web/leaf-pair/death-ch pins [D] and record 12 vs work/ee_start_05.bin [L].
- docs/levels/0700000d-audit.md: per-kind Game[0x1d04f0 dispatch + control_funcs @0x26f890]/Port(file:line)/GAP headers + one row per placement; the scripts section (no channel writes anywhere [D]; the owner map: 3 SP-owned, 9 code-spawned effect NISes); the mission section.
- Corrections: record 12 is @0x2a4530 (not 0x2a44a0); 0x1827d8:98-108 (not :128-141); Player_CollWire 0x197130; the census skins; Player_Activate is 0x1a5170.
- Big gaps for their own tasks: 0xD8 rigid bodies (math-only in the port), 0xE7 ThirdCams (data-only), the 0x71 rotor (visual, needs a render), the effect-NIS particle stand-in.
- Suite: the background run (04:52) partial 242 binaries / 1547 passed / 4 known-slim-ISO failed at 06:12 UTC, still running; the two new binaries green standalone.

Stage Summary:
- Branch s/level-0d-audit = master 30598272 + the audit docs + 2 test files + the level-doc corrections; the report is docs/reports/level-0d-audit.md.
