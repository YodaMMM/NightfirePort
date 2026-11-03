# real-health-agent-difficulty-01-07 — finish levels 01-07 on real health at Agent

Branch from origin/master.

The real-health walks (walk_..._real_damage in walkreal_070000{01..07}_live) run at the default difficulty (Easy: Plr_DMod_Easy 0.4 damage scale). A player on the next difficulty up takes more damage per hit and faces the difficulty's stat changes.

Do:
1. Add an Agent-difficulty sister of each real-health walk for 01, 02, 03, 04, 05, 06, 07 (set the sim's difficulty the way the menu does at level start; real input only, no health writes).
2. Where Bond dies, read the damage log; the port must match the game's per-difficulty damage and stats [M the difficulty multipliers and the stats the difficulty changes]; fix the port where it differs, otherwise improve the walk's tactics (cover, pickups, kill order).
3. Every new sister must pass. Run every walkreal_*_live, then the full suite.

Rules: shared test helper changes (crates/nf-view/tests/common/) must be opt-in per level (the code must check the flag); re-run every walkreal_*_live before finishing. A visible fix ships with a real-input test that reaches the spot from the level start.

---
claimed-by: sandbox3 at 2026-11-03T01:14Z
