# Lifts, elevators, platforms, cab doors — the census over every on-foot level

*Evidence tags: [M] decompile (file `work/decomp/ACTION.ELF/<addr>_<name>.c`), [D] disc bytes
(nf-extract dumps of the level archives), [L] EE snapshot, [S] suggested (port-only choice).
Every assertion about the game cites its evidence; port-only choices are marked [S].*

Task lifts-buttons-all-levels. Sweep of 07000001-0d, 11-16, 1b, 41, 4a. One table per level
(§5). The mechanism chapters first: what a lift IS in this engine, and the exact chain from a
button press to Bond standing on a moving cab.

## 1. What moves a lift [M]

There is no lift object class. Every lift/elevator/platform/cab in the game is one (or both) of:

1. **A SimpleScript (placement 0xE2, class 0x25)** whose keyframe/spline path moves the SS's own
   object: `SS_Update` 0x1a9948 interpolates the playhead along `SS+0x0C` (the placement's
   framelist) and writes the pose to the object; obj+0xFA bit 0x20 marks it moved each interp
   tick (docs/fields/ss.md). The 0700000a cab-mouth doors #524/#525, the 07000012 lift screens
   and `lft_LIFTDOOR1/2`, the 07000009 LIFTDOORS and the 0c winches are SS-driven.
2. **A ScriptPlayer (placement 0xD9, class 0x1B)** whose NIS script creates script entities
   (`Script_EntityStart` 0x1d8d78) and keys their matrices (`Script_Interp` 0x1d9a08 case 5/6):
   the 0700000b paternoster cab (entity 0x2000367 in script 0x06000048), the 07000008 gondola
   carriage (OBJECT_Carriage 0x0200038a in 0x0600004C), the 0700004a Tower-2 platform (entity
   0x20005b4 in 0x06000571), the 07000009 lobby cab pair (#21/#22, scripts 0x0600007E/0x0600007D [D placements dump])
   and the 07000016 shuttle-bay lift (#347, 0x060009E9).

Both land in the collision world wherever they are posed: `Collide_PickObj` 0x1eae80 tests any
object linked into a cel with collision data, whatever moves it [M world.rs:164 note]. The port
mirrors this: every script entity, every SS, and every door joins `set_dynamic` at its posed
matrix each tick (`Drones`' `script_parts`, drone.rs — entities excluding the trigger/volume/
pickup classes, ids `oi<<6|part`, doors bit 40, SSs bit 41).

## 2. The ScriptPlayer state machine, lift-relevant arms [M]

`SP_CreateScriptPlayer` 0x1db9a8 ends with a **prime beat**: every placed SP whose mode A
(SP+0x26 = param 2) is ∉ {0, 1} calls `SP_LoadScript` **at create, regardless of the trigger
mask**. `SP_LoadScript` 0x1dbc08 then branches on the mode (SP+0x26+2·cur):

| mode | what happens | state word |
|---|---|---|
| 1 | `Script_Play(si, 0)` — play once | 2 |
| 8 | `Script_Play(si, 1)` — **play looping** | **6** |
| else (64…) | `Script_Play(si,0)`; `Script_Update`; `Script_Stop` — the preload-then-stop | 1 |

Consequences the lifts depend on:

- **A mode-8 lift cycles from level load.** The 0700000b paternoster (#232), the 07000009 cab
  pair (#21/#22) and every Skyrail gondola (mask 0 included — the prime ignores the mask) are
  moving from the first tick; no trigger starts them.
- **A mode-1 lift plays once at create** (the 07000016 shuttle-bay lift #347: no writer for its
  out ch 0x50 exists on the disc and none is needed — the docs' "writer not found" question
  dissolves; ch 0x50 is its OUT marker, self-sustaining once on, and the drone that waits on it
  (param 3 = 0x50 [D]) is level furniture).
- **A mode-64 SP waits armed in state 1** (the 07000009 panel button #70, the 07000008 gondola
  #295): the state-1 arm `1 if trig` (SP_Update 0x1dbda8 case 1) starts the play with the loop
  flag from the mode (mode 64 → one pass).

The trigger mask (param 5) selects which `SP_Hit` 0x1dc470 bits count: **bit 0x1 = "the out
channel is ON"** (`SP_Hit` seeds its result byte with `switch_channels[out] != 0`), 0x2 = an
explosion in a stream object's hit chain, 0x4 = the use-key latch (`SP_Activate` 0x1dc338's
`+0x2C`), 0x8/0x10 = a class-3 player / class-2 drone touching a stream object.

While playing (state 2), SP_Update's **case-2 rule**: with mask bit 0 set and the out channel ON,
`SP_UnPause` 0x1dca58 runs every tick — it clears the script's delay word (SCRIPTINFO+0x9C = 0)
and latches SP+0x2C. **The out channel therefore releases the script's self-holds while it
holds.** The 0700000b lift's script holds itself 499 ticks at each end of its cycle (stream 3:
`Event{kind 3, params [0x1f4]}` at t0 and t630 [D nis dump]; event 3 → the delay = p0 − 1 [M
0x1da998's negative-delay arm]); `lift_stop_trigger` #245 (the roof-deck touch volume, out ch
0x75 [D trigger dump]) is the channel that releases them — with Bond on the roof the cab does
not linger at its end stops.

The state-0 start arm (`0 if trig` → `SP_LoadScript` + `Script_Play(si, 0)` — note **non-looping**
whatever the mode, 0x1dbc08's mode-1 arm shape) applies only to SPs that actually sit in state 0:
a mode-8 placement never does (the prime moved it to 6 at create).

## 3. Riding — how Bond moves with the cab [M]

`Player_CollisionHandler` 0x1949a8 (early-return when BLData+0x94a ≠ 0, :24 — disabled by a
cut-scene) ends at the feet query `Player_FeetOnPoint` 0x194188, and the ground object's motion
is inherited through **`Control_InheritVelocity` 0x133e88**: the ground's translation over the
last tick, plus the change of heading about the object's position as pivot. Bond standing on a
moving cab is carried — position and yaw — every tick he is grounded on it. The port: `World::
ground_motion` (world.rs `GroundMotion`) consumed in `scene.rs`'s step (gated on `!disabled`,
the 0x1949a8:24 gate; the relink segment starts at the previous tick's end position,
fix-swim-entry). Proven live on the 07000005 truck, the 07000008 gondola
(walkreal_07000008 `walk_the_gondola_ride_to_the_end`: "the carriage carries Bond"), the
0700004a platform (fix_finale_live) and the 0700000b paternoster (this task's
`lifts_ride_live::the_paternoster_carries_bond_from_load`).

## 4. Cab doors

Lift/cab doors are ordinary doors and SSs: swing/slide doors (class 0x1C, `Door_Activate`
0x135be0 via the use key) and SS leaves gated on a channel (docs/fields/ss.md §state machine —
the 0x40 state waits for `SS_Operate` = the channel half). The 0700000a cab-mouth doors
#524/#525 are SSs whose create-sweep (param 0 = 1 armed) poses them OPEN and holds the end pose
(wrap flag 1) — resolved by task 710a; walkreal_0700000a walks out through the open mouth and
asserts the sweep. The 07000009 LIFTDOORS_LEFT/RIGHT_ENDLEVELDOORS #2/#3 wait on ch 0x39 (57)
which the lobby panel button writes (docs/systems/elevator-button.md — the whole chain, task 700).

## 5. Per-level census

Kinds shorthand: SP = 0xD9 ScriptPlayer, SS = 0xE2 SimpleScript, DOOR = 0xDB, TRIG = 0xEA/0xEB
touch (once) trigger, SW = 0x29 switch, HP = 0xE4 hurt poly. "Port" = the port's status with
its proof (test name) where one exists.

### 07000001 Paris Prelude — no lift content [D docs/levels/07000001.md, whole-file sweep]
No lift/elevator/platform/cab placement, no lift channel. Vertical travel is the stairwell
(ladders) — ladder.rs.

### 07000002 The Exchange — no lift content [D 07000002.md]
Nothing lift-shaped; the walkreal route touches no cab.

### 07000003 Alpine Escape — no lift content [D 07000003.md]
One ladder; the rail/roof routes are geometry.

### 07000004 Enemies Vanish — no lift content [D 07000004.md:67,71-72]
Explicit absence: "lifts and their call buttons (no lift kind exists)" in the zero-kinds census;
"the cable car / gondola pieces: absent" (the carpark Limo parts are kind-0x19 static cels).

### 07000005 Mayhew A — no lift; the ride here is the truck [D 07000005.md:45-50]
The scripted ride is the truck chain (SP #37 0x06000A22 + touch plates → mux #316 → ch 0x1D) —
a vehicle, out of lift scope; its carry is the Control_InheritVelocity proof level
(truck_ride_to_the_castle_ends_the_level, level_reload_live).

### 07000006 Mayhew B — no lift content [D 07000006.md:40-47]
Kind census has no lift kind; the 28 SSs are touch/bullet-driven panels.

### 07000007 Mayhew C — no lift content [D 07000007.md:123-124]
Explicit: "no lifts/elevators, no buttons/switches (0x29)".

### 07000008 Mayhew D — THE GONDOLA (the cable car) [D 07000008.md:36-49,138-142]

| piece | id / kind | channel / script | the game does | port |
|---|---|---|---|---|
| cab-floor switch | #236 TRIG (Touch) | out 4, mask 8 | Bond on the boarding platform floor → ch 4 | ported (triggers.rs) |
| briefcase | #290 pickup 0xF0 type 4 | switch ch 74 = 0x4A | the take writes ch 0x4A | ported (pickup.rs) |
| boarding mux | #235 (0xEC) | out 1 = 0x4A AND 4 | both inputs → **ch 1** | ported (mux) |
| the ride | #295 SP, mode 64, mask 1, out 1 | script 0x0600004C (level bin; 53 streams, 1420 ticks) | armed at create (state 1); ch 1 → state-1 start → the carriage entity 0x0200038a + 43 parented parts ride (112.0, 32.4, −126.3) → (112.0, 18.0, −179.4); stream 46 writes **ch 0x53** at t1410 | ported; walkreal_07000008 `walk_the_gondola_ride_to_the_end` (Bond aboard the whole descent, ch 83/0x53 at the bottom) |
| bottom cinematic | #296 SP | starts on ch 0x53; its stream 10 writes ch 5 | the arrival cut-scene | ported (nis.rs) |
| the helicopter | #45 (0x2F) | gate ch 5, out ch 0x4B | ch 5 → the flight script → ch 0x4B → mux #276 → ch 100 | ported (copter.rs) |

No button/keypad operates the gondola — the "switch" is the cab floor plus the briefcase.

### 07000009 Tower A — THE MAIN ELEVATOR (the task-700 level) [D 07000009.md:46-96,135-147]

| piece | id / kind | channel / script | the game does | port |
|---|---|---|---|---|
| panel button | #70 SP, mode 64, mask 4, out 57 (0x39), stay 1 | script 0x0600059B | E on the class-0 script entity (the panel, art baked absolute at ≈(−18.1, −368.2, 25.0)) → SP latch → state 1→2 → **ch 0x39 = 1**; one-shot (deletes) | ported; `elevator_buttons_09_live` (task 700; docs/systems/elevator-button.md) |
| objective mux | #274 (0xEC) | ch 0x46 = AND(ch 0x39) | objective 2 "activate the main elevator system" | ported |
| cab doors | #2/#3 SS | wait ch 0x39, spline, one-shot | the LIFTDOORS swing/slide open on ch 0x39 | ported; elevator_buttons_09_live |
| inside-lift plate | #67 TRIG TouchOnce | out 0x32 | Bond in the cab → ch 0x32 | ported |
| exit | #63 TRIG 0xE8 LOAD_LEVEL | gate ch 0x32, key gate 0 | loads 0700000a (the ride IS the level transition; E works too — key gate 0) | ported (the elevator route is the pinned exit; the E-route through the closed doors is the use-pick origin gap, §7) |
| the cab | #21/#22 SP, mode 8, mask 4, out 0 | scripts 0x0600007E/0x0600007D [D placements dump; the level doc's 0x060005FE/5FD hashes are a mis-print] | **prime at create → the cab cycles from load** (idle loop; out 0 → E writes nothing) | ported (prime); pinned by `lifts_ride_live::the_09_cab_pair_loops_from_load` |
| shaft kill | — | — | the shaft is lethal geometry (HurtPoly family) | ported (hurt.rs) |

### 0700000a Tower B — the lift landing [D 0700000a.md:28,73,106,130-134]
Bond spawns on the landing OUTSIDE the cab mouths; the arrival plate #58 (out ch 0x47) completes
objective 3 "Take main elevator to office floor" at the spawn [L 0x47 on at load].

| piece | id / kind | channel | the game does | port |
|---|---|---|---|---|
| cab-mouth doors | #524/#525 SS | ch 0 (none), p0 = 1 armed, wrap 1 | the create-sweep poses them OPEN and holds (75-tick sweep [M 0x1a9c58:51]) | ported; walkreal_0700000a asserts the sweep and walks the mouth (task 710a — the old "missing writer" ignore premise was stale) |
| shaft fall | #745-749 HP | p1 = 1000 | the lift-shaft fall volumes at y −29.2 | ported; asserted live by the census test |

No lift SP/buttons on 0a; the ride here is the 09→0a level load.

### 0700000b Tower C — THE PATERNOSTER (the roof lift) [D 0700000b.md:65,136-141,164,168-169]

| piece | id / kind | channel / script | the game does | port |
|---|---|---|---|---|
| the lift | #232 SP, mode 8, mask 1, out 0x75, stay 1, p9 = 0 (no gate) | scripts 0x06000048 + 0x06000A09 | **prime at create → the cab cycles forever**: entity 0x2000367 (+ parented 0x2000421) keys y 11 → 48.44 → 11 over 1239 ticks at x 0, z ≈ 25, with 499-tick self-holds at t0 and t630 (stream 3 event 3 [D nis dump]); sounds 0x288/0x28c ride the cycle | ported (prime + Script_Interp + set_dynamic); `lifts_ride_live::the_paternoster_carries_bond_from_load` |
| the hold release | #245 TRIG Touch (roof deck, y 56.8..63.8) | out 0x75 | Bond on the roof deck → ch 0x75 → the case-2 rule releases the 499-tick holds every tick (the cab skips its end stops) | ported (state-2 UnPause); pinned by the same test's `with ch 0x75 the top hold releases` arm |
| mid-shaft plate | #171 TRIG TouchOnce | out 0x53 | the exec-floor boarding marker | ported |
| the ride gap (STALE) | — | — | docs/levels/0700000b.md:164 "the port does not carry a Bond standing on it" **predates the Control_InheritVelocity port**; the carry is proven (§3, this task's test). The walkreal's teleport stand-in for the balcony→exec leg remains (route shape, not the lift mechanism) | test added this task |

The exec→roof leg itself is the roof cut-scene (SP #177 0x060004FA: event 6 re-places Bond at
[−14.71, 58.00, −2.06] — inside #245's volume [D pack 0790000b]) — the lift is the way DOWN
from the helipad and the exec-floor transit, exactly the paternoster pattern.

### 0700000c power station 1 — no lift; winch SSs [D 0700000c.md:69,77,100]
Explicit absence of lifts/elevators. The `obj_stor*_pulley` winch SSs #351/#364/#377 (2-key, ch 0)
are the pulley cars — their driver (the SS_Operate self-latch on ch 0) is the fix-0c-vent2 open
item, documented there; not a Bond ride.

### 0700000d power station 2 — no lift content [D 0700000d.md, whole-file sweep]
The crane chain (Overhead Crane_touchplate #460 → ch 0x2b → shipping_handle SS #112 → SP #115)
is cut-scene machinery Bond never rides.

### 07000011 Tower2 A — no lift content [D 07000011.md:97-121,142,158-160]
The vertical route is the stairwell (walked by walkreal_07000011_live). No lift kind in the
census; no 0x29 switches at all.

### 07000012 Tower2 B — THE LIFT LOBBY (doors, screens, call buttons) [D 07000012.md:17-30,45-59,126-134]

| piece | id / kind | channel | the game does | port |
|---|---|---|---|---|
| call buttons (E-touch) | #699 / #702 TRIG mask 4 | out 0x58 / 0x59 | E → ch 0x58 (the shutter/screen cosmetic swap) / **ch 0x59 (the lift doors)** | ported; walk_07000012_live `the_lift_call_button_swaps_the_simple_scripts` |
| call buttons (switch twins) | #962 / #965 SW 0x29 | 0x58 / 0x59 | the same channels' switch twins — **not main-map placements**; `UseObjects::create` attaches from the main map only (drone.rs Drones::attach_level) → the E path misses them (the use-key-census multi-map gap, documented, unfixed — the E-touch twins carry the function) | gap documented (use-key.md §4, 12.md:133) |
| lift doors | #677/#678 SS `lft_LIFTDOOR1/2` | ch 0x59 | slide to x 3.07/1.34 on ch 0x59 | ported; the walk test |
| lift screens | #700/#701 SS | ch 0x59 | the liftopen/liftclosed screen swap | ported; the walk test |
| shutter pair | #657/#676 + #697/#698 SS | ch 0x58 | the cosmetic swap | ported; the walk test |
| objective console | #731 TRIG TouchOnce mask 4 | out 0x49, lock 0x87 | "elevator override control" (the 0x41 inverter holds 0x87 low while 0x49 is on) | ported |
| shaft entrance | #683 TRIG Touch type 2 | out 0x1F, mask 8 | Bond in the shaft mouth → ch 0x1F gates exit #669 → 0700004a | ported; walkreal_07000012 |
| objective 0x04000038 ch 0x4D | — | written by **0700004a's** plate #277 | "Escape through the elevator shaft" completes on the NEXT level's drop | cross-level; walkreal_07000012 documents the handover |

No floor selection — one call button per channel; the ride is 4a's script.

### 07000013 Tower2 C — the FAKE lift [D 07000013.md:39-42,46-47,87,104-105,117,138-151]

| piece | id / kind | channel / script | the game does | port |
|---|---|---|---|---|
| fake-lift panel | #161 SP, mode 64, mask 4, out 0x23 | script 0x06000959 (25 ticks) | E the west-wall panel → entity light-up (0x02000A2C/0x02000A2B swap) → ch 0x23 | ported; gate13_live (task 716b — §4's "no writer" reading superseded) |
| Shutters2 | #149 SS | ch 0x23, spline, 2 keys, rate 240 | the front-entrance gate leaf rises +4.824 y on ch 0x23 | ported; gate13_live |
| dormant twin | #427 SP at the world origin | same script, mode 0x40 | the origin-placed SP is unreachable to the use pick (the "09 family" origin-SP shape; its script-keyed matrix is the unported part) — dead disc data, never the route | documented (useobj.rs NOTE) |

Never a ride — the fake lift is scenery; no cab exists.

### 07000014 EvilBase — no lift content [D 07000014.md:1,3,12-64; corrected by 0700004a.md:4-6]
The header's `UpdateMFX_HT_Level_Tower2Elevator` music pin is a **mis-pin the 4a doc corrects**
(the music switch maps it to 0700004a; 14's own doc still carries the stale name — one-line doc
fix, not this task's code). The vertical content is the crawl-vent start and a static-art
"Vent_StoreRoom_Ladder" (kind 0x19, not a ladder). **There is no elevator on 07000014.**

### 07000015 missile silo — no lift content [D 07000015.md:84-106,92-93]
The 3×3 silo shafts are kill-volume shafts (0xE4 plates, y 38.57) and 0x22 ladders; no cab, no
lift channel in the 968-placement census.

### 07000016 launch centre — the shuttle-bay lift (scenery) [D 07000016.md:125,130,131,148-150; placements dump]

| piece | id / kind | channel / script | the game does | port |
|---|---|---|---|---|
| bay lift | **#347** SP, mode 1, mask 1, out 80 (0x50), stay 1 (the doc's "#694" id was a different pass) | script 0x060009E9 | **plays once at create** (SP_LoadScript's mode-1 arm [M 0x1dbc08]) — no ch 0x50 writer exists and none is needed; ch 0x50 is the OUT marker. The "writer not found" open question dissolves | ported (prime + mode 1) |
| lift crush | #699/#352 HP | gates 0xcb/0xf3 | the kill volumes around the lift/shuttle | ported (hurt.rs) |
| shaft polys | #532 (0x3B) | p0/p1 = 0xcc/0xcd | Blocking_polys_lift_shaft1 — the pit-squad death channel consumers | ported |

### 0700001b space station — no lift content [D 0700001b.md:51,60-83]
The census negates the whole interactive set; zero lift references. Vertical motion is zero-g.

### 07000041 space-station approach — no lift content [D 07000041.md:17,33-36]
The only "ride" is the tether-wire crossing (five Wire placements); no lift machinery.

### 0700004a THE TOWER-2 ELEVATOR (the scripted descent) [D 0700004a.md:7-9,15,21-28,43-44]

| piece | id / kind | channel / script | the game does | port |
|---|---|---|---|---|
| the lift | #42 SP `SCRIPTPLAYER_Lift`, mode 1, out 0x22 | NIS 0x06000571 | plays at create: entity 0x20005b4 **descends 150 units t90 → t5040**, sound 0x28c rides, event 3 holds the script at t1611, **ch 0x40 at t1610 unlocks the −52 floor doors #110/111**; ch 0x22 is the play marker (the state-2 rule releases the t1611 hold while it is on) | ported; fix_finale_live `the_0700004a_lift_ride_reaches_the_lobby` (Bond aboard the whole descent); the negative-delay fix (script-negative-delay.md) |
| reveal plate | #277 TRIG TouchOnce | out 0x4D | the shaft's first plate (y −9.9) reveals "Gain access to lobby" | ported |
| floor plates | 13 × TRIG TouchOnce | 0x17/0x3d/0x18/0x3f/0x19/0x45/0x1a/0x3e/0x1b/0x4f/0x84 (+0x1c/0x4d at the top) | each fires its floor channel as the lift descends | ported |
| objective complete | ch 0x4A | **no disc writer** (scanned: triggers, 11 scripts' events, drone deaths, SP outs, 42 maps) | the port writes it when the lift script ends (platform parked at the lobby) — the documented stand-in [S], nis.rs keyed to 0700004a + 0x06000571 | stand-in; fix_finale_live pins the ride + completion |
| known gap | the shaft doors' auto-open on the lift's approach | — | a rider is scraped off around the −52 floor mid-ride in the port; the test re-seats Bond along the script's key line | documented gap (0700004a.md:44) |
| parked walkreal | walkreal_0700004a_live | — | parked on the #289 enable-marker contradiction (the event-6 snapshot matrix vs the world-space-keys reading) — reverted once because the same change displaced 07000014's fusebox wire and 07000008's safe entities; needs the main agent's live check | parked (fix-final-blockers2; the live check is restated in §7) |

## 6. The mechanisms and their tests (one live test per distinct mechanism)

| mechanism | levels | proof |
|---|---|---|
| E button → SP latch → channel → SS doors (the use key) | 09, 12, 13 | elevator_buttons_09_live; walk_07000012_live; gate13_live (all pre-existing, re-run green this task) |
| SP mode-64 armed → channel start → the ride script moves the cab; Bond carried | 08 | walkreal_07000008 walk_the_gondola_ride_to_the_end (pre-existing, re-run green) |
| SP mode-8 prime → the cab cycles from load; Bond carried; the out channel releases the self-holds | 0b (09's cab pair: the prime alone) | **lifts_ride_live (this task)** — `the_paternoster_carries_bond_from_load` (+ the 0x75 hold-release arm) and `the_09_cab_pair_loops_from_load` |
| SS create-sweep cab-mouth doors | 0a | walkreal_0700000a (pre-existing, re-run green) |
| SP mode-1 scripted platform descent with Bond aboard | 4a | fix_finale_live (pre-existing, re-run green) |
| SS channel-swap lift doors/screens | 12 | walkreal_07000012_live (pre-existing, re-run green) |

## 7. Live checks for the main agent

1. **0700004a, the −52 landing**: gdbwatch the enable matrix + Bond's y through the −52 landing
   (the #289 NIS enable placement vs the platform's live pose — the parked walkreal's
   contradiction). The backlog item L-463. Expected per the world-space-keys reading: the
   cut-scene rescues Bond onto the still-descending cab; per the event-6 snapshot reading: the
   enable places him at (1.80, −54.128, −3.453) key pose.
2. **0700000b, the paternoster end stops**: with Bond NOT on the roof (ch 0x75 off) the cab
   holds ~499 ticks at y 11 and y 48.44; on the roof deck it should cycle without the holds.
   Watch the cab entity 0x2000367's matrix y + `switch_channels[0x75]` (0x26fc90 + 117).
3. **07000009, the E-route through the closed lift doors**: stand at the closed LIFTDOORS,
   press E — expect the 0700000a load (the #63 key-gate-0 route). The port's use pick runs from
   the render camera (scene.rs passes `camera` as head) where the game builds it from
   Player_GetHeadPos [M 0x1a5170:101-106] — the 1.0 sphere never spans the exit box at the
   camera's hang-back (interact_07000009_live's ignore). A pick-origin fix is shared with every
   tuned reach and needs its own session.
