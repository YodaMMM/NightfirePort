
## rigid-body-remainder

- (13:50 UTC, 10-26) Branch created from origin/master (7e35bd77). Task: port the rigid-body
  functions the port never names (RB_CalcTorque 0x1d7658, RB_ApplyWind 0x1d7c78,
  RB_AddImpulsiveForce 0x1d7d40, RB_AddImpulsiveForceAtPoint 0x1d7d58, RB_ToWorld 0x1d7e18,
  RB_ToBody 0x1d7eb8, RB_apply_omega 0x1d7f50, RB_VecSpecial 0x1d8038; placement kind 0xD8,
  RB_Create 0x1d6bf0; RB_EulerEvolve noted unported in docs/models/misc.md) — find the callers and
  the on-foot levels that place 0xD8 bodies, port the update + the force calls that reach it,
  tests: rest stays put, a blast or a shot moves it along the game's integration. Sandbox kept
  repo/ISO/target/cargo/alsa from the bond session; work/ and nf-queue redone.
