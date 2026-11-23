# s/g031-object-order: the players' place in the per-tick object order (GAPS G-031)

**Timing shift (moves the shared random stream on routes with a carrier):** the carry fix turns 5
real-input walk tests red that are green on master: walkreal_0700000b (3: the recorded tick count
17779 → 17781, two real-damage deaths) and walkreal_0700000d (2 real-damage deaths). Kept on this
branch. The trigger-lag fix is on its own branch, s/g031-trigger-lag, and is not part of this one.

## The game's order [M]

`Game_Run` 0x1c9790 → `control_movement_object_handler` 0x1336f8:
1. `Drone_InitComms`, `MP_DebugDraw` (empty stub), `Doors_Calc`, `Trigger_Calc` (touch triggers: clear, then `Trigger_Update` → `Trigger_Hit`).
2. The cel channel watch; the `switch_channels_prev/hold` snapshots.
3. The update walk over `DynamicObjList`, newest first (`control_add_object_to_list` 0x134698 prepends; the walk reads `next` before each update, so an object created mid-walk is first updated on the next tick).
4. The delete pass, the moved pass (prev pos `+0x40 ← +0x30`, matrix row → `+0x30` for 0x40-flag objects).
5. `Collide_Update` 0x1ea398 (:238): every object's collide slot, the contact lists (`obj+0xD0`) rebuilt.
6. The moved pass again, the cel relink pass.

Where Bond sits: `ResetMap_Load` creates the placements (`parsemap_block_map_data_dynamic`, 0x1d6360:146) and only then `Player_Start` → `Player_Init` 0x1910f8:33 `control_create_object` (:154). So within a tick:
- run-time objects (bullets, spawned guards, dropped pickups, emitters, blasts, debris) update **before** Bond;
- Bond's `Player_Update` (class 3);
- every placed object (doors, triggers, script players and their lifts, placed guards, placed pickups, searchlights, switches) **after** him;
- then `Collide_Update`: Bond's collide slot `Player_CollisionHandler` 0x1949a8 → `Player_FeetOnPoint` 0x194188 → `Control_InheritVelocity` 0x133e88 (the carry), and the trigger contact lists.
- `Trigger_Hit` 0x1b1fc8:18 reads `obj+0xD0`, which only `Collide_Update` fills, so every trigger read in a tick sees the previous tick's contacts.
- `Bullet_update` 0x12e7b0 is called only by the walk, so a round fired at tick N makes its first move at N+1.

## The port before this branch

`Sim::step_bond_sim` ran the carry, then Bond's walk, then `Drones::tick` (Trigger_Calc on Bond's new position, the ordered walk, `set_dynamic`). The carry used the carrier's move of the previous tick, and the player's trigger contact was the same tick's position.

## Fixed on this branch (with a test that fails on the old code)

1. **Lifts carried Bond one tick late.** The carry now runs after the world's tick (`Sim::ride_ground_post_tick`, also for the online remote replicas so the lockstep keeps one rule). On the 0700004a lift Bond used to stand 0.0303 above the descending floor at every tick's end; now the drift is under 0.005. `g031_object_order_live::bond_rides_the_0700004a_lift_on_the_same_tick` (old: worst drift 0.030311584).
## Fixed on s/g031-trigger-lag only (not merged here)

2. **Plates fire one tick early.** The player touchers used by `Trigger_Calc` and the walk's trigger updates should be last tick's (`Drones::prev_player_touchers`); the guards' touchers already stand at last tick's positions. `g031_object_order_live::the_05_reveal_plate_fires_on_the_tick_after_the_contact` fails on master (ch 0x45 on after tick 1) and passes there (tick 2). On top of the carry it turns about 25 more walk tests red: walkreal_07000001 (4, the escort AND plate and the exit), 02 (2), 03 (1), 05 (1, the truck objective), 06 (2), 08 (1), 09 (1), 0a (1, the recorded tick count), 0c (1), 11 (2), 14 (1), 15 (5, the mine fence), 16 (3). The routes are tuned to the same-tick plates; each needs its own retune before this can land.

## Still different (not fixed here)

- Run-time objects (spawner guards, dropped pickups) update after Bond's move in the port; in the game they update before it and see his previous position. A one-tick difference in what a spawned guard sees.
- Bond's rounds make their first move (and hit) on the shot tick in the port (`fire_player` in `run`, then `update_player_bullets` in the same `Drones::tick`); in the game the first move is the next tick. The port's own random draws ride that first step, so changing it moves the shared stream and many pinned fights; left for its own task.

## Unproven

- That the game's first tick after a load has empty contact lists rests on the code (no `Collide_Update` call in `ResetMap_Load`); no live capture of the first two ticks was made. The fresh-boot snapshot with ch 0x45 set fits either order, its tick is unknown.
- The carry's exact position inside `Collide_Update` relative to the trigger contact gather was not traced; the port's trigger toucher uses the post-walk, pre-carry body.

## Tests run

- `g031_object_order_live` 1/1 (the lift), `fix_finale_live` 4/4, `gate_guard_truck_live` 2/2, `level_end_live` 7/7, `fix_mission_live` 4/4.
- walkreal_* (the 28 level binaries + celprobe; the ~100 single-test probe binaries were not run), carry only: green except 0b (3 red) and 0d (2 red), both green on master, and 13 (2 red) which is red on master too.
- With the trigger lag as well: 16 binaries red (listed above).
