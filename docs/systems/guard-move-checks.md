# Guard movement checks: the NDrone2 can-move probes, the node move tests, `AnimForDist`, `DistanceToAIPoint`

Sources: ACTION.ELF decompile (`work/decomp/ACTION.ELF`, function address = filename prefix). [M] = read in
the code, [D] = disc bytes, [L] = live measurement, [S] = suggested. Port: `crates/nf-game/src/drone.rs`
(`can_move`, `can_backoff`, `choose_combat_move`), `crates/nf-game/src/nav.rs` (`move_test`,
`target_nodes`, `astar`, `node_from`), `crates/nf-game/src/mp/bots.rs` (`can_side_move`, `pick_goal`,
`anim_for_dist`). This chapter exists because the call-tree check
(`tools/calltree/calltree.py --all-mentioned-dstates --list`) found these game functions reachable from
ported guard states while no port code named them; every one is now ported or cited at the code that does
its work.

All ten share one shape: a **local-space probe**. Take the drone's feet (`NDrone2_FeetPos` 0x149b88),
add a local offset rotated by the drone's matrix (obj+0x90, `ApplyMatrixLVI`), find the target cel
(`build_FindCel`, falling back to `NDrone2_FindCel`), publish the target into the reach globals
(`DroneReachCheckPos` x / DAT_0029b2f4 y / DAT_0029b2f8 z / DAT_0029b2fc pad), and gate the move through
`NDrone2_MoveTest` 0x155330 == 1. The port's shared spatial probe is `Drone::can_move`
(drone.rs) with the documented floor-vs-AI-boundary deviation (docs/systems/drone-navigation.md).

## The Can* family [M]

Each returns bool; gates in game order. `dcv[0]` = obj, `dcv[1]` = drone. The `(drone+0x4f8) & 0x10`
gate ("frozen/controlled", common to all) and the speed-mul scale
(`class +0xc5 == 0x1e → BOT_getMovementSpeedMul` scales the offset) apply inside every probe.

| function | address | flag gate (drone+0x31c) | behaviour prop | anim state(s) (`DroneAnim_CanDoAnimState` 0x13dfc8) | probe offset (local x, y, z) |
|---|---|---|---|---|---|
| `NDrone2_CanStepLeft` | 0x14f640 | bit 3 clear | 0x42 | 0x5a | (+1.8, 0, 0) |
| `NDrone2_CanRollLeft` | 0x150338 | bit 3 clear | 0x2e | 0x53 | (+5.3, 0, 0) |
| `NDrone2_CanStrafeDodgeLeft` | 0x14fdf8 | bit 3 clear | 0x13 | 0x29 | (+3.6, 0, 0) |
| `NDrone2_CanStrafeDodgeRight` | 0x14ffc0 | bit 4 clear | 0x13 | 0x29 AND 0x2a | (−3.6, 0, 0) |
| `NDrone2_CanBackoff` | 0x1501a0 | — (none) | 0x06 | 4 | (0, 0, −1.8) |
| `NDrone2_CanMoveToRelativePos` | 0x154838 | — (none) | — (none) | — (none) | (dx, dy, dz) arguments |

Offsets are the decompile's float literals: 0x3fe66666 = 1.8, 0x40666666 = 3.6, 0x40a66666 = 5.3,
0xc0666666 = −3.6, 0xbfe66666 = −1.8. The right-side twins (not this task's list, cited for the mirror):
`CanStepRight` 0x14f808 (−1.8, anims 0x5a+0x5b, prop 0x42) and `CanRollRight` 0x150500 (−5.3, prop 0x2e).
The strafe (non-dodge) pair: `CanStrafeLeft` 0x14f9e8 / `CanStrafeRight` 0x14fbe0 (prop 0x41, anim 0xc).

Callers [M, decompile grep]: `NDrone2_ChooseCombatMove` 0x150c18 (step/roll/dodge cases, right-first or
left-first per `Rand_Rand(2)`), `NDrone2_EvasiveMove` 0x1506d8 (same option list, the dodge case's
result states 0x77/0x78, and the class ≠ 0x1e gate on its `Drone_SM_SetState`), the dodge/strafe
DSTATEs (`DSTATE_StrafeDodgeLeft` 0x167b50 / `DSTATE_StrafeDodgeRight` 0x167cf0 re-probe on entry),
`DSTATE_AimBackoff` 0x166b98 and the aim-fire family (CanBackoff), `DSTATE_NinjaAttack` 0x16bb50 /
`NinjaAttackMidRange` 0x16c248 / `NinjaStandFire` 0x16d5b8 and `DroneWeap_AimTarget_IsOpponent`
0x178ae0 (CanMoveToRelativePos with (0,0,±5) and (0,0,0.4)), and the mp bot states
(`BOTSTATE_checkAttackMove` 0x126a40, `DSTATE_BotAttackRun` 0x16f390, `BotGlobal` 0x16e060).

Port mapping: `Drone::can_move(d, x, z)` is the shared probe (the y leg of
`CanMoveToRelativePos` is folded into the port's floor rule — the deviation the drone-navigation chapter
documents). The prop/anim gates live at the call sites, exactly as the game's callers gate them:
`choose_combat_move` cases 1/3/4 carry prop 0x13/0x2e/0x42 + the 0x29/0x53/0x5a `can_do` checks;
`can_backoff` carries prop 0x06 + anim 4 + the (0, −1.8) probe. The mp bot side's `can_side_move`
(bots.rs) is the BOTSTATE stand-in ([S], documented there). **Correction this task made**: the port's
roll pair probed ±5.2; the decompile's `CanRollLeft`/`CanRollRight` literals are ±5.3 — fixed to 5.3
[M 0x150338 / 0x150500].

## `NDrone2_MoveTestFromNode` 0x1554a8 and `NDrone2_MoveTestToNode` 0x155620 [M]

Mirror twins of `NDrone2_MoveTest` 0x155330 (identical bodies, the MAPNODE's position +0x10 / height +0x14
substituted for one endpoint): d = to − from; total d² must be ≤ DAT_002c7954 (900.0, `Drone_PostLoad_Init`
0x136938); |dy| ≤ DAT_002c794c (4.0); if |dy| > DAT_002c7948, |atan2(dy, horiz)| ≤ DAT_002c7944 (the
slope band); then `AINetwork_BoundsTest` — gated on DAT_002c8774 (the boundary count; 0 short-circuits to
pass). The port's `Network::node_scan_ok` (nav.rs) is this exact gate chain, and the two call shapes map to:

- `target_nodes` (mode 0 of `AINetwork_NodeSearchCel` 0x156e18): `MoveTestFromNode(node → pos)` per
  candidate, first pass = the goal node, first fail = the fallback [M 0x156e18:127-144].
- `astar`'s start scan and `node_from` (`AINetwork_NodeSearch` mode 1): `MoveTestToNode(pos → node)`,
  the −1 result stopping the scan [M 0x156e18:147-176].

`AINetwork_OptimiseRoute` 0x156760 calls `MoveTestToNode` twice — the port's route trim in
`Route::to_position` (start→next-node, dest→prev-node). `AINetwork_AlterDestFor_DROUTE_Nearest` 0x156088
(the FromNode caller) is not ported — the documented gap (docs/systems/uncited-audit.md).

## `NDrone2_DistanceToAIPoint` 0x155d78 [M]

Writes `AIPoint+0x10 = Vec_Dist2D(NDrone2_FeetPos(obj), AIPoint+0x20)`. The AIPoint here is the drone's
embedded goal record at drone+0x6f0 (so +0x20 is the goal position the states call "+0x710" and +0x14 the
goal accuracy at "+0x704"). Callers: `DSTATE_GoToGoalPosition` 0x165640, `HostageGoToGoalPosition`
0x165a68, `AllyGoToGoalPosition` 0x15f938, `BotGotoGoalPosition` 0x171240, `BotGuardFriendFollow`
0x1718e0, `DSTATE_RunAwayFromObject` 0x168aa8, `BOTSTATE_pickGoal` 0x126b88 — every one calls it
immediately before `NDrone2_MoveToGoalPosition` 0x1517d8. The port computes the same distance as
`route.remaining` (drone.rs `move_to_goal`, the route-distance deviation bots.rs `pick_goal` documents)
over `self.goal` instead of an embedded record; `nav::dist2d` is the port's `Vec_Dist2D`.

## `NDrone2_AnimForDist` 0x1512e0 [M]

The drone move-anim pick. Signature (the mangled name drops the arg order): `(float dist_move, DCVars*)`.
Inputs: the drone's move speed +0xf0, the opponent distance +0x1a0, the turn latch +0x228 bit 2, the aim
angle +0x1d4 (vs 0.7853982 = π/4), the current anim state +0x56a, the class +0xc5, behaviour props 0/1/2.
Selection, in game order [M 0x1512e0:14-89]:

1. start at 0x1e (run); a class-0x1e drone starts at 8, kept only if `CanDoAnimState(8)`.
2. **not turning** (+0x228 bit 2 clear): if `1.5·speed ≤ dist_opp` keep the run; else a prop-1 drone with
   (prop 2 or `CanDoAnimState(8)`) walks (8).
3. **turning** and `dist_move < 0.5·speed`: anim 10 (the stop-walk).
4. **turning** and `dist_move ≥ 0.5·speed`: if `dist_opp ≥ speed` fall to the step-2 near-walk try; else a
   (prop 0 or prop 2) drone that `CanDoAnimState(0xf)` strafe-walks (0xf), else the near-walk try.
5. if the picked id differs from +0x56a, `DroneAnim_CallAnim(0, id, …)`; the fire-request byte +0x3b is
   written from the "picked a non-run anim" flag behind a turn/aim-angle condition the decompile shows with
   unresolved registers (0x1512e0:86-89) — the exact store shape is not recoverable from the C.

Its only caller is `DSTATE_BotAttackRun` 0x16f390:87, which passes drone+0x8e8 (the remaining route
distance `NDrone2_MoveToObject` 0x151618 stores). The port lifts the selection as the pure
`mp::bots::anim_for_dist` (bots.rs) with every game input as a parameter and unit tests pinning the table
above; the port's bot brain carries no drone-anim state (the mp bodies run the game's player-body anim
system, `AnimSetUpdate` 0x1c0938 — docs/systems/mp-body-anim), so the pick is not applied to a body yet
[S deviation — the draw side can consume the function's result when a body wants it].

## Tests

`crates/nf-game/src/mp/bots.rs` unit tests pin the `anim_for_dist` table (numbers only, [M]-cited, each
with a sister that can fail). `crates/nf-view/tests/guard_move_checks_live.rs` runs the real path
(`nf_view::load` + `Sim::step_bond`): a prop-0x42 guard on open floor steps (1.8 probe passes) and a
guard against a wall does not (the probe must fail), plus the roll 5.3 distance through
`choose_combat_move`'s case-3 probe.
