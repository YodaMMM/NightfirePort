# WORKLOG

### level-09-live-guards-walk (start, 04:47 UTC, 2026-10-30)

Task: level 09 (07000009) — `walkreal_07000009_live::walk_tower_a_start_to_finish` sets
`drones.hold_npcs = true` (line 76) so the guards never act. The level-09-every-objective
report left: a challenge-state guard took zero damage from the player's rounds. Root-cause
that ([M] the damage path; [D] the guard's records), fix the port side that is wrong, then
make the walk pass WITHOUT hold_npcs on real input (clear each patrol beat, resupply when
dry). Every test in the file green; run the touched binaries and quote their totals; full
suite via the fast runner, quote the total line. Branch from origin/master 814c8976.
