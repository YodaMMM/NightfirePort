# WORKLOG

## real-health-agent-difficulty-01-07 (2026-11-03)

Task: finish levels 01-07 on real health at Agent difficulty. The real-health walks
(walk_..._real_damage in walkreal_070000{01..07}_live) run at the default difficulty
(Easy: Plr_DMod_Easy 0.4 damage scale). Do: (1) add an Agent-difficulty sister of each
real-health walk for 01..07 — set the sim's difficulty the way the menu does at level
start, real input only, no health writes; (2) where Bond dies, read the damage log; the
port must match the game's per-difficulty damage and stats [M the difficulty multipliers
and the stats the difficulty changes]; fix the port where it differs, otherwise improve
the walk's tactics (cover, pickups, kill order); (3) every new sister must pass; run
every walkreal_*_live, then the full suite. Shared helper changes opt-in per level.

Branch cut from origin/master 6f16146e. Sandbox carried from the previous session (repo,
work/, ISO present); no setup redo needed.
