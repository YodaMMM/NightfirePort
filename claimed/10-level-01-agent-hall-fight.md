# level-01-agent-hall-fight — finish the 01 Agent real-health walk

Branch from origin/s/level-01-agent-escort-fight (merge origin/master into it first) and read docs/reports/level-01-agent-escort-fight.md. That branch restores walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent and survives the contact escort's first three fights (all changes gated difficulty >= 2 in the level's own test file). The walk still dies in the hall fight: last death tick 3366, the east stage walk into #6; the door trade at 6.8 u is the closest loss.

Do: win the hall fight on real input the way a player would (the report's weapon/grenade lever, approach angle, kill order). No health top-ups, no forced guard state, shared helpers stay opt-in. Done = the Agent walk green, every other walkreal_07000001_live test still green, every walkreal_*_live green, then the full suite total line.

---
claimed-by: sandbox4 at 2026-11-03T10:23Z
