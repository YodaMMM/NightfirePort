# agent-qclaw-gate: the Q-Claw fires only from a context-icon zone

## The rule

`Player_WeaponFiring` 0x1a5460:515-528 [M]: on the fire-held arm, a Q-Claw record (0x50/0x51) returns
before `Player_RoundToFire` unless `*(char*)(BL+0x95F) == 2` (the player stands in a type-2 third
icon's arm) or `MPSettings+0x180 != 0` (multiplayer). The port fired the claw anywhere the bolt ray
reached a grapple point.

## The change

`crates/nf-view/src/scene.rs`:

- `Sim::claw_gate_open` is taken in `step_bond` from `bond.icon_ctx` before this tick's icon
  refresh (`Player_CollisionHandler` 0x1949a8:22 clears the byte and `ThirdIcon_Update` 0x196e80
  rewrites it in the collide pass, after the weapon arm reads it), or from `mp.is_some()`.
- The latch in `step_bond` requires it; `run` refuses the claw's pull (`claw_block`, next to the
  underwater block) so a refused press runs no fire script.
- MP remote bodies (`step_net_remotes`) are untouched: MP always passes the gate.
- The `+0x160` in-flight check on the same arm has no port counterpart (the port has no claw bolt).

## Where the icons are [D, `Level` load over every level with grapple points]

| Level | Type-2 icons |
|---|---|
| 07000005 | the ledge (-18.46, 37.49, -72.49) r 3.39; (6.22, 5.43, 65.41) r 11.0 |
| 0700000a | (-11.87, 0.72, -6.23), under the point |
| 0700000c | eight, around the five points |
| 0700000d | five, around the three points |
| 07000011 | two beside the point |
| 07000012 | (-9.79, 1.17, 32.78), on-channel 60 |
| 07000014 | one on the trench floor (y 0.41) under each of the three points |
| 07000041 | one, on-channel 1 |
| MP maps (0x23-0x29, 0x4b) | none; MP passes the gate |

## The 07000014 walks

The old route fired the claw from the vent store room at (40.8, 2.4, 65.5), 5 units from the nearest
type-2 icon, and latched the SkyBridge point (45.29, 5.10, 62.94) through the duct wall. With the gate
on, all eight walks that use it went red.

The re-walk:

- **Out of the store room.** The port has no walkable exit from it. The grill closes the south duct
  (docs/levels/07000014.md §1a), and a segment probe finds the east wall at x 41.53 for the full
  duct height from z 62.2 to 66.8. The walks therefore make one documented hop onto the trench
  floor at the old crossing's landing (43.05, 0.45, 62.86), then walk on as before.
- **Into the east vent.** The claw now fires from the mouths' icon arm. The walk goes down the
  corridor to (48.8, -0.6, 44.0), then west onto the trench floor at (46.6, -0.6, 42.9). The test
  asserts `icon_ctx == 2` there, then latches #626 (45.30, 4.79, 42.86). The hang sits over the
  lane's open strip, and the release lands Bond on the lane. Before, the claw fired from the 0.60
  segment, 8 units from the icon.
- `the_east_duct_route_reaches_the_stairwell_slot` now also asserts that the press at the old
  stance does not latch (`icon_ctx == 0xFF`).

Walk-harness fixes that the new timing exposed. They are not rule changes:

- `clear_hostiles` returned early without walking home, so a clear that finished early left Bond
  off his leg.
- `cut_fusebox` steered with an unwrapped yaw difference, so Bond spun a full turn while firing and
  drained the laser meter. It now wraps the difference and aims before it fires.
- The corridor jamb leg's arrival radius is now 0.4, down from 0.9. At 0.9 the leg "arrived" east
  of the jamb.
- The pit's back-east jump in the east-route test is now edge-anchored, the same as the
  infiltration walk's jump. The legs are shared in `corridor_to_the_stairwell_foot_and_back`.

## Other claw tests moved into the icon arms

Each test now asserts `icon_ctx == 2` at its stance:

- **07000005**: the stance moved to the ledge icon (-18.28, 38.19, -71.83), aiming at #281. This
  is the same move as on s/agent-weapons. Tests: `gadget_grapple_live` and its spot sister,
  `qclaw_live`.
- **0700000c**: on the pulley's icon (-5.35, 2.93, -179.48). Test: `interact_0700000c_live`.
- **0700000d**: on the icon (219.43, 3.39, -228.88). Test: `interactables_0700000d_live`.
- **07000011**: on the west icon (-5.81, 46.38, 24.39). Test: `interactables_07000011_live`.
- **07000014**: the trench arm. Test: `interactables_07000014_live`.
- **07000041**: icon #578 (8.59, 0.00, -29.47). Its ch 1 is raised as in
  `the_third_icons_arm_by_their_channels`. Test: `interactables_07000041_live`.
- **07000012** (`qclaw_ride_07000012_live`): the only type-2 icon, #680 (-9.79, 1.17, 32.78), arms
  on ch 0x3C. That channel's only writer is the "Sky bridge" ScriptPlayer #660 [D level audit],
  and what starts that script in play is not traced. The ride now stands in ch 0x3C and the
  stance on the icon. Before, its stand-in stance was the parapet. The reel, hang, release,
  landing and walk-on pins are unchanged and pass.

## Tests

New `qclaw_icon_gate_live` (2 tests):

- From the trench floor beside the SkyBridge icon: context 2, and the claw latches.
- From the corridor floor 3.5 units east: context 0xFF, and the claw does not latch.

On the old `scene.rs` (only that file reverted), three tests fail as they should:
`the_claw_does_not_fire_outside_the_icon_arm`, `the_east_duct_route_reaches_the_stairwell_slot`
(the claw latches from the store room) and `gadget_cells_run_the_cycle_on_07000005` (the claw
fires at the matrix stance).

Green on this branch:

- `walkreal_07000014_live`: 8 of 9 (1 ignored, as on master; the one failure is below)
- `walkreal_07000005_live`
- `walkreal_0700000c_live`
- `qclaw_live`
- `gadget_grapple_live`
- the gadget_grapple spot sister
- `grapple_hook_live`
- `qclaw_ride_07000012_live`
- `interact_0700000c_live`
- `interactables_07000005_live`
- `interactables_0700000d_live`
- `interactables_07000011_live`
- `interactables_07000014_live`
- `interactables_07000041_live`
- `fix_wallhug_prompt_live`
- `wire_live`
- `mp_world_sync_live` (MP passes the gate)
- `weapon_matrix_live` (the claw cell now pins no fire outside an arm)
- `gadget_select_live`
- `fix_weapon_modes_live`
- `progress_remainder_live`

## Still red

- `walk_the_tripbomb_route_pick_up_place_and_shoot_it` (07000014). The placed bomb is gone before
  the shot.
  - A guard (placement #378) follows Bond into the armoury and trips the armed beam during the
    retreat.
  - The hand-back now comes about 200 ticks earlier than on master, and this run's arrival tick
    puts the guard on the beam line. Shifting the arrival tick by 250 or 504 ticks passes, so the
    walk depends on timing, not on the claw.
  - A real fix needs the walk to deal with the follower, for example by killing him before the
    retreat. Two tries at clearing him failed: the armoury shelves block the walker's loose shots.

## Unproven

- The real exit from the vent store room, the route the game's Bond takes. Only the real game can
  settle it.
- Whether the game's SkyBridge reel from the trench lifts Bond into the east duct. The port does
  that from the icon arm.
