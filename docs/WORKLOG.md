
## rigid-body-remainder

- (13:50 UTC, 10-26) Branch created from origin/master (7e35bd77). Task: port the rigid-body
  functions the port never names (RB_CalcTorque 0x1d7658, RB_ApplyWind 0x1d7c78,
  RB_AddImpulsiveForce 0x1d7d40, RB_AddImpulsiveForceAtPoint 0x1d7d58, RB_ToWorld 0x1d7e18,
  RB_ToBody 0x1d7eb8, RB_apply_omega 0x1d7f50, RB_VecSpecial 0x1d8038; placement kind 0xD8,
  RB_Create 0x1d6bf0; RB_EulerEvolve noted unported in docs/models/misc.md) — find the callers and
  the on-foot levels that place 0xD8 bodies, port the update + the force calls that reach it,
  tests: rest stays put, a blast or a shot moves it along the game's integration. Sandbox kept
  repo/ISO/target/cargo/alsa from the bond session; work/ and nf-queue redone.

- (15:18 UTC, 10-26) rigid-body-remainder DONE this session: the evolution core ported into
  crates/nf-game/src/rigid_body.rs (RbState + RB_AddImpulsiveForce/AtPoint/ToWorld/ToBody/
  VecSpecial/apply_omega/ApplyImpulses/ApplyWind/EulerEvolve + RB_Update's heat-puff impulse +
  the class-5 explosion arm of RestingContact), every function [M]-cited with disassembly where
  the C was VU-opaque; the (w,x,y,z) quat layout caught by the test. 11/11 module tests green.
  Callers + the 7 census levels [D] + reachability in the report; remainder = the dispatch
  wiring, the contact trio (RestingContact/Rot/CalcTorque), the up-righting tail, Env_GetWindVel,
  the full suite run. Report: docs/reports/rigid-body-remainder.md; finish.sh OK; ntfy via wget
  (curl timed out, urllib network-unreachable).
