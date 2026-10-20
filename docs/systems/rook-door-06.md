# The 07000006 truck driver — walk start and the mission route (rook-door-06-2)

The playtest symptom (2026-10-15/16): "rook gets stuck in the doorway on 06". Task 750b found
the port never even STARTS the walk — no drone moves after the ch-185 arrival plates. This doc
is the game's walk-start chain with the evidence; the fix and the test live on the branch.

## The driver's chain [M]

- **Family and start.** The driver (placement #494, skin 0x05000035, mode 0x17, team 3,
  switch channel 185) is the drone `Global` start dispatch sends to TruckDriverInit:
  `NDrone2_DSTATE_Global` 0x15b830 msg 1 — family byte `obj+0xc5 == 0x17` → state 0x32
  (the state table `NDrone2_StateFuncs` 0x29b300: 0x32 = `NDrone2_DSTATE_TruckDriverInit`,
  0x33 = InitAlert, 0x34 = Idle, 0x35 = Mission).
- **Init** (`NDrone2_DSTATE_TruckDriverInit` 0x160bd0 case 1): behaviour prop 0x1d non-zero →
  `NDrone2_AssignAIPath(dcvars, 8)` (0x153720 — the kind-8 mission path; the nearest path
  within 0.3 of the drone, no path → `+0x9f2` stays 0); the start anim script `+0x554`
  (placement param p14, `NDrone2_DefaultInit` 0x14b300:610-627) plays via
  `DroneAnim_SetScript(+0x554, 0x24, 0x11a, 0x1e, 0, 5, 0)` and the state HOLDS (the 880-frame
  sit script, [L] 05-truck.trace: he sits in 0x32 the whole window). The driver's route on the
  disc: kind-8 path 10, 13 nodes, east from the truck park (66.2, 28.9) → (84.9, 21.5)
  [D loader census, `rook_absent_06_live`].
- **The walk start = case 5** (0x160bd0, the anim script's done msg): `drone+0x9f2 != 0`
  (the route InitAIPath assigned) → **state 0x35 TruckDriverMission**; else → 0x34 Idle. This
  is the arm the port got wrong (hard-coded 0x34 — the 05-truck trace only ever showed 0x32,
  so the case-5 branch was never observed; the 70k-tick probe then showed no walk at all).
- **Mission** (`NDrone2_DSTATE_TruckDriverMission` 0x161030): ENTER clears the ready bits and
  plays the walk (`CallAnim(0, 0x37, …)`), +0xcc0/4/8 the BondTalk timer; UPDATE (case 3):
  `NDrone2_UpdateMissionRoute` 0x154a80 → `NDrone2_FollowRoute`; result 3 = arrived → prop 0x1d
  clears → 0x34; creep blocked (mode 9/0xc) → speed <0.66 stand idle else CallAnim(0x21); scare
  arms 0x11/0x12/0x15/0x16/0x1e → `DroneVision_EnemyAlerts`, 0x1f forced attack, 0x21 explosive,
  impacts resume 0x12. No msg 0x14 (SOUND) arm — gunfire sounds never move him (fix-06-driver).
- **The wake gate.** ch 185 (0xB9) rises from the truck-park TouchOnces #756/#757/#782 [D
  placements]. The driver is NOT in WaitSwitch at load: `Global` 0x15b830 reads the drone's
  switch channel (+0x134) AND second channel (+0x136) — with the second channel set the family
  check runs and 0x17 goes straight to 0x32 (the 05-truck trace's 0x32-at-load confirms [L]).
  The escort (#450, skin 0x0500002e, mode 0, team 1, switch 185) takes the PlayScript branch
  (`+0x554 != 0` → state 3) — never the driver states [M the same family gate].

## What the port does now

- `crates/nf-game/src/drone.rs`: `TRUCK_DRIVER_INIT, msg::ANIM_DONE` gates on the route slot
  (the port's `+0x9f2` equivalent — cf. the PARTY_GIRL_INIT comment) → `TRUCK_DRIVER_MISSION`;
  the 0x35 arms port ENTER/UPDATE/scare/impact 1:1 with 0x161030 (the mission route walk itself
  is the shared `update_mission_route`/`FollowRoute` machinery CivilianMission already used).
- Test: `crates/nf-view/tests/rook_door_06_walk_live.rs` — the real path (truck-ride carry →
  intro NIS → ch-185 plates → the sit script ends): the driver must enter 0x35 and walk >2
  units from his park spot within 12k ticks. Pre-fix run: parks in 0x34 at (66.12, 39.60, 28.90)
  forever, FAIL; post-fix: 0x35 at +577, walking at +679, PASS. Sister: the escort never enters
  0x35 (the family gate keeps the guard on his own chain).

## Where the walk goes — and the doorway question

With the fix, through the real path with Bond following (`rook_door_06_live` probe, 70k ticks):
the driver walks path 10 END TO END — (66.1, 39.2, 28.9) → (84.6, 35.0, 23.6), route idx 13/13
— and settles in 0x34 at the route end (the arrival arm). **He never touches a door** (path 10
has no flag-2 door nodes; no 0x81 hits in the whole window). Note: 750b's "0 movers" reading
was partly the probe's own mover filter — `moved` compared against `lastpos` snapshotted every
iteration, so a walker's ~0.04-0.08/iteration never crossed the 0.15 threshold; the position
census (start vs end) is the reliable signal and now shows the walk.

So the user's doorway stall is NOT the driver on his mission route. The remaining candidates,
in check order for the next session/live pass:

1. Which NPC does the user see stall — the escort (#12) or a scripted interior guard? The
   interior door-node pairs are path 7 n0–n1 → door #1 (36.50, 40.00, −16.25, the
   courtyard→castle entrance) and path 3 n11–n14 → door #4 (4.00, 49.00, −64.88, upper floor)
   [D probe dump]. Whoever walks THOSE routes (patrols, the escort after his script, an
   alarm-spawned guard) crosses them; the 0x81 door chain is ported (`drone.rs` route_reached_
   special + the 0x81 arms), with one documented deviation left: the route is frozen while
   parked in 0x81 (`drone.rs` ~4787) where the game's creep keeps walking (`LinkCreep_Handler`
   0x15a2c8 skips the blocked test while `+0xBA8` stands).
2. The escort's post-script chain: at watch end he sits in 0x4 (Idle) in the port; the game's
   PlayScript resume for a guard-family drone is ATTACK 0x56 (alt_state) — with no opponent he
   should land back on his start state (0x6 InitPatrol — no path → Idle). Needs the live check
   below before pinning anything.

## Live checks for the main agent

1. Fresh boot of 07000006, truck-ride entry: after the arrival cut-scene, does the DRIVER
   (skin 0x05000035) walk away east along the truck park and stop at the far (east) end —
   matching the port's path-10 walk? (Position over ~30 s vs (66.1,28.9) → (84.6,23.6).)
2. The user's stall: WHICH NPC stands in which doorway — the escort (skin 0x0500002e) at the
   courtyard door #1 (36.5, 40, −16.25)? Watch his state byte (0x81 expected if the door path
   engages), `Drone+0xBA8` (the bound door, non-null), the door's `obj+0xF4` (2/4/8/0x10).
3. Does the escort walk anywhere after his start script (with Bond unarmed at distance)? If he
   follows the driver east, the port's escort chain (PlayScript → resume) needs the same real-
   path treatment as the driver's.

## rook-door-06-3: the [L] walk proof and the driver-test re-derivation

The held merge (818's s/rook-door-06-2 onto master) failed 5 driver tests that pin
"ANIM_DONE → TruckDriverIdle (0x34)". Re-derived onto the game's own evidence — the pins were
written before the walk start was found and over-generalised one branch of case 5:

- **The gate's two halves [M].** Case 1 (0x160bd0) calls `AssignAIPath(8)` 0x154190 only when
  behaviour prop 0x1d ≠ 0 — and prop 0x1d IS set for this driver twice over: the placement's
  start script id 0x600011d hits `DoModeSettingsNEW` 0x14c600's arm (`setProperty(0x1d, 1)` +
  mode 0xc5 = 0x17), and `init_DMODE_TruckDriver` 0x14e348 sets it too. AssignAIPath hunts the
  closest node whose ROUTE flags (route+8, the disc AIPath kind at record+0x80 [M AIPath_Parse
  0x1d2598:24-33]) carry bit 3 (mask 8) within **0.3** 2D of obj+0x30; none → returns 0 with
  +0x9f2 still 0 (cleared at entry). On success `InitAIPath` 0x153720 walks the route's node
  chain filling +0x9f2 (≥ 1; a kind-8 route exits after one node unless the first node is a
  jump). Case 5 then: `*(short*)(drone+0x9f2) != 0` → 0x35, `== 0` → 0x34.
- **The game's driver walks [L].** `tests/traces/06-firstsight.trace` (a fresh-boot PCSX2
  recording on 07000006): the driver is IN 0x32 from frame 147176 at (66.12, 39.20, 28.89)
  (the sit, x span ~0.03, y the sit settle); at **frame 148014** (+838, the sit script's done)
  his state becomes **53 = 0x35 TruckDriverMission** and he walks CONTINUOUSLY — 66.65 → 67.94
  (148100) → 69.93 (148200) → 70.88, y 39.32 → 38.47 — until **frame 148247**, when with armed
  Bond 2.4 units away he goes to **0x13 CivilianHiding** (the Mission scare arms
  0x11/0x12/0x15/0x16/0x1e → EnemyAlerts [M 0x161030:75-89]; the Mission state has NO in-tick
  armed-person scan [M 0x161030 case 3]). A second witness for the scare chain from the sit:
  `tests/traces/06-searchlight.trace` shows a 0x13 drone holding the park spot (66.1, ~39,
  28.9) for 2850 samples (frames 140866..144629) — scared off the sit, cowering IN PLACE.
- **Conclusion.** The 06-2 port behaviour (route → Mission, walk, arrival → Idle [M 0x161030
  case 3 result 3]) is the game's. The 5 tests' "0x34 after the sit" pins predate the walk
  start and cited only "[M 0x160bd0 case 5]" — the +0x9f2-clear premise is false on 07000006.
  Re-derived (this branch, rook-door-06-3):
  - `fix_driver_air_live`: the post-sit state pin 0x34 → 0x35 ([L 148014]); the settle sister
    now measures on the walking driver (the settle gate is the hip's, not the state's [M
    0x140968 tail + 0x140e30-48]).
  - `fix_06_driver_live`: the gunfire test opens on 0x35; the cower test waits for the mission
    route's arrival (result 3 → prop 0x1d clears → 0x34 [M 0x161030]) and then runs the
    ORIGINAL Idle scare choreography at the route end (Idle's case 3 NearArmedPerson [M
    0x160ed0] — the Mission state must not be scared by proximity alone).
  - `fix_drone3_live`: the driver test asserts 0x35 at the sit's done, waits for the arrival,
    then the same Idle scare + the never-aims + grounded sisters.
  All 5 re-derived tests pass on this branch with the 06-2 drone.rs UNCHANGED
  — the merge held, the rook door still works on the real path (`rook_door_06_walk_live`
  green), and the driver's states are the game's.
