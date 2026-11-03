
## difficulty3-07000001-pins (started 2026-11-03 16:30 UTC)

Branch from origin/master (cbed0983). Task: pin everything difficulty 3 changes on 07000001 against
difficulties 1 and 2, each with its address — the extra Drone_Create 0x136be0:49 placements [D the
param per placement], the pain scale, the accuracy row, locks/pickups/ammo deltas [M] — in a new test
file difficulty3_07000001_live.rs (spawned set + stats at load, per difficulty). Fix any reader the
port is missing. walkreal_07000001_live untouched (level-01-agent-hall-fight-2 owns it). The escort-fight
and hall-fight reports read first. Runs: the new test, interactables_07000001_live, nf-game; the fast
runner in the background from the start.
