# 50-real-health-00agent-01-hall-5

07000001's 00 Agent real-damage walk (`walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_00agent`).

The branch `s/real-health-00agent-01-hall-4` (its own report is on that branch) was cut from an old base (2265b403e).
Squashed onto current origin/master it keeps the d1/d2 walks green, but the 00 Agent walk dies EARLIER than master
alone: t3546 at (7.40, 5.99, 8.39), #6 (skin 0x5000036, 0x1c) at 6.6 u - master alone dies at t6054 at
(7.40, 6.47, 10.55). The report's measurements (the AND firing at 13831, the escort reached at 21.4) were taken on
the old base.

Task: start from current origin/master, merge `origin/s/real-health-00agent-01-hall-4` in (it merges cleanly; drop
its docs/WORKLOG.md and docs/reports/* from the result), then re-measure every lever on the merged tree and re-tune
until the 00 Agent walk gets at least as far as master alone, ideally past the AND gate. Every lever stays
gated difficulty >= 3; the d1/d2 walks must stay byte-identical. Push on `s/real-health-00agent-01-hall-5`.
Tests: walkreal_07000001_live, interactables_07000001_live.

---
claimed-by: sandbox3 at 2026-11-14T02:08Z
