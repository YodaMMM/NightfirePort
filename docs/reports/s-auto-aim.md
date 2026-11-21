# s/auto-aim: Bond's auto-aim as ACTION.ELF runs it

**RNG: this branch moves the shared random stream.** Shots now take slightly different lines
(decay ticks, aim-mode shots, zoomed shots), so hits and the draws behind them land differently.
Keep it on the branch until it is merged as a batch.

Spec: docs/systems/auto-aim.md. Code: `nf_game::autoaim` (new), `Drones::auto_aim_view`
(drone.rs), `trig::asinf_libm`, and the per-tick step in `Sim::run` (scene.rs).

## What the game does [M]

- `Player_Update` 0x193600 runs `Player_Weapon` (which ends in `Player_AutoAim` 0x1a7c98) and then
  `Player_WeaponFiring`, so a shot reads the offset written that tick.
- The offset `BL+0x120/+0x124` is a screen position, ±1 at the screen edges. `Check_AutoAim`
  0x1a73f8 projects the target with `View_3DPoint2Screen` 0x1e7708 and stores twice its screen
  position. `Player_AutoAim` halves it. `HUD_UpdateCrossHair` draws it as is.
- `Player_GetAimingPoint` 0x1a4a98 aims through that point, divided by the zoom (viewer +0x108).
- The gate is the aim bit (`+0x60` bit 0), not the button. Bit set: ease toward the cursor 0.5 a
  tick, lock kept. Bit clear, option on: scan. Option off: halve, lock kept.
- The Q-Claw forces the option on. Its cap starts at the record's +0x1c (50), and the release only
  fires when the cap still equals 25, so a claw scan never drops its lock.
- The lock pointer and offset change only when the projection succeeds. The cap takes the distance
  either way.
- `asinf` 0x232d48 is fdlibm's. `__fdlib_version` is 1, so |x| > 1 returns 0.0.

## What changed in the port

1. `aim_offset` holds the game's screen units, not yaw/pitch radians. The lock tick aims at the
   target as before, but decay ticks now halve in screen space and the shot goes through the
   game's camera frame.
2. The aimed ease ran twice a tick (step_bond and the dir pick) and in angle units. It now runs
   once, on the aim bit.
3. Aim-mode and zoomed shots go through the cursor divided by the zoom. Before, a zoomed shot left
   at the unzoomed angle, off the crosshair.
4. The option off no longer clears the lock. The Q-Claw scans with the option off and keeps its lock.
5. The scan uses the game's ops: `Vec_Dist3D`, the k product order, `atan2f` (`atan2f_libm`) and
   `asinf_libm` in place of host `atan2`/`asin`.
6. `trig::asinf_libm`: the wrapper and core op for op.
7. Test walkers: the aimed stands steer the cursor with `Sim::cursor_for` (the target's screen
   position times the zoom), and read the effective aim through `Sim::aim_offset_angles`.

What a player would notice: aimed shots with a zoomed weapon land on the crosshair. Shots right
after a lock drift back along the game's line. The Q-Claw snaps to grapple points with auto-aim
switched off, as on the PS2.

## Draw note (not changed)

The game's crosshair sits on a fresh lock. The port's drawn crosshair sat about half way there,
because the old store held half the angle. `Sim::crosshair` keeps the port's old drawn position
from the new value. Moving it onto the target is a visual change that needs a look first.

## Tests

Fail-first, all fail on the base commit 0817567d3 and pass here:
- `auto_aim_live` (new, nf-view): the single screen-unit ease (old: [-0.1154, 0.0433] against
  [0.1, 0.05]), the lock kept with the option off (old: None), the claw scanning with the option
  off and keeping its lock (old: never snapped).
- `ai_live::auto_aim_offset_is_the_targets_screen_position` (nf-game): the stored offset equals the
  halved projection and is positive for a guard on the right. The old store held -0.025 rad.
- `trig::asinf_libm_is_the_games_wrapper_and_core`: 0.0 outside [-1, 1] (the old clamp gave ±π/2),
  within 4 ulp of the f64 arcsine inside.
- `autoaim` unit tests: the focal scales give 0.76966/0.57724; the halved projection aims at the target.

Also green: ai_live auto_aim (3), fix_aim_raise_live (5), civilian_no_shoot_live (3),
progress_remainder_failfirst (3), progress_remainder_live (5), fix_hud_zoom_tilt_live (3),
fix_autoaim_elevator_live (1).

walkreal_* (all 28 level binaries, walkreal_07*; the walkreal_probe* binaries were not run):
- Green: 01, 02, 04, 05, 07, 08, 09, 0a, 0b, 0c_barrelrow, 0c_eastroute, 0c_ventexit, 11,
  11_route, 12, 14, 16, 41, 41_wall, 4a, 4a_satchel; 0d's plain walk; fix_guard_misc_live (6),
  gadget_laser_live (8).
- Red, every one a real-damage walk where Bond dies, all green on the base commit:
  - 03 `walk_the_full_route_and_complete_every_objective_real_damage_00agent`
  - 06 `walk_the_castle_interior_to_the_party_real_damage_00agent`
  - 0c `walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage`
  - 0d `walk_arrival_to_the_c5_door_real_damage` and `..._real_damage_agent`
  - 13 `walk_the_warm_chain_to_the_front_entrance_and_out_real_damage`
  - 15 `walk_spawn_to_the_omega_sector_exit_real_damage_00agent`
  - 1b `walk_arrival_to_drakes_defeat_on_real_health_00agent`

  These walks are fitted to the shared stream and to where each round lands. The shots now take
  the game's lines, so the fights roll differently. They need the batch re-pin, not code changes.
- 13 `..._real_damage_agent` is red on the base commit too.
- Walker fixes on this branch: the aimed stands steer the cursor with `Sim::cursor_for`, and the
  tests that rebuilt the shot ray from `cross` now divide it by the zoom, as the shot does. Without
  these, 06, 0d (plain), 15 and others stalled in their aimed stands.

## Unproven

- The view frame both the projection and the aim use is built from Bond's yaw and pitch with the
  port's sin/cos. The game's viewer matrix and `obj+0x90 · RotX` are not checked bit for bit.
- `focal` uses the host `tanf`. The libm `tanf` chain is not ported.
- The EE `sqrt.s`/`div.s`/add models are the port's shared models, not checked live here.
- The head and the camera eye are the same point in the port. The game projects from the viewer
  eye (+0xc0) and aims from `Player_GetHeadPos`. If they ever differ in the game, the lock would
  not land exactly on the target there. Live question: are viewer+0xc0 and the head position equal
  in on-foot first person?
- The drone walk order, the grapple point position and drawn gate, MP teams and zero-g view frames
  are listed in the spec as not ported.
