# WORKLOG

## level-11-strip-fight

Branch from origin/master 290dc4b03. Task 07000011: on 00 Agent the walkreal's stairwell pair
#33/#34 on the 23.0 strip — check the pair's engage and stand rules against the game (does the
game close a routed guard to contact distance and fire there, or stand at its range?) and fix
the port if they differ. Evidence first: the decompile's combat chain (DroneFunc_CombatState
0x148338, NDrone2_MoveToObject, the level's engage/combat overrides [M 0x14b300]), then a
real-input measurement on the walked descent. No draw changes.
Report: docs/reports/level-11-strip-fight.md.
