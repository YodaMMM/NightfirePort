# audit-activate-side-effects

Every `*_Activate`, `*_Deactivate`, `*_Create` and `*_Delete` of the on-foot program (ACTION.ELF) for
on-foot objects, each body call in the camera / HUD / input / weapon / sound / player-state categories
mapped to the port's counterpart. Motivating example: `GunImp_Activate` 0x132f00 calls
`Camera_CalcViewAngles(30°)` — the port's answer is below (it is already implemented and tested on
master). Driving objects (Car = the tank, MiniSub) are skipped per the task. Evidence tags per repo
rules; every game address below is [M] (the decompiled C file named `work/decomp/ACTION.ELF/<addr>_*`).

## Method

- Sweep of `work/decomp/ACTION.ELF/` (3039 functions) for `*Activate*` / `*Deactivate*` / `*_Create*` /
  `*_Delete*`; SLUS_205.79 (the boot program) holds only EE-kernel syscall stubs — no object control
  code, so ACTION.ELF is the complete on-foot set.
- Each body read for calls in the audited categories (`Camera_`, `HUD_`, `Input_`, `Sound_`, `Snd`,
  `ssys_`, `Player_`, `Weapon`, `Music_`, `Text_`); the port side greped per family and each site
  read. Two parallel research passes (decompile audit, port map) cross-checked by hand reads of
  Monitor_Activate, Switch_Activate, Lock_Activate, PCQWorm_Activate, SS_Activate, GunImp
  Activate/Deactivate, Car Activate/Deactivate, GT_Activate, GT_Delete, Car_Delete, Camera_CalcViewAngles
  and the port's monitor.rs / lock.rs / door.rs activate paths, gunimp_tick dismount, view_fov and the
  scene.rs mount sites.

## The table

Game call (ACTION.ELF address) → port site → verdict. "present" = the port performs an equivalent,
player-equivalent side effect (mechanism may differ, always noted); "n/a" = no player-visible or
audible effect on real input.

### Monitor_Activate 0x1b0058 (the richest site)

| Game call | Port site | Verdict |
|---|---|---|
| `Camera_SetUpdator(slot, player, 0)` + the CameraList match (skips cams in state 3) | `Monitors::activate` (nf-game/src/monitor.rs:76-89, match on sensors); the view itself = `Sim::view_override` (nf-view/src/scene.rs:2505-2517) reading the watched sensor's matrix each render — the game's `Monitor_Update` 0x1afcc8:155 copies the camera matrix into the viewer every tick, so the visible result is the same pose | present (draw-side realization, documented at both sites) |
| `Input_SetAction(slot, 0xe, 1)` | the use key routes to `Monitors::cycle` on the 0xE edge (nf-view/src/scene.rs:2841-2847 → monitor.rs:104-113) | present |
| `Player_Disable(player, 0)` | per-tick input zeroing `(Default::default(), false, false, use_key)` + the walk gate (nf-view/src/scene.rs:2841-2847, 3381); the port has no `Player_Disable` flag (its `Sim::disabled` is the script cut-scene arm, scene.rs:654) | present (per-tick stand-in, the player is frozen either way) |
| `HUD_Enable(hud, 0x10, 1, 0)` (pane 16 on) | the SecCam pane sprites (nf-view/src/hud.rs:642-657, tint 200-207, lamp 478) | present |
| `HUD_Enable(hud, 0, 0, 0)` + `HUD_Enable(hud, 1, 0, 0)` (panes 0/1 off) | crosshair hidden (hud.rs:622-623), health swirl hidden (716-724), ammo strip hidden (798/824) | present |
| `*(viewer+4) = 0` and the viewer-state stores | the monitor's state 2 + watched-sensor model (monitor.rs:76-89, `viewing()` 119-121) | present (state model, no viewer objects exist in the port) |

### Switch_Activate 0x1b02a8, SS_Activate 0x1a9ea8, PCQWorm_Activate 0x1b0840, SP_Activate 0x1dc338

| Game call | Port site | Verdict |
|---|---|---|
| Switch: toggles `switch_channels[ch]`, stamps `switch_channels_time` | `UseObjects::player_activate`'s switch arm (nf-game/src/useobj.rs:422-457, the 0x61 refusal kept) and the through-SP arm 372-418 | present (pure level-logic, no subsystem calls) |
| SS: `*(data+0x22)=1` + `switch_channels[ch]=1` | `Ss::activate` (nf-game/src/ss.rs:77-82) | present |
| PCQWorm: state 0→1 | useobj.rs:463-469 (install writes in its tick 239-307) | present |
| SP: dispatches `GT_Activate` / `Switch_Activate` with the facing gate | useobj.rs:339-361 (+ the facing dot test per `SP_Activate` 0x1dc338:51-53, fed `body` from drone.rs:16327) | present |

### Lock_Activate 0x1aed00

| Game call | Port site | Verdict |
|---|---|---|
| `Sound_Play3D(100.0, -1.0, -1.0, psVar1[3], …)` — the locked rattle (key required, none held) | the port keeps the two sound words (`Lock::key_channels`, nf-game/src/lock.rs:67-73) and plays nothing at arm time (drone.rs:16337 documents the skip) | **MISSING but unobservable**: every 0x2E lock placement on the disc carries 0xffff in both sound words (0700000a/0b/0c/08/11/12/14/15/46 swept — the [D] citation lives on `Lock::key_channels`'s doc, lock.rs:60-73), so the game's own call plays no sound on real input. The state side of the arm (state→3 keypad arm, `GlobalVars`/user save) is present: drone.rs:16326-16362 → `Keypad::arm`. |
| `Sound_Play3D(100.0, -1.0, -1.0, psVar1[2], …)` — the unlock sound (key held) | same — see above | same verdict (unobservable on the disc) |

The lock family's OTHER sounds are present: the decryptor chimes (`Lock::pending_sounds`, lock.rs:108-112, played scene.rs:2753-2774) and the laser-burn unlock 0x249 (lock.rs:169-172).

### Door_Activate 0x135be0

| Game call | Port site | Verdict |
|---|---|---|
| `Sound_Play3D(100.0, -1.0, -1.0, *(u16*)(data+100), obj+0xc0, …)` — the push-against-door sound (only on the approach-side arms: `data+0x76==1 && dot<0` or `==2 && dot>0`) and on the `ch_lock`/locked-flag refusals | `Doors::activate` (nf-game/src/door.rs:690-759): the `snd_hit` word (placement param 7, door.rs:405) played on the refusal gates and both approach arms (door.rs:695-701, 713-719) | present |

### GunImp_Activate 0x132f00 / GunImp_Deactivate 0x132c18

| Game call | Port site | Verdict |
|---|---|---|
| `Player_ChangeSubState(player, 0xc)` | no substate 0xc in the port's model — mounted-ness is `GunImp.active` (nf-game/src/gunimp.rs; the mount at drone.rs:15874-15878) | n/a (internal state; the visible behavior — control taken from the player — is modeled) |
| `Player_SetCamMode(bloke, 0xe)` | the port treats the mount as the same first-person case (nf-view/src/scene.rs:3379-3381) | present-equivalent |
| `Player_Disable(player, 1)` | per-tick input/trigger zeroing + walk gate (scene.rs:2805-2814, 3381) | present (stand-in, as the monitor) |
| `Input_SetAction(slot, 0xe, 1)` | the use key doubles as action 0xe; `gunimp_tick`'s `use_key` dismounts (drone.rs:16009-16027) | present |
| `*(player_obj+0xfa) |= 0x44` | the port's object model carries no flags word here | n/a (internal) |
| `Player_WeaponNone(player)` | `Gun::go_to_hands` at the mount (nf-view/src/scene.rs:5438; gun.rs:197-204 saves the held weapon) | present |
| `Camera_CalcViewAngles(0x3f060a92 ≈ 30°)` (SP) / `0x3f860a92 ≈ 60°` (MP) | `view_fov` (nf-view/src/lib.rs:100-105): 30° while a gunimp is active in SP, else `GAME_FOV/zoom`; consumed by the walk camera (lib.rs:5734, 7301) | **present — the motivating gap is already fixed on master**, tested by `0700000d_gunimp_door_live::the_mount_narrows_the_view_and_the_dismount_restores_it` (asserts 0.5235988 mounted, π/3 dismounted, on real input from the level start) |
| Deactivate: `Player_SetCamMode(0)`, substate restore, `Player_Enable` | implicit in the dismount (the walk resumes from the carried pose, drone.rs:16009-16027 doc) | present-equivalent |
| Deactivate: `AnimObj[99] = AnimObj[100]` (weapon restore) | `switch_request = saved_weapon` on dismount (scene.rs:2810-2813) | present |
| Deactivate: `Camera_CalcViewAngles(0x3f860a92)` (60°) | `view_fov` falls back once no gunimp is active (lib.rs:100-105) — covered by the same live test's second half | present |
| Deactivate: `Mat_GetDir(player+0x90)` → `cart_to_spherical` → zero `player_obj+0x50/+0x58` | the port keeps the aim as `bond.yaw/pitch` (gunimp.rs:117-122) and zeroes only the velocity (drone.rs:16026 — the game's `Vec_Zero(bloke+0x50/+0x10)` [M 0x132c18:44-45]) | n/a (the game's spherical store is discarded by its own zeroing two instructions later; nothing visible remains) |
| Deactivate: `Sound_Stop(*(data+0x98))` | the fire-loop handle stopped at dismount (drone.rs:16019-16025) | present |

### Car_Activate 0x131c48 / Car_Deactivate 0x131d88 (the tank — DRIVING, skipped per the task)

`Player_ChangeSubState(0xb)`, `Player_SetCamMode(0xd)`, `Player_Disable(1)`, `Sound_Play3D(…, 0x60a / 0x10b)`,
`Player_WeaponNone`, `Camera_CalcViewAngles(60°)`; deactivate: the restore, `Sound_Stop`,
`Explode_Create(…, 0x2d6210, 0x6000052, …)`, the `TankSpawns` matrix copy, `build_LinkToRoom`. The port's
`Car` state machine exists (nf-game/src/car.rs:108-407, the sound ids and the spawn-matrix copy defined)
but the module header (car.rs:16-21) documents the whole 0x28 walk as remaining sim work — nothing wires
it into play. **Skipped as driving**, recorded here so the next car task starts from this list.

### GT (turret) 0x1816f0 / GT_Create 0x17ffa0 / GT_Delete 0x181678

| Game call | Port site | Verdict |
|---|---|---|
| GT_Activate: state machine only (no subsystem calls) | `GTurret::activate` (nf-game/src/gt.rs:465-479, the fold→RESTORING), dispatched drone.rs:15976-15993 | present |
| GT_Create: `Script_Load`/`Script_Play`/`Script_Update` ×2, the MP duplicate-destroy `Explode_Create` | `from_placement` (gt.rs:303+); the port's turret behavior is modeled directly (GTEvent fire sounds drained drone.rs:17367-17414); the MP duplicate-destroy is MP-only | present (SP); MP-only explode n/a |
| GT_Delete: `Script_Free` ×2 | `delete` (gt.rs:502-505) | present |
| The mount side (`Player_SetCamMode(0xf)` via `ronin_view`, the 0x53 re-equip, pane 18) | `ronin_view` (drone.rs:15954-15970 → scene.rs:2514-2516), the hand-back at scene.rs:2832-2835, pane 18 hud.rs:658-677 | present |

### Trigger_Activate 0x1b2318 (level flow)

| Game call | Port site | Verdict |
|---|---|---|
| `Text_AddMsg` "locked" | `Trigger::activate_locked_message` (nf-game/src/triggers.rs:725-732) | present |
| `Music_Event(3,1)`, `GS_PauseGame(1)`, `ResetMap_LevelToLoad`, `GameFlow_PushState(…)` | the `level_exit` load path (triggers.rs:834-847 documents the carry; drone.rs:16442-16443 sets `level_exit`, the view loop loads) | present (the port's load performs the flow; the music event rides the transition) |
| `Player_RamSave()` (the exit checkpoint) | the port's own progress model (no memory-card ram-save) | n/a (save model differs; the exit itself is the checkpoint) |
| the `switch_channels` writes (types 1/2/10) | `Trigger::activate_request`'s channel writes + the cancel-channel gates (triggers.rs:734-779) | present |

### Player_Activate 0x1a5170 (the dispatcher)

The head-sphere use pick (`Collide_SphereIntersect` radius 1.0 at head+look) and the class switch
(0x1c door, 0x1d trigger, 0x25 SS, 0x28 car, 0x36 GT, 0x38 monitor, 0x3a lock with the facing gate,
0x47 gunimp, 0x49 PCQWorm, SP via tag 0) are the port's use-pick walk (drone.rs:16326-16470 +
scene.rs:5407-5440) — every class arm's counterpart is in the table above. The creepwall pre-arm
(`*(bldata+0x95f) == 6` → `Player_CollCreepWall`) is the port's creepwall path.

### The remaining *_Create / *_Delete sweep

`Break, Creature(+Delete), Drone(+Delete), DroneSpawner(+Delete), NDrone2_CreateNinjaEyes/CreateObj,
Emitter(+Plist), Env, Env_Tree, Mine, Shooter, Copter(+Delete), Hurt, Occlude, MiniSub(+Delete, driving),
Lightning, Cloud, RainBox, Swoosh, Corona, DynamicObject, Apocalypse, CreepWall, ThirdIcon, Wire,
Grapple, Ladder, Pickup(+FromSet/+Simple), Bullet_Delete, Explode_Delete, Destroy_Create` —
**none contains a camera/HUD/input/sound/player-state call** (regex sweep per file, spot-reads of
Grapple/Mine/Shooter/Ladder/Pickup confirmed pure construction). Their gameplay sounds/effects live in
the Update/Activate paths, which are outside this task's scope and covered by their own tasks (pickup
take sounds: drone.rs:16815-16878; ladder grab: scene.rs:3395-3409). `Door_Create`'s starts-open
channel write and `Switch/SS/Trigger` channel writes are pure level-logic, present in the port
(door.rs:448-452, useobj.rs, triggers.rs). `control_delete_object` 0x133178 dispatches the per-type
destructors (`control_funcs` table) — all audited above via the type list; its `MP_objectBeingDeleted`
arm is MP-only.

## Fixes

**None needed.** Every camera/HUD/input/weapon/sound/player-state side effect a player can see or hear
in the Activate/Deactivate/Create/Delete set is present in the port. The two candidates examined and
closed:

1. The named `GunImp_Activate` 30° view — already fixed on master (nf-view/src/lib.rs:100-105 `view_fov`,
   with the [M] citation in its doc comment) and covered by a real-input live test
   (`0700000d_gunimp_door_live::the_mount_narrows_the_view_and_the_dismount_restores_it`).
2. `Lock_Activate`'s two sounds — the only MISSING entries in the table — play nothing in the game
   either: every shipped lock carries 0xffff in both sound words [D, the 10-level sweep cited at
   lock.rs:60-73]. Adding the calls would be dead code on real input.

## Verification

- This branch's only commits are docs (WORKLOG + this report) — **no code changes**, so the suite
  obligation has nothing of mine to regress: the run below is master's own baseline.
- The full four-crate suite was launched in the background (`python3 tools/test/suite.py`, NF_ISO set)
  at 18:48 UTC from a cold target: at hand-off (19:03 UTC) it was still in its build phase (target
  1.9 GB, runner alive, no per-binary lines yet). The runner kept going after hand-off; the driver's
  batch run and the next session on this branch can read its total from `suite.out` / `../suite.log`.
  Nothing on this branch can turn a suite binary red that master itself does not.

## What's left

- The Car (tank) / MiniSub family is unwired in the port (driving; its own task) — the six
  Car_Activate side effects above are the checklist for it.
- The port models the mounts (monitor, gunimp, GT) with per-tick input zeroing instead of a
  `Player_Disable` flag and with draw-side camera realization instead of viewer objects — equivalent
  in play, documented at each site; a future refactor to a shared mount-state model would retire both
  stand-ins.
- MP-only arms (GunImp 60° MP FOV via `MPSettings._384_4_`, GT duplicate-destroy explode,
  `MP_RegisterPickup`) are unmodeled (the port's MP is separate work).

## Live checks for the main agent

Both visible verdicts are headless-verified; a render would make them player-visible:

1. `--bond` at the 0700000d deck gun, mounted (the plate → E walk): the view should be the narrowed
   30° mount view (compare with the standing view at the same spot — the horizon should show ~twice
   the standing width). Expected from `view_fov` [M lib.rs:100-105].
2. A monitor console view (e.g. 0700000a's security monitor): the screen should show the watched
   camera's pose with the pane-16 HUD and no crosshair/health/ammo panes [M 0x1b0058:43-49; the port's
   SecCam pane hud.rs:642-657].

## ntfy failed (partial)

curl: `curl: (28) Connection timed out after 20002 milliseconds`. python3 urllib:
`urlopen error [Errno 101] Network is unreachable`. The wget fallback exited 0 (empty body, treated as
delivered).

## Usage

Session token figures visible to me (read from the Task tool result metadata of the two research
subagents this session): subagent 1 (decompile audit) prompt 50461, completion 9873, total 60334;
subagent 2 (port map) prompt 149888, completion 7452, total 157340. My own session's context window,
input/output token totals and any credit or cost figures: not visible to me.
