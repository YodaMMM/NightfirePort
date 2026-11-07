# mp-object-deleted — bots drop a goal whose objective object is gone

Branch `s/mp-object-deleted` (from origin/master 10956badb), 2026-11-07.

## What the game does, read in full

- `control_delete_object` 0x133178:20-22 [M 00133178 decompile]: under the MP gate
  (`MPSettings+0x388 != 0`) every object deletion calls `MP_objectBeingDeleted(obj)`. The
  delete itself is driven by the object-control pump: an object with bit 0 of `+0xfe` set is
  removed by `control_movement_object_handler` 0x1336f8:70-77 (`Effect_RemoveSObjs` +
  `control_delete_object`, then the pump's reference sweep).
- `MP_objectBeingDeleted` 0x18ac90 [M 0018ac90 decompile, read in full]: early-outs when MP
  is off or the obj is NULL [:12-15]; clears the 8 player-slot obj words `MPGame+slot·0x30+0x1c`
  [:18-25]; the mode-row sweep matches on `MPSettings+0x420` [:27-253]:

  | scenario word | rows the game clears | port store today |
  | --- | --- | --- |
  | 0x20000004 CTC | `Flags[2]` @0x317240, `Bases[2]` @0x317360 (each 0x90-byte rows, AI emitter freed first) | `MpObjectsRt.flags[2]`, `.bases[2]` + `_bounds` |
  | 0x20000040 Demolition | the one `Demolition` row | `MpObjectsRt.site` |
  | 0x20000080 Protection | the one `Protection` row | `MpObjectsRt.site` |
  | 0x20000100 Blueprint | `BluePrint` + `EsponageBase[2]` @0x3188f0 | `MpObjectsRt.blueprint`, `.esp_bases[2]` + `_bounds` |
  | 0x20000200 GoldenEye | `GoldenEye` @0x318020 + the second row @0x318080 | `MpObjectsRt.ge[2]` |
  | 0x40000800 / 0x60001000 KOTH | `Hill` @0x318CE0 | `MpObjectsRt.hill` |
  | 0x60000008 Uplink | `Uplinks[0..UplinkCount]` @0x317480 | `MpObjectsRt.uplinks` |

  then the 64 `MPObjects` slots [:256-266], and on any mode-row/MPObjects hit routes msg 0x3D
  (dest 0xc5 BotGlobal, arg = the deleted obj) through `Drone_SM_RouteMsg` [:268-276]. The
  slot +0x26 status bits are NOT touched. The AI-emitter frees and the slot +0x1c words fold
  in the port (the port keeps no player world objects and owns no AI emitters); the `MPObjects`
  fold is the RT-row drop (the port's created records carry no per-object ids).
- The consumer: msg 0x3D is the BotGlobal arm at 0x16E060:626-634 →
  `BOTSTATE_cancelGoalToObj` 0x128590 [M 00128590 decompile, read in full]: per goal slot,
  type 2 cancels when the objective is gone or is the deleted obj [:22-25], type 3 on the
  direct match [:26-29]; the canceled slot's +0x4A/+0x45/+0x44/+0x48/+0x20 zero [:46-52], the
  +0x734 bit2 follows the subtype-1 word [:33-41], the +0x765 active byte clears [:43-45];
  when the canceled slot was the bot's active goal, `BOTSTATE_setStateChange(0xF9)` [:52-55]
  — the SMI commits the BotIdle state on the same tick's pass. Delivery is dispatch-now
  (delay 0 [M 0x172128]), so every live bot has handled the delete before its next state tick.
- The site getter tail: `MP_getObjExtObj` 0x18bca8 [M 0018bca8 decompile, read in full] — when
  the row's script obj (`SP_getScriptInfo(SP+0xc)` → `Script_GetObj(0)`) differs from the
  row's obj, the row is re-pointed and msg 0x40 (dest 0xc5) routes so the attackers re-target;
  the re-target arm reads `MP_getProtectionObj_NotExt` 0x18bc68 / `MP_getDemolitionObj_NotExt`
  (the same getter). The port keeps `get_obj_ext_obj` as that verified tail [M :35-53].

## Which objects can really be deleted during a match

- **Between rounds, every created mode object — proven.** `MP_RestartScenario` 0x183fc0
  [M 00183fc0 decompile, read in full] deletes classes 5 / 0x35 / 0x10 / 0xe
  (`Control_DeleteAllObjectsOfType` :14-17 — 0x35 is the class `MP_CreateObject` stamps
  [M 0x185de8:122]) plus the MPObjects SP-obj sweep (`+0xfe |= 1`, :29-39); the pump deletes
  each and `MP_objectBeingDeleted` runs per object, clearing the rows and sending 0x3D per
  deleted object while the bots are still alive (they respawn after, :76-90). Only the
  Demolition/Protection site is re-created (the `Rand_Rand` pick + HP 2000, :41-56); the other
  scenarios' rows stay gone — the game has no other create in the restart.
- **Mid-round, nothing is provable from the decompile.** No `MP_*` function calls
  `control_delete_object` (the decomp callers are the pump, `AnimDebugHandler`, and the
  Switch/Creature/DroneSpawner create-failure paths). The `+0xfe |= 1` writers are:
  `Pickup_Handler` 0x18e650:339 (pickups — the `MP_PICKUP` table, not the mode rows; the port
  already unregisters those via `MP_UnregisterPickup` 0x18a058), `control_handle_cel_change`
  0x133288:266 (classes 5/6/0x10), `DynamicObject_Update` 0x18d3b8:25 (a channel-gated object
  whose channel turned off — created MP objects carry a fixed cel, not a placement channel
  [S]), `SP_Update` 0x1dbda8 case 5 (a script ending in the remove state), and the
  NDrone2 death/fade paths (players/bots, not mode rows). The Demolition/Protection site's
  fall switches its SP to script 1 [M 0x1870a0]; whether that script ends in SP state 5
  (deleting the site obj mid-fall) is **not verified** — see Live checks.
- So per the task's branch: no scenario can be shown to delete a mode object mid-round from
  the decompile alone; the between-rounds sweep is the proven in-play deletion. The wiring is
  pinned by units through the real path (`restart_scenario` → `object_being_deleted` → the
  per-bot drain), and a mid-round delete runs the same functions.

## What the port does now

- `MpMatch::object_being_deleted(id)` (nf-game/src/mp/mod.rs) replaces the uncalled
  `object_being_deleted(objs: &mut [usize], obj)` stand-in: the scenario-gated row sweep over
  `MpObjectsRt` exactly as the game's match arm, one queued 0x3D per deleted object in
  `MpMatch::pending_goal_cancels` (the game's per-object RouteMsg order).
- `BotBrain::cancel_goal_to_obj` + `BotBrain::msg_goal_cancel` (nf-game/src/mp/bots.rs) port
  `BOTSTATE_cancelGoalToObj` 0x128590 and the BotGlobal 0x3D arm 0x16E060:626-634
  (`setStateChange` = SetState + the +0x72C record; the SMI commits 0xF9 on the pass).
- The driver (`step_mp_bots`, nf-view/src/scene.rs) drains the queue per bot in slot order
  before the bot's own tick — the game's delivery order and tick.
- `restart_scenario` (nf-game/src/mp/rules.rs) now runs the game's sweep through
  `object_being_deleted` (fixed row order standing in for the MPObjects creation order [S]),
  re-creates only the site (the `Rand_Rand` pick, HP 2000, last-hitter −1, limit 0
  [M 0x183fc0:41-56]), leaving the other scenarios' rows gone as in the game.
- The 0x40 re-target: the port's site view has no SP script-obj identity (`MpObjState` keeps
  the `sp_script`/`sp_state` words only), so the ext-swap case of 0x18bca8 is not observable;
  the attackers re-target through the per-tick `BotObjectives` rebuild plus the 0x3D cancels
  on the delete path [S port-shape]. `get_obj_ext_obj` stays as the verified tail pin.
- No shared-Rand draws added: the restart re-pick reuses the game's own `Rand_Rand` position
  [M 0x183fc0:42]; the sweep and the drain draw nothing.

## Verification

- 6 new unit pins (`mp::bots::object_deleted_tests`), each assertion citing its decompile
  lines: the CTC/Uplink/site sweeps + the 0x3D queue + the scenario gate; the goal-cancel
  semantics (slot clear, +0x765, +0x734 bit2, the SMI-pending 0xF9 + the +0x72C record); the
  restart sweep → re-create → the bot leaves the goal (the driver's drain order, the same
  tick); CTC's rows stay gone after a restart.
- Full suite (`tools/test/suite.py`, release, per-binary): every `mp_*_live`,
  `mp_world_sync_live`, `npc_doors_live`, `fix_guard_misc_live` ran inside it.
- Suite result: see below (filled after the final run).

## What's left

- The mid-round deletion candidates (the site's destruction script, channel-gated objects)
  need the live checks below before anything is wired for them; the decompile alone does not
  prove them and nothing was pinned on a guess.
- The port's uplink rows are values, not obj rows: the sweep zeroes the row in place
  (state 0, sub 0) instead of dropping it — the port's Vec keeps the indices stable [S
  port-shape].
- `restart_scenario` has no driver caller yet (pre-existing); the sweep runs whenever the
  port's round restart is wired to the driver.

## Live checks for the main agent

1. Demolition site fall (any Demolition map, fresh boot): set an instruction-write watch on
   the site obj's `+0xfe` byte around the fall (`SP_SwitchToScript(site_sp, 1, 0)` at
   0x1870a0). Expected if the destruction script removes the obj: `SP_Update` 0x1dbda8 case-5
   writes bit 0, the control pump deletes the obj next pass, `MP_objectBeingDeleted` clears
   the row and routes 0x3D. If it never fires, the site obj survives its own fall and the
   port's sub=1 handling is the whole story.
2. CTC capture: breakpoint `MP_RestartScenario` 0x183fc0 — does a CTC round restart reach it
   in play (the flags/bases rows really going empty between rounds), or is it site-rounds only?
3. Uplink mid-round: watch the uplink terminals' `+0xfe` bytes during a full round for any
   `|= 1` write (the `DynamicObject_Update` channel gate / an SP script ending in state 5).

## Usage

not visible to me — this session's driver shows no token, credit or context-window figures.
