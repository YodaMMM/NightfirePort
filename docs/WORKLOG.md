# WORKLOG

## guard-stuck-14-corner (2026-11-11)

- The 07000014 corner stall round: `NDrone2_DroneNearPos` 0x143e68's dropped radius pinned from
  the disassembly at the `LinkCreep_AtSpecialNode` 0x159dd0 call site — 0x3ecccccd (0.4) is
  loaded into `$f12` at 0x159eb8-0x159ec0 for the 0.4 compare and stays live at the
  `jal 0x143e68` (0x159ef8); the callee squares `$f12` at 0x143eac (`mul.s $f20, $f12, $f12`).
- The tier-2 (drone-list) check ported: `Drone::near_pos_fn` composes the player tier
  [M 0x143ec4-0x143f20] with the list tier [M 0x143f40-0x144000] over the `near_peers` snapshot
  `Drones::tick` lends every drone (the class-2 gate folded to the door.rs mapping). Wired into
  all five `Route::follow` call sites. Red proof on the old code (the old player-only closure),
  green pin `nav::tests::special_node_band_counts_a_peer_on_the_node`.
- The alarm-only 07000014 sweep still shows drone#0 frozen at the closed 0x1A leaf's west face
  (50.38, 52.34) — that case is the leaf-push order (move, then the DeltaP push), not the
  tier-2; measured raw step (0.040, 0.000), slide tangent zero. Left for the next round with
  the live checks (see the report).
