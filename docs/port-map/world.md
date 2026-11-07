# World-family port map

Every `Door_*`, `Break_*`, `Destroy_*`, `Lock*`, `PC*`, `Trigger_*`, `Sensor*`, `Camera*`
function in ACTION.ELF (SLUS-20579 NTSC-U) — 74 in total; the other decompiled programs
(DRIVING.ELF, the main binary, SFX.IRX) contain none of these prefixes. One line each:
PORTED with the port's code location, ABSORBED (the port implements the equivalent under
its own system), or MISSING with the reason. A comment does not count as ported; the code
was opened for every row.

Method: the family was enumerated from the decompile filenames
`work/decomp/ACTION.ELF/<address>_<Name>.c` (the per-function address is that filename's
leading hex — rows here carry names only, because the push guard rejects some hex strings
in tables). Each function's exact address was then searched across `crates/nf-game/src`,
`crates/nf-formats/src` and `crates/nf-view/src`; a citation only counts when it sits on
or directly above implementing Rust logic, and the logic was read for every ambiguous row.
Game-side reachability came from grepping the decompile tree for callers by name; the
per-tick rows below are dispatched through the game's per-class handler tables (which is
why they show no named C callers) — the port's equivalent is the `Drones` world-object pass
(`Drones::tick_inner`, `collide_step`, `update_player_bullets`) plus the nf-view scene tick.

## The MISSING rows (2)

**`Lock_ReLock` — missing on this master checkout; the fix ships on `s/port-map-player`.**
The port has the real logic: `crates/nf-game/src/lock.rs::Lock::re_lock` (returns the key
id to unequip) and `Lock::update_burnt_open` (the `Lock_Update` case-2 guard: a kind-0
decryptor lock re-arms the moment its out channel drops; kind-1 laser-burn locks never
do). Both carry a unit test. But on master **no production code calls either** — the sim
ticks every other `Lock_Update` case (burn in `update_player_bullets`, idle refill, the
case-1 channel hold, the keypad cases in `Keypad::tick`, the decryptor scan in nf-view's
decryptor pass) and case 2 alone is dead outside `#[cfg(test)]`, so a burnt-open decryptor
lock whose channel drops never re-locks in the game. The exact fix — the keypad law that
equips the key on solve, re-arms kind-0 through `Lock_ReLock` on the channel drop and
clears the ring, red-then-green on the real level path — landed on the sibling branch
`s/port-map-player` (finished, awaiting merge). It is deliberately **not duplicated here**
to avoid a two-branch merge conflict on the same lines; this row is the map's record.

**`Cameras_PostLoad` — missing by design.** Reached from `ResetMap_Load`; draws the
3-player split-screen divider sprite. The port has a single viewer (documented at
`crates/nf-game/src/mp/rules.rs`), so the divider has no port path. Render-side; the main
agent's render pass owns anything visible here.

## Table

| Name | Status | Port location / proof |
|---|---|---|

Statuses: **ported** = the port's code for it at the cited location (folded = folded into
a caller's body, address cited on that body); **absorbed** = the equivalent exists under
the port's own system, citation on the mapping or the implementing item; **missing** = see
the section above.

### Break / Destroy

| Name | Status | Port location / proof |
|---|---|---|
| Break_Kill | ported | `crates/nf-game/src/breakable.rs` `Breaks::kill`/`kill_inner`/`kill_by_bullet` — dead flag, sound roll, glass-shatter vs debris loop |
| Break_Update | ported | `breakable.rs` `Breaks::bullet_hit` + `drone_step` — the per-tick contact walk, health drain, kill below zero |
| Break_ApplyDamage | ported (folded) | health subtract in `breakable.rs` `bullet_hit`; the explosion route `drone.rs` `run_beam_explodes` case for the breakable class |
| Break_DoBreak | ported | `drone.rs` `Drones::drain_script_requests` (`Request::DoBreak` arm); NIS event decode in `nis.rs` `Nis::event` |
| Break_Create | ported | `breakable.rs` `Breaks::create` — the breakable class from placement (kind, health, bullet filter) |
| Destroy_Smash | ported | `crates/nf-game/src/props.rs` `Props::smash` — latch, dead mark, death-channel write, scripted blast by type |
| Destroy_CollisionHandler | ported | `props.rs` `Props::bullet_hit` (+ `destroy_colour_tick`) — bullet-contact damage drain to smash, hit-flash colour step |
| Destroy_Create | ported | `props.rs` `Props::create` destroyable arm — health, mask, kind, blast damage, kill channel |
| Destroy_Update | ported | `props.rs` `Props::channel_kills` — the external-kill-channel arm; the countdown arm is unmodeled (its frames start at or below zero from shipped placements, so the game cannot fire it) |

### Door

| Name | Status | Port location / proof |
|---|---|---|
| Door_Create | ported | `crates/nf-game/src/door.rs` `Doors::create` — the door class from placement (params, path keys, straddle bit) |
| Door_SetupSwing | ported (folded) | kind-0 arm of `Doors::create` — swing re-quantisation, closed half-pose, inverse matrix, swing sign |
| Door_SetState | ported | `door.rs` `Doors::set_state` — state write, open/close sounds, open-channel motion mirror |
| Door_Interp | ported | `door.rs` `Doors::interp` — position advance, end clamps, close snap, spline/keyframe pose |
| Door_ReactToHit | ported | `door.rs` `Doors::react_to_hit` — reverse-on-contact for moving doors; dispatched from the collide step |
| Door_Activate | ported | `door.rs` `Doors::activate` — toggle/move with one-way guard; driven by the player use path and AI door-open |
| Door_DoMsg | ported | `door.rs` `Doors::do_msg` — the aim-ray "door locked" hint with its latch and multiplayer gate |
| Doors_Calc | ported | `door.rs` `Doors::calc` (+ `begin_pass` group-state wipe) — master-to-group state copy, per tick |
| Door_Update | ported | `door.rs` `Doors::update` (+ `blocked`) — channel/auto-close arms, lock label rising edge, group accumulate |
| Door_Link2Portal | ported | `crates/nf-view/src/scene.rs` `link_doors_to_portals` — gate flag = `Door::straddle` set in `Doors::create`; the game's only caller is the doors-to-portals builder at map reset |
| Door_IsClosed | ported | `door.rs` `Door::is_closed` (+ `Doors::closed_flags`) — kind-1 always closed, grouped reads group state, else state test; the port also gates portal visibility with it |
| Door_IsLocked | ported | `door.rs` `Doors::is_locked` — lock channel zero means locked; consumed by the port's nav pathing (`nav.rs`) |
| Door_Init | ported (folded) | `Doors::create` + group-state zeroing, driven by `drone.rs` `Drones::attach_level_id` (game: map game-init) |

### Lock / PCQWorm

| Name | Status | Port location / proof |
|---|---|---|
| Lock_Create | ported | `crates/nf-game/src/lock.rs` `Lock::create` ← `Drones::attach_level_id` — decryptor/laser-burn kinds, operative auto-unlock, burn timer |
| Lock_Update | ported (case 2 unwired on master — see MISSING) | the split per case: burn in `drone.rs` `Lock::burn` (`update_player_bullets`), idle refill `Lock::idle_tick`, case-1 channel hold (`drone.rs` lock hold loop), case-3 keypad arm, case-4 `Keypad::tick`, case-0/5 decryptor scan in `crates/nf-view/src/scene.rs` `Lock::decrypt_tick`; case 2 exists as `lock.rs::update_burnt_open` with a unit test and no production caller on master |
| Lock_Activate | ported | `drone.rs` `Drones::player_use_doors` — state gate, key-ring slot, facing gate (cites the address on the block) |
| Lock_Unlock | absorbed (distributed) | the unlock state + out-channel writes live in `Lock::burn`/`decrypt_tick`/keypad solve/`create` (lock.rs, drone.rs); the grant-the-key-to-the-player half is the player-family gap fixed by the keypad law on `s/port-map-player` |
| Lock_ReLock | missing on master | `lock.rs::Lock::re_lock` has the real logic (returns the key id) + unit test, but the game's only caller (the case-2 burnt-open re-arm inside `Lock_Update`) is unwired; the exact wiring ships on `s/port-map-player` — not duplicated here |
| Locks_Init | ported | `lock.rs` `KeyCodes::init` ← `drone.rs` boot — the keycode digit table, with the documented local-RNG deviation |
| PCQWorm_Create | ported | `crates/nf-game/src/useobj.rs` `UseObjects::create` → `PcQworm` — the Q-Worm target PC placement kind |
| PCQWorm_Update | ported | `useobj.rs` `UseObjects::tick` — state 0 waits for the Q-Worm gadget bullet, state 1 reveals the keycode and writes the out channel, state 2 rests; the revealed-keycode drain sits in `drone.rs` |
| PCQWorm_Activate | ported | `useobj.rs` `player_activate` ← the player use path — armed placements only, state 0 to 1 |

### Sensor

| Name | Status | Port location / proof |
|---|---|---|
| Sensor_Create | ported | `crates/nf-game/src/sensor.rs` `Sensors::create` — sweep keys from the path blocks, cone angles, channels |
| Sensor_Update | ported | `sensor.rs` `Sensor::update`/`Sensors::update` ← `Drones::tick_inner` — sweep → alert (31 ticks) → track → sight hold → out channel → return/disable |
| Sensor_InCone | ported | `sensor.rs` `Sensor::in_cone` — range + cone + line-of-sight gates, player-only (the cone/sphere intersection modeled) |
| Sensor_SetBeam | absorbed | `sensor.rs` `Sensor::beam_ray` + the inline lens write inside `Sensor::in_cone` (both cite the address); runs every tick inside `Sensor::update` |
| Sensor_Init | absorbed | the game body is list-pool init only; the port's `Sensors`/`Monitors` are vectors built at attach (documented in the sensor.rs header) |
| Sensor_ApplyDamage | absorbed | the game body is an 8-byte empty stub (read this session); sensors take no damage in game or port (documented in the sensor.rs header) |

### Trigger

| Name | Status | Port location / proof |
|---|---|---|
| Trigger_Create | ported | `crates/nf-game/src/triggers.rs` `Trigger::from_params` + `triggers::build` — out, inputs, mask/lock/hide slots, mask default |
| Trigger_Calc | ported | `triggers.rs` `triggers::calc` — bond-moment scan, the two per-level latch copies, touch clear+hit OR, process-touch window; ticked from the world pass |
| Trigger_Update | ported | `triggers.rs` `Trigger::update` — lock return, lock-msg cooldown, key-spline step, per-type cases, load-level auto-fire |
| Trigger_Hit | ported | `triggers.rs` `Trigger::hit` — player/drone contact arm and the class-5 bullet arm with the per-weapon filter |
| Trigger_Activate | ported | `triggers.rs` `Trigger::activate_request` + `activate_locked_message` — cancel channels, key gate, fallback/success writes, movie latch |
| Trigger_LoadLevelCreate | ported | the load-level arm of `from_params` + `build` — forced type, mask, key/gate/fallback/success slot re-homing |
| Trigger_MoviePlayer | ported | the movie arm of `from_params` — slot re-homing and the movie-code plumbing (next-level resolve) |
| Trigger_Touch | ported | the touch arm of `from_params` + the touch pass in `calc`; script-entity touch triggers built in `drone.rs` |
| Trigger_TouchOnce | ported | the touch-once arm — the one-shot latch/delete gate modeled as the glist field |
| Trigger_MultiplexIn | ported | the mux arm of `from_params` + `update` case AND — the wrapper slot shift and the prefix scan |
| Trigger_MultiplexOrIn | ported | same mux arm + `update` case OR |
| Trigger_MultiplexSIn | ported | same mux arm + `update` ordered-AND window (the lower-bound-only window has its own test) |
| Trigger_MultiplexOut | ported | same mux arm + `update` case OUT — the out channel mirrored into every target input both ways each tick |
| Trigger_Type | absorbed | the port stores the type as `Trigger.kind` and reads it directly (citation on the field doc) |
| Trigger_Locked | ported | `triggers.rs` `Trigger::locked` — the load-level key-gate arm vs the generic lock arm, plus the locked-message cooldown |

### Camera

| Name | Status | Port location / proof |
|---|---|---|
| Camera_Enable | absorbed | `crates/nf-view/src/lib.rs` `overlay_for` (HUD-gate half) + `drone.rs` `hud_frozen` — the draw/HUD-freeze flag; the render-state alloc half is not applicable (port has one viewer) |
| Camera_Reset | absorbed | `nf-view/src/lib.rs` viewer-init doc + `scene.rs` load glue — the zero-init/matrices/radii folded into the port's own viewer init |
| Camera_UpdateGlbVars | ported | `drone.rs` `Drones::tick` fade advance (`ViewFade`) + `crates/nf-view/src/hud.rs` `fade_cover` — the exact unclamped-alpha state machine |
| Camera_CheckLocation | ported | `nf-view/src/scene.rs` `eye_cell` — the point-in-cell chain with keep-previous-on-miss; load-bearing for the portal world cull |
| Camera_CalcViewAngles | absorbed | `nf-view/src/lib.rs` `Camera::view_proj` + `Sim::view_aspect`; the view-angle constants live on the frustum helper (`drone.rs`) |
| Camera_Create | absorbed | `nf-view/src/scene.rs` load-time viewer/world glue + `lib.rs` `Camera` — slot alloc/viewport subsumed by the single viewer |
| Camera_CreateCameras | absorbed | `nf-view/src/scene.rs` load setup; the 1/2/3/4-player viewport layout is deliberately unported (noted in `menu.rs`) |
| Camera_Update | absorbed | `nf-view/src/scene.rs` Sim tick — the underwater shear/swing arms (both cite the address); the shake-decay/vector-draw half is replaced by the documented stand-in (see `Camera_Shake`) |
| Camera_Orbit | ported | `crates/nf-game/src/player.rs` `Bond::orbit_camera` + `scene.rs` `Sim::orbit_camera` — phase advance, eye-on-circle, sphere pull-in, radius ease |
| Camera_Shear | ported | `nf-view/src/scene.rs` `Sim::camera_shear` + `camera_shear_step` — sine skew, phase wrap, the global-blur flag |
| Camera_Set | absorbed | `nf-view/src/lib.rs` per-frame camera build; the rot/pos mapping documented in `player.rs` (cites the address) |
| Camera_PushStates | absorbed | `drone.rs` — the saved enable pair becomes the port's script-camera / HUD-frozen edges (cites the address) |
| Camera_PopStates | absorbed | same pair + the hand-back clear in `drone.rs` `tick` |
| Camera_UpdateAll | absorbed | the port's single-viewer equivalents: `scene.rs` Sim tick + `lib.rs` frame loop; the game-flow skip mirrored in `menu.rs` |
| Camera_SetFade | ported | `drone.rs` fade arm + `ViewFade` + `hud.rs` draw — rate from duration, negative = fade-in, the refuse-while-ramping gate |
| Camera_ApplySwing | ported | `nf-view/src/scene.rs` `Sim::camera_apply_swing` + `camera_swing_step` — the phase/sine pitch-yaw-roll deltas and zoom factor |
| Camera_SetToPlayer | absorbed | `nf-view/src/lib.rs` first-person build (head + yaw + pitch), death-cam mapping in `scene.rs`; the matrix composition is documented in `drone.rs` |
| Camera_PosYRot | ported (factor-1.0 arm) | `nf-view/src/scene.rs` `cam2_camera` — the yaw snap; the game only reaches it with lerp factor 1.0 (the behind-Bond camera), so the generic lerp is not modeled |
| Camera_PosXRot | ported (factor-1.0 arm) | `nf-view/src/scene.rs` `cam2_camera` — the pitch snap, same caller and caveat |
| Camera_SetUpdator | absorbed | the callback mechanism is replaced by `scene.rs` `Sim::view_override` + the monitor state machine in `crates/nf-game/src/monitor.rs` (the security-camera view) |
| Cameras_PostLoad | missing by design | the 3-player divider sprite has no port path (single viewer) — documented at `crates/nf-game/src/mp/rules.rs`; render-side, see MISSING above |
| Camera_Shake | ported (formula; view on stand-in) | `drone.rs` `camera_shake_amount` — the exact store formula (range, quadratic falloff, truncation, cap); still marked not-wired, so the live view uses the documented stand-in decay. Wiring it is a visible change and needs the main agent's render |

## Counts

74 rows: **55 ported** (3 folded; `Lock_Update` case 2 unwired on master, the two camera
rot snaps modeled at factor 1.0 only, and `Camera_Shake` formula-only with the view on its
stand-in all carry recorded caveats), **17 absorbed**, **2 missing**
(`Lock_ReLock` — fix in flight on `s/port-map-player`; `Cameras_PostLoad` — by design).

Notes:

- The per-tick rows (breakable/destroyable/door/sensor/lock/PCQ-worm updates, the door
  contact reaction, doors calc) are dispatched by the game through per-class handler
  tables under the world-object pass; the port's equivalent dispatch is the `Drones`
  pass in `drone.rs` plus the nf-view scene tick. That is why several rows have no named
  decompile caller.
- The camera family is engine-side and reached every frame; most of it is absorbed into
  the port's single-viewer system in `crates/nf-view` (no `camera.rs` exists by design).
- Cross-family footnotes found by the sweep, owned elsewhere: the key-grant/equip law
  (`Player_EquipKey`/`UnEquipKey`, player family — fixed on `s/port-map-player`); the
  script-camera enable/restores cited under `Camera_Enable`/`PushStates`/`PopStates` are
  absorbed by the script-camera system.
