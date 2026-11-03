# real-health-agent-difficulty-01-07-2 — the Agent-difficulty walks that fail on master

Branch from origin/s/real-health-agent-difficulty-01-07, merge origin/master into it, and read docs/reports/real-health-agent-difficulty-01-07.md. Agent = GameState 2 is pinned there.

On current master two of the new walks fail (the rest pass):
1. walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent — Bond dies at tick 4023 at (6.70, 6.45, 6.76); the last hit is drone #11 (skin 0x500003a, weapon 0xe, d 8.3, -7.00).
2. walkreal_07000006_live::walk_the_castle_interior_to_the_party_real_damage_agent — STUCK walking to "the doorway mouth": Bond at (28.57, 41.56, -19.16), target (27.90, 40.20, -22.40), door@(27.9,40.0,-23.0) state 0x2 pos01=0.50 (half open), 6000-tick cap.

Do: re-route each walk the way a player would (kill order, wait for the door to finish opening, approach), real input only, no health top-ups or forced state. Run both on a tree merged with the newest origin/master, every walkreal_*_live, then the full suite total line.
