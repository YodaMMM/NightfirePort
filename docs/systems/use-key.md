# Use key — the census of everything the use key can operate

*Evidence tags: [M] decompile (file `work/decomp/ACTION.ELF/<addr>_<name>.c`, checked against the
disassembly where the C looks incomplete), [D] disc bytes, [L] EE snapshot, [S] suggested
(port-only choice). The port-only choices below are marked [S]; everything else cites the game.*

## 1. The input path (button → activation)

The use key is action **0xE** (style-0 map: □ / R2, docs/systems/input.md §3). The per-action
flag byte lives at `0x2a39cc + 14 + pad*0x158`; bit **0x4 is the fresh-press bit**
[`Input_Action` 0x17f208 masks it and returns value×100].

Chain, every tick [M]:

1. `Player_Update` 0x193600:413 → `Player_WeaponFiring` 0x1a5460 (arg 0).
2. 0x1a5460 calls `Input_Action(v, 0xE, 4)` (fresh press); skipped while the player substate
   is 10 (in a cutscene) — otherwise → **`Player_Activate` 0x1a5170(player, BLData)**.
3. The bool return (OR of the `Door_Activate` results + a weapon hack-bit) suppresses that
   tick's weapon fire — using and shooting never land on the same tick.

The port rides the same edge: `Sim::step_bond`'s `use_key` argument is the fresh press of
action 0xE (crates/nf-view/src/scene.rs `step_bond`), and `Act::Use` in the walker pulses it
one tick in six to respect the edge trigger.

## 2. The use probe (`Player_Activate` 0x1a5170)

- Special arm first (:67-98): while the third-icon context byte `BLData+0x95F` == 6 (written by
  class-0x42 ThirdIcon placements, type 0xFB), the nearest **class 0x2B creep wall** within
  SqDist 1000.0 is grabbed (`Player_CollCreepWall`) and the probe returns.
- Sphere pick (:100-108): centre = `Player_GetHeadPos + Mat_GetDir(view)*1.0` — head plus one
  unit along the exact view direction; `Collide_SphereIntersect` **radius 1.0**; the player's
  own object excluded; every hit is walked (HITDATA+0x58 = object), so walls do not stop the
  list. **There is no global facing cone** — directionality comes from the 1-unit forward
  offset plus the per-class checks below.
- Dispatch switch on `obj+0xFF` (:126-175): `0 (obj+0xE4 != 0)` → `SP_Activate`, 0x1C → Door,
  0x1D → Trigger, 0x25 → SimpleScript, 0x28 → Car, 0x36 → GunTurret, 0x38 → Monitor,
  0x3A → Lock (with an inline `dot >= 0` gate), 0x47 → GunImplacement, 0x49 → PCQWorm.

## 3. Class census [M]

| class (obj+0xFF) | placement kind [D] | activate fn | conditions (exact) | effect on success | port status |
|---|---|---|---|---|---|
| 0x00 script entity (NIS op-7 object, `obj+0xE4` = owner) | 0xD9 ScriptPlayer | `SP_Activate` 0x1dc338 | owner dispatch 0x36→GT, 0x39→Switch (dot(entity_dir, player−entity) ≥ 0), else generic: SP mask (+0x2E) & 4; if mask&1 the out channel (+0x30) must already be ON | `SP+0x2C` = 1 latch; `SCRIPTINFO[+0xC+4*mode]+0x9C` = 0 (restart); `SP_Update` 0x1dbda8 consumes it → state 2, countdown 5, `Script_Play`; on completion writes `switch_channels[out]` (state-1 path = 1, state-3 path = 0) | `useobj.rs player_activate` + `nis.rs` (origin-placed SPs have no entity and are unreachable — documented gap, elevator-button.md) |
| 0x1C door | 0xDB | `Door_Activate` 0x135be0(…,1) | locked byte DOORINFO+0x77 set → nothing; lock ch (+0x68) set but OFF and closed → hit sound only; busy (state&0x10) → nothing; approach-side one-shot vs activator (dot logic :71-95) | swing away from the activator's side / close toward 0.5 (±0.3 margin), slide reverses; 2-tick timer; returns "did something" | `door.rs player_use` — the pick sphere (r 1.0 at head + viewer dir [M 0x1a5170:100-108]) vs the door's LIVE world sphere (obj+0x80/obj+0x8C; the candidate is sphere-vs-sphere [M Collide_PickObj 0x1eae80, the 0x1ec940 refinement cannot remove the node, the 0x1a5170 walk has no distance gate] — docs/systems/door-use.md), sound events on the open path |
| 0x1D trigger | 0xDC / 0xE8-0xEF | `Trigger_Activate` 0x1b2318 | activator class 3 only (:17). Type 1 TouchOnce: mask (+0x36) & 4 → ch[out(+0x20)] = 1 + timestamp. Type 2 Touch: mask&4, out ≠ 0 → ch[out] = 1. Type 10 LoadLevel: **no mask gate**; ch 99, 0x62, 0x61, 0x60 must be OFF; lock ch (+0x26) set & OFF → label 0x2000021 msg, 180-tick cooldown, return; else loads the level. Type 0xD: mask&4 → driving-state pushes | type 10 tail: `Music_Event(3,1)`, `GS_PauseGame(1)`, `ResetMap_LevelToLoad`, `GameFlow_PushState(300.0f,7,0xff)`, type := 99, `Player_RamSave` (exit-hold.md) | `triggers.rs` (walk-in kinds + `player_use_triggers` E path, drone.rs:10053) |
| 0x25 SimpleScript | 0xE2 | `SS_Activate` 0x1a9ea8 | armed `SS+0x22 != 0` (param 0 ≠ 0 at create; re-armed to 0x40 after each run when param 1 = 0, disarmed when param 1 = 1) **and** mask (+0x1E) & 4 | `SS+0x22` := 1 (playhead advances this tick — KeyFrame/Spline per param 6) and **`switch_channels[SS+0x10]` := 1 immediately**; `SS_Update` 0x1a9948 advances the playhead while the channel is ON, sound (+0x14) on the 0x40→advance edge | `ss.rs Ss::activate` (mask&4 gate), state-0x40 channel sweep in `Ss::tick` |
| 0x28 car | 0xDA | `Car_Activate` 0x131c48 | car state +0xF4 == 0 | player substate 0xB, cam mode 0xD, car state := 5, `Player_Disable(,1)`, sound 0x60A/0x10B, `Player_WeaponNone` | `car.rs` (driving levels; no STORY on-foot level carries a car) |
| 0x36 gun turret | 0x27 | `GT_Activate` 0x1816f0 | state ∉ {0,1,2,3,6}; instance+0xEC == 0 (no current user) | `inst+0xDC` = 0, state := 2 → `GT_TakeControl` 0x181740 | out of scope on-foot (no 0x27 placement in the story levels) |
| 0x38 monitor | 0x30 | `Monitor_Activate` 0x1b0058 | state +0xF4 == 0 **and** activator class 3; a camera with `cam+0x5E == monitor+0xF6 (param 0)` must exist | state := 2, camera/viewer stored, `Camera_SetUpdator`, `Input_SetAction(0xE,1)`, `Player_Disable(player,0)`, HUD pane 0x10 on; exit = E again in `Monitor_Update` 0x1afcc8; shot → explode and restore | `monitor.rs activate` (reach 2.2), state/watched exposed |
| 0x39 switch | 0x29 | `Switch_Activate` 0x1b02a8 (via SP_Activate owner dispatch; **dot(entity_dir, player−entity) ≥ 0**) | not (one-shot param 4 set && done-count ≠ 0); **not (activator class 3 && ch == 0x61)**; `obj+0xF4 == switch_channels[ch]` (in sync); lock ch (param 5) absent or OFF; `SP_GetState(SP)==0` 0x1dc900 (idle); ch (param 0) ≠ 0 | **toggle** `switch_channels[ch]`; timestamp when it becomes 0; `Switch_Update` 0x1adea8 resyncs `obj+0xF4`, bumps +0x18, `SP_UnPause` (the button's own anim); param 2 = auto-flip delay when channel ≠ param 3 | `useobj.rs SwitchObj` + `player_activate` (:330-345 implement the gates) |
| 0x3A lock | 0x2E | `Lock_Activate` 0x1aed00 | **dot(lock_dir, player−lock) ≥ 0** (inline in Player_Activate :154-164); activator class 3; `MenuManager_Exists(1)==0`; state 0 or 5; key ring `BL+0x70C + param 1` | mode 0 & **no key** → state 3 = the keypad menu (`MenuManager_Create`, digits vs KeyCodes; match → `Lock_Unlock` 0x1b0650). mode ≠ 0 & no key → locked sound (+0x06) only. **With the key (any mode)** → state 1 + unlock sound (+0x04), `Lock_Unlock` equips key param 1, spline anim, state 2; relock ch (+0x08) dropping → `Lock_ReLock` 0x1b0750 | `lock.rs` — E without the key plays sounds only and must NOT unlock (pinned negatively in the census test); decode/burn paths are the gadget chain (decryptor.md) |
| 0x47 gun implacement | 0x34 | `GunImp_Activate` 0x132f00 | gun state +0xF4 == 0 && player substate (`obj+0xF6`) != 4 | player substate 0xC, BL+0x878 = gun, cam 0xE, gun state := 1, `Player_Disable(,1)`, `Input_SetAction(0xE,1)` (E exits), `Player_WeaponNone` | `gunimp.rs` + scene.rs gunimp_activate |
| 0x49 PC-QWorm target | 0x35 | `PCQWorm_Activate` 0x1b0840 | inst+0x04 (param 2 armed flag) ≠ 0 && state == 0 | state := 1 → `PCQWorm_Update` 0x1aee20: if param 0 ≠ 0 prints KeyCodes[param 0] (label 0x2000014) and sets `KeyCodes[param 0][4] = 1` (code learned); **out ch (+0x02, param 1) := 1** + timestamp; state 2. Alternative: Q-Worm bullet type 0x58 contact when param 3 ≠ 0 | `useobj.rs PcQworm` (E when e_usable ≠ 0, install then channel) |
| 0x2B creep wall | 0x3F | `Player_CollCreepWall` (via 0x1a5170:67-98) | third-icon context `BLData+0x95F == 6`; nearest class-0x2B within SqDist 1000 | wall-hug grab | `creepwall.rs` + scene.rs icon_ctx 6 path |

Adjacent but **not** use-key (bullet/touch/proximity operated — listed so the census is
complete): FuseBox 0x3C / kind 0x31 (SP mask + `FuseBox_Register` 0x1b0970, laser-cut wire),
Pickup 0x2F (touch), Hint 0x43, ThirdIcon 0x42 (writes the context byte), SoundTrigger 0x40,
MusicTrigger 0x44, SSys_Monitor 0x1ad998 (channel phase sequencer), DynObject 0x3B (channel
driven), NIS op 0xF `Script_EventHandler` 0x1d94c0:172-175 (the register-callback op behind
GT_Register/FuseBox_Register).

## 4. Per-level placement census [D]

Kind counts per on-foot story level, from the disc census (docs/levels/*.md; the live sim lists
agree — the census test derives its targets from them). E-usable = carries a class the probe can
reach with the use key.

| level | E-usable placements [D] | census test operates |
|---|---|---|
| 07000001 | monitor #105 (group 1) + camera sensor #280; doors ×5 (01.md:61-67) | monitor, door |
| 07000002 | monitor #323 (param 0 = 2) + cameras #406/#423 (02.md:39,55) | monitor |
| 07000003 | none (ch 0x96 is script-owned) (03.md) | absence |
| 07000004 | none — mux #46 only; switch/lock template data dead at origin (04.md:61-68) | absence |
| 07000005 | none (05.md:108-110) | absence |
| 07000006 | SimpleScripts ×28 but every activate mask is 0x1/0x2 — touch/bullet driven, none E-usable [D the live parse]; ScriptPlayers ×4 | absence |
| 07000007 | none explicit; doors ×15 (07.md:123-127) | door |
| 07000008 | safe: locks #207/#208 (out 134/135 → mux 136), SS #205 safe door (ch 136) + #217 bolt (ch 137), SP #286 (08.md:130-142) | lock (negative: E must not unlock) |
| 07000009 | SP #70 elevator (mask 4, out 0x39, real panel entity), alarm switches ×11 (0x7C/0x7D — busy one-shots, latch 0), #71 at origin, SS LIFTDOORS ×2 (ch 57) (09.md:131-146) | switch; SP elevator is the elevator_buttons_09_live test |
| 0700000a | PC #688 armed (param 3 = 1) but **e_usable = 0** — the Q-Worm gadget/bullet target, not an E target [D the live parse; PCQWorm_Activate 0x1b0840 needs param 2 ≠ 0]; switches ×9 — 8 ch-0x61-locked (E-rejected), #643 at origin; monitor #783 at origin (0a.md:99-110) | PC-E absence pin; switch ch-0x61 negative |
| 0700000b | switch #249 (ch 9), lock #178 (out 47), PC #297 (0x4C) (0b.md:161-172) | switch, PC, lock negative |
| 0700000c | locks ×5 (#139-142 out 14-17, #182 out 84), SS ×8 (vents ch 18, #544 secret door ch 8) (0c.md:77-103) | SS, lock negative |
| 0700000d | doors ×6 (2 E-swing), SS #112 (ch 0x2b, mask 1 — not E), E-panel trigger (TouchOnce out 43, mask 4), DynObject buttons #113/#114 (0d.md:89-106) | door; E-panel trigger (the SS-negative is not observable here — the panel drives the SS's channel; 13 carries the class row) |
| 07000011 | PC #216 "Computer to hack" (E param 2 = 1, out 0x38), lock #177 (out 0x12), SS vents ×2 (mask 4) (11.md:136-158) | PC, SS, lock negative |
| 07000012 | locks ×2 (out 0xa — the decryptor pair), monitor (grp 10, 5 sensors), PC ×1 (out 0x34, E), SS ×8; the ch-0x58/0x59 E-switches #962/#965 sit on another map and the sim attaches use-objects from the main map only [M Drones::attach_level → UseObjects::create(main_map.placements), drone.rs] — **multi-map gap, see the report** (12.md:116-133) | PC, monitor, lock negative; switches asserted absent from the main map |
| 07000013 | SS #149 Shutters2 (ch 0x23, mask 1 — not E), SP #427 at origin (13.md:84-87) | SS mask-1 negative |
| 07000014 | switches #605/#632 → 0xDE/0x4F (busy one-shots that never fired — latch 0, still activatable [M 0x1b02a8's +0x18 gate]), doors ×55, SPs ×15+ (14.md:50-64) | switch, door |
| 07000015 | KeypadLocked ×3 (#788→0x95, #765→0xf7, #711→0xf9) + doors #683/#689/#685 (15.md:89-95) | lock negative ×3 |
| 07000016 | switch #693 KikoDeathButton (0x90, gate 0x8d), E-panel triggers #597/#617/#684/#685 (mask 4) (16.md:119-131) | switch, E-panel trigger |
| 0700001b | ScriptPlayers ×78 (the Drake defeat chain), no switches (1b.md:51,82) | absence |

Wrappers 07000041/43/46/48/49/4a/4b are menu/MP hosts (4a has SP #42, mode 1, out 0x22) and are
out of the story census; 0700000e/f/10 have no archive on this ISO [D docs/systems/levels.md §7].

## 5. The census test

`crates/nf-view/tests/use_key_census_live.rs` — one test per story level. Each test loads the
level through `nf_view::load` (the real disc path), derives its targets from the live sim lists
(the parsed placements), places Bond beside one target **per class present** (the documented
place-once stand-in for the walk), settles, aims the view direction at the target and pulses the
use key one tick in six (the fresh-press edge [M `Input_Action` 0x17f208]), then asserts the
game's effect:

- switch → `switch_channels[ch]` toggles (`drones.channels.on(ch)` flips),
- PC-QWorm → the install completes and the out channel turns ON,
- SS (mask&4) → the drive channel turns ON and the playhead advances,
- E-panel trigger (mask&4, non-load kind) → the out channel turns ON,
- door → the door leaves the closed state (`pos01`/`state` move),
- monitor → state 2 with a watched camera,
- gun implacement → `active` and player substate 0xC,
- lock → **negative**: E without the key must not unlock and must not write the out channel
  [M `Lock_Activate` 0x1aed00 — no key, no unlock; mode 0 opens the keypad menu instead],
- switch with ch 0x61 → **negative**: class-3 activator is rejected [M 0x1b02a8],
- SS with mask without bit 4 → **negative**: E is a no-op [M 0x1a9ea8's mask&4 gate],
- levels with no E-usable placements assert exactly that (the absence is the census row).

Each operated target has a sister assertion: the effect flag is recorded OFF immediately before
the use press, so removing the press (or the operation) fails the test.
