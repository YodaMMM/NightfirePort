# MIS01 (Paris Prelude): the opening sniper section

What the original shows (PCSX2, reached through the real menus: Play without saving, Operative,
Paris Prelude) [L 2026-11-20]: the mission does not start behind the wheel. Bond holds the
**Sniper Rifle** (HUD "24 / Sniper Rifle") on a moving vehicle above the riverside road and
shoots at pursuing cars below (black saloons and a truck), at night with lamp-lit facades. The
driving executable is already loaded at this point, so the section runs in DRIVING.ELF.

Evidence tags: [M] DRIVING.ELF decompile (EE addresses), [D] disc / ELF data bytes, [L] live
capture, [S] suggested (not proven). Port: `nf_formats::drive_script` (the script decoders) and
`nf_game::driving::pov` (weapon table, loadout, gunner, mode flag), `nf_game::driving::flight`
(flight path, triggers, timers, view limits, shot test); disc test
`crates/nf-game/tests/driving_sniper_live.rs` (needs `NF_ISO`).

## 1. The vehicle is the helicopter

- The `MSet` name `"helicopter"` [D] is the player vehicle name: `FUN_00204d18` stores it at
  mission +0x1b44 (the `lotus`/`playervan`/… fallbacks are player cars) [M].
- `data\sim\attrib\pvehicle\helicopter.atr` [D]: `RENDER_FILENAME.s=helicopter.crp`,
  **`SECONDARY_TYPE.s=vanquish`** — the Vanquish is the helicopter's secondary vehicle
  (the boot loads `SECONDARY_TYPE` as the secondary, driving-load-path.md §3.6).
- Bond is not in the car model: he is a separate actor (`AICharacter`, 0x170 bytes, vtable
  0x3696e0) built by `FUN_00123950` and mounted on the vehicle's `AIC_HSNIPE_POS` /
  `AIC_HSNIPE_ROT` points, model `sniper`, anim set `6.a_snipe.b_snipe.70` [M+D 0x3275d0..].
  `paradis_car` / `c_domm1` are an AI element (AIEl record 23, §4), not the player.

## 2. The `MSet` is the weapon loadout (correction)

drive-mission-scripts.md §3 read `MSet` +0x2c/+0x4c as "objective channels". They are weapon
slots: `FUN_00205498` calls `FUN_0020d378(DAT_0033a530, i, count, …)`, and `FUN_0020bbe0` fills
`DAT_0033a530`+0x10 by copying the 32-entry weapon table `FUN_0020f2b8` builds at 0x4165c0
(stride 0x54, 0x4165c0..0x417040) [M]. Entry fields used: +0x04 name, +0x14 flags (bit 0
owned, bit 2 clip dry), +0x18 rounds carried, +0x1c clip, +0x20 clip size, +0x24 last shot
tick, +0x34 (the MSet third array; consumer not traced), +0x38 shot counter on, +0x3c refire
ticks = `(int)(seconds[i] * fps)` from 0x33a030 [D], +0x44 sound stem.

MIS01 enables weapon **2 Missile, 30 rounds** and **26 (0x1a) Sniper Rifle, 24 rounds**
(clip sizes 4 and 30 from MSet +0xcc) [D]. The load selects the first enabled index past 0
with a count: the Missile [M 0x205498]. `FUN_0020d378` overrides the clip size and caps the
clip at the count only while the POV mode is on (manager +0x350) [M].

## 3. The POV mode flag and the weapon swap

`DAT_0033a024`+0x350 (the "auto-drive camera" / POV mode) is written only by `FUN_00206398`
[M]; its callers are scripted events:

| event (kind) | handler | effect |
|---|---|---|
| `ESetAutoDriveCamOn` (77) | 0x1576e8 | when off: camera start `(angle/360, pitch/360)`, mode on, gunner shown |
| `ESetAutoDriveCamOnMovie` (234) | 0x157838 | the same, but `FUN_00206398`'s third argument 0 skips its `FUN_00167d80`/`FUN_00166cb0` calls (not traced) |
| `ESetAutoDriveCamOff` (76) | 0x157460 | mode off, gunner hidden |
| `ESetAutoDriveCamOffMovie` (233) | 0x157540 | gunner hidden, mode off |

Each frame the weapon manager `FUN_0020c078` swaps: mode on with a non-POV weapon current →
stash it (+0x0c) and select the first owned POV weapon (ids 0x17..0x1f) with rounds; mode off
with a POV weapon current → restore the stash (or weapon 0, or the first owned) [M]. So the
Missile is stashed and the **Sniper Rifle** becomes current — the HUD's "Sniper Rifle" [L].
A POV kind is `id - 0x17` (sniper = 3) [M 0x123950, 0x20c078].

## 4. The script that runs the section

The `Map ` record's window holds `el` (event list) and `sr` (string) instance records besides
the chunks; `Trgr` +0x18 and `Rule` +0x0c are `el` references, event parameters use `sr`
references (layouts in `nf_formats::drive_script`) [M walkers 0x22c890 / 0x2095f8, D refs].
Event kinds index the factory table 0x36fda0 (`FUN_001757b8`); names come from each class's
typeinfo accessor [M]. MIS01 [D]: 868 lists, 2787 strings, 754 triggers, 114 rules; every
trigger/rule reference resolves; four `EPlaySystemAnim` strings are absent from the archive.

1. **Start.** World init `FUN_0022cd30` fires every trigger whose list holds `EStartMission`
   (0x16) [M]: only trigger **684** (list 685) [D].
2. **Sniper intro.** Trigger **685**, radius 2.5, 0.9 units from 684 at (778.3, −7.6, 207.4),
   fires list 686 [D]: `EControlToSpline` (the player vehicle onto spline `AISp` offset 0),
   **`ESetAutoDriveCamOn(-60°, -20°)`** (the only one in the mission), throttle 0.71,
   `EChangeStage(19)`, music stream `snipeintro`, objectives 2800–2803 (current 2800),
   letterbox on, four power-ups disabled, `ESetReflectiveObject(AIEl 23 = paradis_car)`.
3. **The phase.** `EChangeStage(n)` stores the next rule group (`FUN_00149828`: mgr+0x10);
   `FUN_00206340` runs the default group and the current one each update, then makes the next
   one current; `ECallStage(n)` runs group n once (`FUN_00206310`) [M]. Stage 19 holds rules
   3, 45, 46, 79, 80, 85, 105, 107, 113 [D] (AI commands to the pursuing elements, timers,
   stage calls). Trigger 712 (544.9, −1.9, 458.7) changes to stage 5; trigger 727
   (−320.6, −10, 117.9, radius 74) to **stage 13** [D].
4. **Hand-over.** Stage 13's rule **34** (`SRuleSpeed`) fires list 789 [D]: letterbox off,
   audio mixes, `ESetAutoDriveCamOnMovie(0,0)`, `EEndControlToSpline`,
   **`ECustomTransition("paris")`**, `ESetAutoDriveCamOff`, **`ESetVideo(paris2.mad,
   paris2.sub)`**, `EResetPlayerCarPos(trigger 683 at (−321.2, −0.1, 66.3))`, AI elements
   off. A second route, trigger 741 (radius 80 near the start) → list 742, carries the same
   transition, video (no subtitle file), spline end, mode off and reset [D]; which route a
   normal run takes is not proven.
5. **`ECustomTransition("paris")`** `FUN_0014b020` [M]: the player car swaps through its vtable
   +0x294 / +0x8c (not decomposed — the swap target is the `.atr` secondary, the Vanquish
   [S]); the MSet copy loses every POV weapon (enabled 0x17..0x1f cleared), then weapon 0
   ("Gun", stem `mntmach`) gets 500 rounds — or, when `DAT_0032e690` is set, the Missile count
   is zeroed — and `FUN_00205498` re-registers from the copy (inventory reset first).
   `DAT_0032e690` is set only on `paris_mis01` with PTP+0x618 == 1, +0x984 == 0, +0xa18 == 0
   [M 0x174bf0:269-276]; what that state is has not been traced.

The ported run over the real data (`the_phase_runs_on_the_real_script`): load → Missile
current; list 686 → mode on, Sniper Rifle current with 24 rounds; list 789 → mode off, rifle
gone, Gun 500 + Missile 30, Missile current, video `paris2.mad`, car reset to trigger 683.

## 5. Bond's gunner (fire, reload, scope)

The update `FUN_00118af0` calls vtable slot `0x44 + 8*state` (state = +0x4c) [M]:

| state | handler | behaviour |
|---|---|---|
| 8 | 0x1243f0 | spawn/ready anim; at its end → 2, then a pending switch |
| 1 | 0x124240 | switching: waits on 8 (ready) or 0xd (put-away, then the ready anim) |
| 2 | 0x124348 | idle loop; a pending reload starts; a changed scope wish toggles |
| 9 | 0x124440 | scope toggle (scope-up anim 7 / scope-down anim 6 for the sniper) |
| 10 | 0x124570 | scoping; at anim time (4.9, 5.3) going up: model hidden, scope view on |
| 11 | 0x124650 | fire anim; sniper bolt sound `SFX_sniperbolt` at time (13.9, 14.1) |
| 12 | 0x124770 | reload anim |
| 13 | 0x124800 | put-away; at its end → 0 with the new kind |

Anim indices per kind (sniper column): ready 4, idle 1, put-away 3, reload 5, fire 0/0, scope
up 7, scope down 6, spawn 2, bank 0xe [D 0x3274b0..0x327650].

Fire (`FUN_0020de20`, POV branch) [M]: no rounds → nothing; Sniper Rifle with manager +0x5ec
set and the scope object's +0x2c clear → `SFX_DryFire`, no shot; held during the switch
cooldown (+0x50), before `last + refire` (sniper 5 ticks at 60 fps), or while the gunner is
not idle; a shot spends one round from the carried count and the clip; an emptied clip sets
flag bit 2. The empty-clip loop in `FUN_0020c078` starts the gunner's reload once
`last + refire − 5 < now` and clears the bit; `FUN_0020e978` refills the clip to
`min(carried, clip size)` once the gunner is idle again. A reload request (`FUN_0020e868`)
needs an idle gunner, a clip below its size and below the carried rounds.

Scope input (`FUN_001be038` cases 0x38 press / 0x3a release) [M]: press raises the scope
wish; with PTP+0xa44 set (toggle) a press while up lowers it, otherwise the release lowers it.

## 6. The flight path (`AISp` → `pt  ` tracks)

Port: `nf_formats::drive_script::{AiSplineRecord, PathTrack, TrackChannel}` (decode),
`nf_game::driving::flight` (`track_pose`, `PathFollower`).

- **`AISp`** is 0x70-byte records (`FUN_002da590` walks it at that stride and fixes up the
  reference at +0x40) [M]: +0x00 a 4x4 row matrix, +0x40 the track (an `sr` string naming
  `CARP::<<Map>>::{pt  nnnn}`, index in hex), +0x44 the name, +0x6c "identity offset"
  (`FUN_002db690`: non-zero → the controller's offset matrix +0x70 is the identity) [M+D].
  MIS01 has 19 records and 19 `pt  ` records; record 0 is `path_Helicopter02` → `pt  0001`,
  record 1 `path_Heli_Hover` → `pt  0003`; every record has +0x6c = 1 [D].
- **`pt  `** (`FUN_002db370`) [M]: +0x00 the length in 60 Hz frames, +0x04 the channel count,
  +0x0c the rotation channel, +0x10 the translation channel (-1 none), +0x18 the name; the
  channels at +0x40, stride 0x14: name offset, time-table offset, key offset, key count,
  flags. Flag bit 1 = quaternion keys, bit 2 = one float per key, bits 0 and 3 select other
  modes (`FUN_002db188`, `FUN_002bfb50`) that no MIS01 track uses [D]. `pt  0001`:
  1264 frames, 131 rotation keys, 150 position keys from (809.6, 4.2, 215.0) to
  (-143.0, 13.5, 542.4) [D].
- **Evaluation** [M]: the key is the cached one (or a neighbour) when
  `times[c] <= t < times[c+1]`, else `lower_bound(t) - 1` clamped (`FUN_002dae58`,
  `FUN_002dbfa0`); fraction `(t - t0)/(t1 - t0)` (0 when `t0 >= t1`); positions blend
  linearly on four lanes, quaternions by the shortest-arc normalised blend `FUN_002cc018`
  (sign picked on `|b-a|² - |b+a|²`); the quaternion becomes matrix rows by `FUN_002cc0c8`
  (decoded from the VU words: row 0 = (1-2yy-2zz, 2xy+2zw, 2xz-2yw), …), the position is
  row 3.
- **Who follows it**: `EControlToSpline` (`FUN_0014ae60`, player when word 1 = 0) → the
  player's AI vehicle, vtable +0x24 = `FUN_00131508` (state 3) → `FUN_0013dfd8` builds the
  spline controller `FUN_0013da80` (or resets it, `FUN_0013dcc0`). The controller's path
  element: rate (+0x58) **0.5**, start time 0, base flag 0, playing [M]. The follower
  (`FUN_001b1258` reset, `FUN_001b14b8` update) [M]:
  - clock = `tick * (1/fps) * 60` (`FUN_001b0f48`), so the track's time is in 60 Hz frames at
    any frame rate;
  - speed (+0xbc) moves toward the desired speed (+0xc4) by the step (+0xc0) after a hold
    count (+0xc8); reaching a desired 0 stops it;
  - time += (clock delta) × speed × 0.5; past the length it wraps into the next track (+0x9c,
    which the reset sets to the same track, so a track loops), or stops at the end when none;
  - pose = `BASE_TURN · M(t)` (`FUN_002cbb88`; the base 0x381f70 is a quarter turn of the
    model axes, the position row is untouched), velocity = Δposition × fps;
  - the vehicle (state 3, `FUN_00131620`) copies the pose and takes the follower's velocity.
- **Throttle events** [M]: `EPathSetDesiredThrottleStatic` (116, `FUN_00152cb0`) sets the
  desired speed (list 686: **0.71**); `EPathChangeThrottleStatic` (215, `FUN_00151dc0`)
  sets hold / step / desired and resumes; `EPathSetTimeStatic` (153) sets time = length ×
  fraction; `ESetNextAISpline` (209, `FUN_00158370` → `FUN_001b1838`) sets the next track.
- **The run** (`the_helicopter_flies_the_phase`, 60 Hz, every armed player-class trigger the
  pose touches, no rules, no shots) [D through the port]: 18 triggers fire along the river in
  a fixed order (the same for body radius 0.5 and 2.0): music / speech / stage calls, the
  throttle changes (723: 0.6, 698: 0.85, 700: 0.68, 735: 0.65, 703: 1.0, then 701: 0 at
  0.01 a frame), 712 at tick 973 (its list changes to stage 5 and plays a camera anim),
  569 (a camera spline), 742 at tick 3435 (`ESetNextAISpline` → `path_Heli_Hover`); the
  helicopter stops at tick 3554 (about 59 s) at the end of `path_Helicopter02` in x/z, on the
  hover track.

## 7. The trigger contact rule

Port: `flight::{Triggers, trigger_touches, class_mask}`. Every frame `FUN_0022f0c0` runs
`FUN_0022d240` for each vehicle body [M]:

- the body's class (+0x6d) picks a mask: 1 → 4, 2 → 8, else 0x40; the callers of the body
  ctor derive the class from an index (`< 0` → 2, 0/1 → 1) [M]; that 0/1 are the human
  players is [S]. MIS01's flight triggers carry mask 4, trigger 727 (stage 13) mask 8.
- candidates come from the collision grid cells the body overlaps; a trigger fires when flag
  bit 0 is on, 0x2000 off, 0x400 only while the mission is live (`DAT_0033aa88`+8, cleared by
  `EPlayerLose`/`EPlayerWin`), the mask matches, and `FUN_0022eb28` passes [M];
- `FUN_0022eb28` [M, VU words decoded]: shape byte +0x11; the reach is
  `(body radius + trigger radius)²` against the squared distance, strictly inside, three
  lanes for a sphere (2), x and z otherwise; flag 0x800 is one-way (`dot(+0x30, velocity)
  >= 0`); a cylinder (3) then needs the body's y band to overlap `[y, y + height(+0x14)]`; a
  box (1) runs a box-box test `FUN_001f5eb0` (not ported);
- the fire (`FUN_0022c890`) runs the list, then flag 2 clears bit 0 (one-shot) [M]; at load
  flag 0x200 clears 0x400 (`FUN_0022cbe0`); `EEnableTrigger` / `EDisableTrigger` set / clear
  bit 0 [M 0x14bcd8 / 0x14b790].

## 8. What ends the phase (rule 34)

Rule type 9's typeinfo reads "10SRuleSpeed", but its evaluator `FUN_0020a940` (descriptor
0x397aa8 +0x14) watches a **mission timer** [M]: payload +0 the timer (one of ten 0x18-byte
timers at manager +0x150), +4 the direction, +8 the threshold; v = ticks / fps; a falling rule
fires when v drops below the threshold having been at or above it (the stored previous value
starts at threshold - 1). The timers (`FUN_00209c88`): value = base + rate × (now - start)
while running; `ESetTimer` (47, ctor `FUN_00159178`) sets base = `(int)(seconds × fps + 1)`
and start = now; kind 48 (ctor `FUN_0015ca58`) starts it with a rate (MIS01: -1) [M].

Trigger 727 (−320.6, −10, 117.9, radius 74, box, mask 8, starts disabled) runs list 728:
kill two AI elements, timer 5 = 4 s counting down, stage 13 [D]. Stage 13's timer rules then
fire at +8 ticks (rule 36: AI elements on), +32 (rule 35: camera anim, sim rate 1) and
**+182 ticks (rule 34 → list 789, the hand-over)**; rule 33 (4.9) never fires on this route
[D through the port, `the_hand_over_timer`]. 727 is enabled by trigger 740's list, 740 by
711's (mask 0x40) [D]; which body class touches 711 and 740/727 (mask 8, an AI car, by the
class mapping) is not traced, nor are the stage-19 rules (health / timer rules that disable
trigger 712 and pick other branches).

## 9. The gunner's view (aim limits, zoom, sway)

- **Limits** [M `FUN_001a49e0` load, `FUN_001b6c60` clamp; D camera.ini
  `[CopterAutoDriveCam:Arm0]`, the auto-drive camera of the `helicopter` car]: up to five
  yaw sets (`maxYaw`/`minYaw` in turns, clamped ±0.5) each with a pitch band
  (`maxPitch`/`minPitch` as sines, clamped ±0.999; bound = `-asin(v)/2π`). The clamp finds
  the set around the yaw, interpolates the pitch band between neighbouring sets, slides the
  view along a band edge that is steeper than the yaw span, then clamps yaw and pitch. The
  copter: yaw sets -0.315, -0.09, -0.07 turns (−113°, −32°, −25°: the view is held to the
  helicopter's side), pitch bands [−0.0965, 0.1350], [−0.0451, 0.0889], [−0.0451, −0.0451]
  turns. The script's start view (−60°, −20°) is inside. Port: `flight::{AimArm,
  ViewLimits::clamp_view}`.
- Other arm values read (with the loader's conversions): `aimFov` 20, `minFov` 2,
  `LockArmToCar` 1, deadzones 0.15 / 0.18 rad, autoaim 0.09 / 0.12 rad, cursor speeds
  0.48 / 1.8 (× π/2) and 0.3 / 3.0 [D].
- **Zoom** (`FUN_001b72f0`) [M]: unscoped, the target FOV is the camera's default; scoped, at
  least `minFov`; a non-zero zoom rate (camera +0x2ac) moves the zoomed FOV within
  `[minFov, default]`, stored only above 2.0. Port: `flight::scope_fov`. What writes the rate
  is not traced.
- **Sway**: `FUN_00124848` (the gunner's matrix) lags kinds 1-2 behind the vehicle and
  jitters kinds 1/4/6 while firing; the sniper (kind 3) gets neither, its matrix is the
  camera's [M]. Port: `flight::gunner_sway`.

## 10. The shot and its hit test

- `FUN_0014ca60` (the shot spawner), case 0x1a: origin = the camera position, direction =
  the camera forward (`FUN_001bcca0`), damage = the slot's +0x34 = MSet +0x14c[26] = **50**
  (the Missile's is 100), projectile `FUN_00198e80` with speed 400 capped to 220 [M+D].
- `FUN_00198e80` → `FUN_001998c0` (vehicle pass over 0x90 bodies): `FUN_00211098` tests the
  ray against each body's sphere (centre +0x10, radius +0x7c): no hit from inside; the
  segment is 1000 units; a hit when the segment point nearest the centre is within the
  radius. `FUN_0019a548` keeps the nearest by the distance to that point. Vehicles then get a
  box test (`FUN_001f5eb0`), not ported. No vehicle hit → the range is 150 and the world pass
  `FUN_00199f88` runs [M]. Port: `flight::{ray_sphere, nearest_hit, shot_speed}`.
- The hit's effect (`FUN_0019a710`): the target's vtable +0x1c with the damage, 0.25 and
  type 2, then the impact effect `FUN_0019d5a8` [M]; the car's health and kill behind that
  vtable call are not traced. The POV fire itself (`FUN_0020de20`) only sets the gunner's fire
  state (`FUN_001240c8`) and the manager's firing flag; where that flag reaches
  `FUN_0014ca60` is not traced.

## 11. Not proven / not ported

- Rules: only rule type 9 is ported; the stage-19 rules (health, timer, prog rules that
  disable trigger 712 and switch branches) are not, so the flight run is the no-shots, no-rule
  path. `ESetSimRate` (569's list sets 0.8) is not applied to the follower clock.
- Which body classes the helicopter and the pursuers have, the helicopter's body radius, the
  box trigger shape, the vehicle box refinement of the shot, the damage → health → kill of
  the pursuing cars, the camera's stick-to-angle integration (deadzone / autoaim paths of
  `FUN_001b72f0`), the camera matrix build from the angles, and the zoom rate.
- The event queue order (`FUN_00175730` allocates, the class destructor runs the effect): the
  port runs a list in its stored order [S]. The `el`/`sr` references → runtime pointers loader
  is not traced (the references resolve against the `Map ` window's instance records [D]).
- Not ported elsewhere: the pursuer AI commands (`EAICommand` → `AICo`), the movie, the
  vehicle swap itself, the zoom-fraction blend (+0x160), the scope object's own guards. The
  port's driving view (`crates/nf-view/src/driving.rs`) still starts in the Vanquish.
- Live checks that would settle the open parts: break on `FUN_0022c890` during the flight and
  list the triggers (expect §6's order; 712 tells whether stage 19's rules disabled it);
  read the helicopter body's +0x6d and +0x7c; break on `FUN_0020a940` returning 1 for rule 34
  (expect 182 ticks after 727); watch the follower's +0xac / +0xbc (expect +0.355 per frame
  after list 686).
