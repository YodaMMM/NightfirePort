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
- [15:20-16:40 UTC] sandbox reset, setup redone (clone, work-slim, ISO, rust, p7zip); branch
  resumed. The 02/03/04 Agent (d2) sisters restored byte-exact from e356a0ca (master's 2879088f
  had reverted them; the three files differ from e356a0ca only by the revert, verified by diff)
  and the three `_00agent` (difficulty 3) sisters added on top. Runs: 02 binary 3 passed + the
  d3 sister red; 03 binary 6 passed + the d3 sister red; 04 binary 10/10 (the d3 sister green
  as-loaded). The d3 rows re-verified in tuning.rs (Plr_DMod_Hard 0x3f99999a=1.2,
  DroneFiring_Accuracy_Hard 0x3f800000=1.0, DroneDamage_Hard 0x3f4ccccd=0.8, Autoaim_HardMul 0).
- [16:40 UTC] the 02 d3 tactics iterated 3x (the dropped-shotgun economy: scoop_drops,
  arm_weapon 0x1c, the per-round captor scoops, the arm_fullest verify re-cycle) — the run
  progressed from tick 6398 to the bunker door (health 27.6) before the courtyard rifle chips;
  parked red. The 03 d3 sister's blocker measured: drone #8 at 1 hp survived four 900-tick
  fights (killed false x4, no flank prints = the ally-blocked hold) and its 0x12 chips bled
  Bond out at tick 25850 [L the ledgers]. Both parked for the next session; report written.
