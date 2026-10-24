# WORKLOG

Entries appended at the end, one per task. Do not rewrite earlier entries.

---
Task ID: level-06-castle-fire
Agent: sandbox1
Task: 07000006 held/repeated fire: first press spawns the round but turns the view -1.38 rad from the aimed yaw; every later press spawns nothing (12 presses -> 1 bullet). Reproduce with real input, find the cause in the fire path [M Player_SetFiringAnim 0x1a3c70, Player_WeaponFiring 0x1a5460, Player_SetWeaponAnimObj 0x1a2e38] vs the port's Gun::update_with and the fire-script/state handling in crates/nf-view/src/scene.rs, fix, pin with tests that fail before the fix, un-ignore the two 06 walks.

Work Log:
- (started) Cloned repo, created branch s/level-06-castle-fire from origin/master at 26543436 (batch-1023j).

Stage Summary:
- Branch created; setup and investigation in progress.
