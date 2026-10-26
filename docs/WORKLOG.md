# WORKLOG

## level-objects-uncited — branch start (2026-10-26)

- Setup: repo + work/ + ISO already in sandbox; created `s/level-objects-uncited` from origin/master, pushed this checkpoint.
- Sandbox note inherited from the previous session: background jobs are killed between tool calls here, so the
  fast suite runner (`tools/test/suite.py` via nohup) is expected to die silently; will attempt once and fall
  back to foreground crate suites.
- Task: for each of the 22 level-object functions never cited in crates/*/src
  (Copter_FireMissile, SpaceLaser_Register, Sunreflection_Update, Bubble_Update, camera_fixed, LS_LoadBonus,
  Sub_Laser, Sub_DeployTorpedoes, Sub_Update, Sub_ApplyDammage, Sub_Collide, Sub_Delete, MiniSub_FireTorpedo,
  MiniSubMove_Forward, MiniSubState_Attack, MiniSub_Update, MiniSub_Collide, MiniSub_Delete, Creature_Delete,
  Creature_Update, Creature_Hurt, Creature_Move): name creator/caller + reaching placement kind/script event +
  levels; port on-foot-reachable simulation with tests; driving-only/unreachable get a proof comment.
- Prior art to build on: docs/systems/uncited-audit.md (Creature_* dead on retail data — no type-0xe0
  placements [D census]; camera_fixed dead — no SetCamMode 4/5/6/0xa writer [M]), docs/systems/space-laser-update.md.
