# WORKLOG

## real-damage-routes-08-11-14-03

- **Branch start** (sandbox5, resumed 2026-11-01 ~11:00Z): the claim's walker-side work was in the
  tree unpushed (no remote branch existed). Committed as the first checkpoint: real-damage wrappers
  for 03/08/11/14 (`walker::with_real_damage(...)`), the captor spares for 03, the shooter re-fight
  rule (a guard still shooting Bond is re-fought after 120 ticks instead of the 240 shield), the dry
  resupply after a fight, answer-shooters-during-weapon-cycle, and lenient restoration in
  `try_walk_to`/`walk_nav_to`. Next: build, run the four walkreal variants, iterate.
