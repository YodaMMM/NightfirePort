# level-12-corridor-guards-stuck — guards stuck at the security-center corridor

Branch from origin/master.

On 07000012, from Bond at (22.49, 1.09, 19.16) facing yaw −1.56 at the start ("Activate elevator override control in security center"), the two guards at the far end of the corridor (by the exit sign, near the dropped case) stay bunched in place and walk into nothing: they look stuck on an invisible wall instead of coming down the corridor.

Do:
1. Reproduce headless from that spot: log both guards' positions, AI states and the collision hits that stop them over the first ~600 ticks.
2. Name what blocks them: a static/world run (which object or room piece), a door or panel collider, the drone-drone push, or a path/node choice. Compare against the game's code for that piece [M] and the level data [D].
3. Fix the port where it differs from the game; cite the decompile. Add a live test: both guards leave their start spots and reach Bond's corridor within the game's timing.
4. Run walkreal_07000012_live, interactables_07000012_live, every stuck-guard test, then the full suite.

---
claimed-by: sandbox7 at 2026-11-02T07:11Z
