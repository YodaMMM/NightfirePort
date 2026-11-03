# WORKLOG

## real-health-00agent-02-03-04 (2026-11-03)

Branch `s/real-health-00agent-02-03-04`, cut from origin/master 01917076 (the difficulty-01-07
walk changes were reverted on master by 2879088f "Revert the Agent-difficulty walks (01/06 red
on the batch)", so the 02/03/04 Agent sisters are restored here from e356a0ca and the 00 Agent
(difficulty 3) sisters added on top). Task: 00 Agent sisters of the Agent real-health walks for
levels 02/03/04 — same route, same end, `Drones::difficulty == 3` at load, real input only.
- [13:10 UTC] branch + WORKLOG; difficulty-3 pipeline verified in the port with the game
  addresses (damage_factor 1.2 on 01-04 [M Player_HandlePain 0x1902c8 + 0x1d3e00];
  hit_chance Hard 1.0 [M 0x179740:74-88]; Autoaim_HardMul 0 [M 0x1a73f8, tuning.rs:36];
  Drone_Create min-difficulty gate [M 0x136be0:49]; captain prop 8 at difficulty 3 [M 0x14b300:504-535]).
