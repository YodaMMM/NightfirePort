
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
