
---
Task ID: level-08-audit
Agent: worker (s/level-08-audit)
Task: the 07000008 behaviour audit — one row per placement/script/mission record in docs/levels/07000008-audit.md (game [M/D] chain vs the port, GAP column); fix small gaps with tests; large gaps → report sections.

Work Log:
- Branch s/level-08-audit created from origin/master (30598272). Sandbox intact from the prior session (work/, ISO, ../nf-queue present). Master already carries the per-kind census (07000008.md §7, 308 placements) and the 996-level-08-decomp-audit entry sweep — my table is the per-placement expansion of those.
- docs/WORKLOG.md started; pushing this checkpoint before any other work.
- Next: suite.py in background; run census_07000008 (tests/census_07000008.rs --ignored) for the full per-placement dump; reuse the gen_audit kind knowledge base (05/06 method) for the per-row table; audit the level-bin ride script streams + mission record 3; small-gap fixes if they fit.

Stage Summary:
- Branch s/level-08-audit exists on origin at the clean master point; no code changed yet.
