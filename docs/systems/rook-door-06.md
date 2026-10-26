# The 07000006 truck driver — walk start and the mission route

Symptom (2026-10-15/16): an NPC (Rook) stuck in a doorway on 06. The first finding: the port never
even STARTED the driver's walk — no drone moved after the ch-185 arrival plates. This doc is the game's
walk-start chain with the evidence; the fix is in drone.rs and `rook_door_06_walk_live`.

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

So the doorway stall is NOT the driver on his mission route. The remaining candidates,
in check order:

1. Which NPC stalls — the escort (#12) or a scripted interior guard? The
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

## Live checks (open)

1. Fresh boot of 07000006, truck-ride entry: after the arrival cut-scene, does the DRIVER
   (skin 0x05000035) walk away east along the truck park and stop at the far (east) end —
   matching the port's path-10 walk? (Position over ~30 s vs (66.1,28.9) → (84.6,23.6).)
2. The stall: WHICH NPC stands in which doorway — the escort (skin 0x0500002e) at the
   courtyard door #1 (36.5, 40, −16.25)? Watch his state byte (0x81 expected if the door path
   engages), `Drone+0xBA8` (the bound door, non-null), the door's `obj+0xF4` (2/4/8/0x10).
3. Does the escort walk anywhere after his start script (with Bond unarmed at distance)? If he
   follows the driver east, the port's escort chain (PlayScript → resume) needs the same real-
   path treatment as the driver's.

## The [L] walk proof and the driver-test re-derivation

With the walk start ported, 5 driver tests that pinned
"ANIM_DONE → TruckDriverIdle (0x34)" failed. Re-derived onto the game's own evidence — the pins were
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
- **Conclusion.** The ported behaviour (route → Mission, walk, arrival → Idle [M 0x161030
  case 3 result 3]) is the game's. The 5 tests' "0x34 after the sit" pins predate the walk
  start and cited only "[M 0x160bd0 case 5]" — the +0x9f2-clear premise is false on 07000006.
  Re-derived:
  - `fix_driver_air_live`: the post-sit state pin 0x34 → 0x35 ([L 148014]); the settle sister
    now measures on the walking driver (the settle gate is the hip's, not the state's [M
    0x140968 tail + 0x140e30-48]).
  - `fix_06_driver_live`: the gunfire test opens on 0x35; the cower test waits for the mission
    route's arrival (result 3 → prop 0x1d clears → 0x34 [M 0x161030]) and then runs the
    ORIGINAL Idle scare choreography at the route end (Idle's case 3 NearArmedPerson [M
    0x160ed0] — the Mission state must not be scared by proximity alone).
  - `fix_drone3_live`: the driver test asserts 0x35 at the sit's done, waits for the arrival,
    then the same Idle scare + the never-aims + grounded sisters.
  All 5 re-derived tests pass with the walk-start drone.rs unchanged
  — the rook door still works on the real path (`rook_door_06_walk_live`
  green), and the driver's states are the game's.

## The real-path stall found — the doorway park (level-06-rook-door, 2026-10-25)

The real player path was walked end to end with real inputs (the walkreal legs: the truck spawn →
the dismount → the greeting plates → the west lane → the well courtyard → the hedge gap → the
guardhouse court → #438 killed on the fire path → E-door #47 → the guardroom → E-door #59 → the
searchlight-courtyard mouth), logging every drone's state/position/door binding per tick
(`crates/nf-view/tests/rook06_realpath_live.rs`, opt-in NF_ISO). The doorway stall is NOT the
driver and NOT the escort — it is the **switch-16 guard pair beside door #47** (drones at
(30.3, 41.1, −16.8) and (36.1, 41.2, −19.4) [D census + drone-props; the placement at
(25.4, 39.4, −14.5) in the same courtyard holds the lead props 0x1d/0x23 and the patrol props
0x24/0x25 but never spawns into the sim's 26]). The chain, per-tick from the walk:

1. Walking the interior route raises the step channels (12/13/16/17/18 observed; 15 at the mouth)
   [D #514–#523]; ch 16 enables the pair.
2. The pair engages Bond (he just fired on #438) and fights in the normal-block combat states
   (0x67/0x68/0x6d/0x69/0x73/0x74 … observed for 5000+ ticks).
3. Losing sight of Bond, one of them takes **RUN_FOR_COVER 0x8c** whose cover route crosses the
   **path 7 door-node pair** (35.9, 40.7, −15.7 / −16.8) at door #47: `ReachedDestNode` stands the
   door in +0xBA8 and parks him in **DSTATE_OpenDoor 0x81** (observed at +9496), the door opens,
   he resumes 0x8c and completes the run INSIDE THE DOORWAY (observed +9505).
4. The RUN_FOR_COVER arrival re-enters ATTACK, and the port's **ATTACK-ENTER flush arm** ran
   `NDrone2_ChangeToAttackMode`: his mode flipped **0 → 25 (0x19)** mid-fight (observed +9506) and
   ATTACK's own mode gate sent him to **SniperAim 0x2a — stationary, sight/combat range 1000**.
   He stood in the door jamb (35.47, 41.00, −16.93), bound to door #1, the door held at 0x4, for
   the whole 1800-tick watch. The player coming back through #47 finds an NPC parked in the
   doorway — the playtest door stall.

### Why the flush arm is not the game's [M]

`NDrone2_ChangeToAttackMode` 0x14a648 is called from exactly five sites — the `jal 0x14a648`
census of ACTION.ELF: 0x14ce68 (inside `NDrone2_GetDroneTypeAttackTypeFriend` 0x14cbd8, guarded
`+0xd8 (kind) == 0xb` and alert ≥ 1), 0x14d45c (`DoTypeSettingsOLD` 0x14d2e8 — OLD placements;
every 06 drone is NEW, param 5 = 0x64 [D drone-props]), 0x15da54 (`CivilianMission` 0x15d860 case
3's mission-arrival arm), 0x15e35c (`EnemyMission` 0x15e1e8 case 3's arrival — itself gated
`GameState != 0x0700006`), and 0x165948 (`GoToGoalPosition` 0x165640). **`NDrone2_DSTATE_Attack`
0x164210 and `DroneFunc_FirstAttack` 0x148010 are not callers.** A kind-2 NEW placement guard in
plain combat therefore never flushes: the port's ATTACK-ENTER arm (the mode 0 → 0x19 stationary
sniper flip) is the stall's whole cause.

The same read exposed the game's +0x5a4 lifecycle the port is missing:
`GetDroneTypeAttackTypeFriend`'s tail (0x14cea8 caseD_3) **writes the attack-fold result INTO
+0x5a4** (`+0x5a4 = uVar4`, the 0x56/0x12/0x2a/0xa7 fold) when the alt mode byte +0xc6 is clear
and the second switch +0x135 is set or the alert field +0x504 ≥ 1.0 — so a guard who has been
through one full-alert start-state computation no longer carries a flushable pending alt at all.
The port's `start_state` computes the fold but never writes it back, which is why the (wrong)
flush arm could fire mid-combat with a creation-time alt still pending.

### The fix, validated — and the 01 conflict that holds it

Removing the ATTACK-ENTER arm (+ the flush moved into a `change_to_attack_mode` helper used by
EnemyMission's existing arrival arm) was validated on the exact walk above: the guard keeps his
placement mode 0, fights THROUGH the doorway (he crossed to the courtyard side and pursued Bond),
door #47 ends at the shut idle 0x2; green on nf-game's 887 unit tests, every rook_*/fix_06_*
neighbor, escort_follow/fix_mission. **It flips `walkreal_07000001_live::
walk_from_the_spawn_to_the_exit_with_the_contact` red (master green, 3 passed on master, 2+1 with
the fix: the contact dies, state 0x47 at (−4.4, 6.3, 14.6), round 10)** — the 01 walk's balance
depends on the non-decompile flush parking the corridor guards as stationary snipers before they
hunt the contact. Landing order: re-derive 01 first (which guard kills the contact, his
dtype/kind and alt_state, and whether the +0x5a4 fold-write alone clears the situation), then
remove the arm. The full evidence walk is committed as `rook06_realpath_live.rs`
(#[ignore]d with the whole chain in the ignore reason until the fix lands).
