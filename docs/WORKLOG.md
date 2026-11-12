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

## guard-leaf-press-07 (2026-11-11T14:26Z, branch s/guard-leaf-press-07) — DONE

- Base: origin/s/guard-leaf-pushout 3a660d01c (origin/master already an ancestor — merge no-op).
  Fresh sandbox: rustup + the ALSA 1.2.8 user prefix rebuilt, work/ + the ISO re-downloaded
  (369823203 → 1061754880).
- Item 1 (the press gate grows): the leaf-press arm (`move_by`'s `leaf_press_rest` branch) now
  covers 07000002/04/07/09/0a/0b/0d/13/14/15/16/1b/41 — every level added one at a time with its
  whole walkreal binary re-run on the added tree, all green: 07 6/0/0, 04 9/0/1i, 09 9/0/0,
  41 15/0/0, 0d 1/0/2i, 15 6/0/3i, 0a 13/0/2i, 0b 9/0/0, 13 3/0/1i, 1b 16/0/3i, 16 15/0/3i,
  02 4/0/1i (the pre-existing ledgers only; no shift anywhere). 07000014's walks re-run on the
  final tree: 7/0/3i.
- Item 2 (the door-side pin): `Doors::contact_centres(i)` accessor added (the obj+0xD0 list as
  `Door_ReactToHit` 0x135980 reads it), and the 07000014 press test now asserts, during the
  press ticks, that door 0x1A's (list index 26) contacts carry the pressing guard (centre = his
  root pos [M the DoorMover build], non-player). Green in
  guard_stuck_sweep_live: 3/0/0.
- The obligated binaries on the final tree: nf-game lib 783/0/4i, npc_doors_live 6/0/1i,
  fix_npc_doors_live 1/0/0, fix_guard_misc_live 6/0/0, ai_traces_live 4/0/0,
  stealth_ai_live 10/0/0. Real input only; no damage/health changes; no shared-helper changes.
- The full suite did not fit (the background fast runner is disk-hostile in this 9.9 GB sandbox
  — it filled the disk in the previous window; per-binary foreground chunks are what ran).
- Report: docs/reports/guard-leaf-press-07.md.

## guard-leaf-press-2 (2026-11-12T03:05Z, branch s/guard-leaf-press-2) — in progress

- Base: origin/s/guard-leaf-press-07 758a89ddc, merged origin/master 08a276ce0 (clean; 16 files, 1049 insertions). Scope: the merged tree turns fix_mission_live::the_07000001_exit_works_end_to_end red (master alone passes). Find the leaf-press interaction that blocks the 07000001 courtyard/exit route, fix it the game's way [M Drone_CollisionHandler 0x140968:204-208], re-run every walkreal_*_live one level per tool call, then the full suite.
