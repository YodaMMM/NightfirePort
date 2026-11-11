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

## guard-leaf-pushout (2026-11-11)

- Continuing guard-stuck-14-corner's "What is left" item 1: the 07000014 drone#0 GO_TO_GOAL
  freeze at (50.38, 1.00, 52.34) against the closed 0x1A leaf's west face. The port's
  probe_move rejects the step every tick; the game moves first and then pushes out in the
  contact pass [M 0x140968:204-208, Collide_Intersect 0x1ebaa8:353-373]. Porting the game's
  order (move, then the DeltaP push-out), keeping the Rand count.
- The leaf press ported: `Drone::leaf_press_rest` — the game's order (move first, then the
  contact pass's DeltaP push-out [M 0x140968:204-208 + Collide_Intersect 0x1ebaa8:353-373]) for
  the pure-face leaf press: the body rests IN CONTACT (the collide radius 0.4 [M 0x140e90:91-93]
  touching the face) instead of the old 0.435-out reject standstill. Gated to 07000014 (the
  door_leaf_slide set-growth precedent). Facts pinned on the way: the doorway is a double SLIDE
  door (kind 2, both leaves hinged (50.82, 1.19, 52.92); 0x1A covers z 51.66..53.00, 0x19
  52.84..54.18, [L the posed-tri AABB dump]); `Door_ReactToHit` 0x135980's kind-2 arm only
  reverses a CLOSING leaf (a closed leaf ignores presses — same as the port); the door opens
  only via a route door-node reach (the special band: dist < 0.4, or 0.4-1.0 with a peer on the
  node [M LinkCreep_AtSpecialNode 0x159dd0]) or the player's Door_Activate — the pressed guard
  (dd 1.13 > 1.0 [L]) cannot reach the band, so the no-input freeze is the GAME's own behavior
  too; with the door opened the guard flows through and presses his alarm (state 0x8a
  PressAlarm [M 0x168ea8]) — the goal (56.06, 0.01, 42.91) is the alarm node [L the flow run].

## guard-leaf-press-07 (2026-11-11T13:05Z, branch s/guard-leaf-press-07)

- Claimed. Base: origin/s/guard-leaf-pushout 3a660d01c (origin/master already an ancestor —
  merge no-op). Task: extend the leaf-press law's gate from 07000014 to 07000007 (the slide arm
  already runs there; the press arm extends the same scan), re-run 07000007's walks and repair
  any shift gated to its level; pin the door side (during the press, doors.collide_step sees the
  pressing guard in its contacts, as Door_ReactToHit reads them [M 0x135980]); if both hold, add
  each further on-foot level the same way, one level per tool call, walks re-run. Real input
  only, never lower damage or health. Plan: checkpoint push → setup (fresh sandbox: downloads,
  env) → read guard-leaf-pushout items 1-2 → the 07 press + walks → the door-side pin → further
  levels → obligated binaries → report.
