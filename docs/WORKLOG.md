## real-health-00agent-05-07 (started 2026-11-03 13:25 UTC)

Branch cut from origin/master 01917076. origin/s/real-health-agent-difficulty-01-07 is already an
ancestor of master (merge no-op), but master's 2879088f REVERTED its seven Agent sisters — the
05/07 Agent sisters are restored here from f1b37c21 (the two test files differ from that commit
only by the revert). Task: add the 00 Agent sisters (`..._00agent`, difficulty 3) of the 05/07
real-damage walks — the same route and end, `Drones::difficulty == 3` asserted at load, the
front-end's launch path [M C_SBNFDFCTY 0x216ca8:15]. Check the d3 rows: the pain scale
(Player_HandlePain 0x1902c8, Plr_DMod_*), the accuracy (DroneWeap_DoBulletAccuracy 0x179740:74-88),
the min-difficulty spawns (Drone_Create 0x136be0:49), any 00-only gates. Fix the port where it
differs; otherwise improve the walks' tactics. Real input only; shared helpers only behind a
per-level opt-in. Run walkreal_07000005/7_live and nf-game.
