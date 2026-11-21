# Driving: weapons, projectiles and hit damage

Port: `crates/nf-game/src/driving/weapons.rs` (unit tests in the same file).

Tags: [M] DRIVING.ELF decompile (work/decomp/DRIVING.ELF/), [D] disc data (the `/DRIVING/*.VIV`
archives), "ELF data" = initialised words read straight from DRIVING.ELF. Nothing here is a live
capture. `now` is the sim tick (0x339f54), `fps` the integer rate at 0x339f4c that
`FUN_001feda0` stores once (60 on NTSC, see drive-physics.md).

## 1. The weapon table

`FUN_0020f2b8` fills 32 records at 0x4165c0, stride 0x54 [M]. `FUN_0020bbe0` copies the whole
block (0x4165c0..0x417040) into each vehicle's weapon manager, sets `+0x3c = (int)(secs[w] · fps)`
from the float table at 0x33a030 and sets slot 0's owned bit [M].

| off | field | notes |
|---|---|---|
| +0x00 | id | 0..31 |
| +0x04 | name | "Gun", "Rocket", "Missile", "Q-Booster", "Time Mine", "Prox Mine", "Rem Mine", "Floating Mine", "Detonator", "Spikes", "Q-Slick", "Q-Smoke", "Q-Worm", "Laser", "Q-Pulse", "Radar", "Cannon", "Q-Vision", "2 Wheels", "Grenade", "Q-Charge", "Torpedo", "Remote Torpedo", "P2K", "Snowmobile gun", "Snowmobile Rocket", "Sniper Rifle", "Ultralight gun", "Ultralight rocket", "FOD Gun", "FOD Rocket", "CH-6 Remote Rocket" |
| +0x08..+0x10 | three ids | kept verbatim; reader not traced |
| +0x14 | flags | 1 owned, 2 barrel action running (blocks fire), 4 reloading, 0x20/0x40 the "Init" sound |
| +0x18 | ammo | |
| +0x1c / +0x20 | clip / clip size | equal at build: 1 on most rows; Missile 4, torpedoes 2, P2K 10, Snowmobile gun 50, Sniper 30, Ultralight gun/rocket 10, FOD Gun 50, CH-6 10 |
| +0x24..+0x30 | last-fire tick per barrel | -1000 at build |
| +0x34 | damage | Gun 10, Rocket 250, Missile 250, Time/Prox Mine 100, Rem Mine 500, Floating Mine 1000, Spikes/Q-Worm/Laser 25, Q-Pulse 50, Cannon 250, Grenade 500, Q-Charge 5000, torpedoes 250, P2K 10, Snowmobile gun 50, Sniper 10, Ultralight gun 30, FOD Gun 50, rockets 250 |
| +0x38 | shot counter on | off for Q-Booster, Detonator, Spikes, Q-Slick, Q-Smoke, Q-Worm, Laser, Radar, Q-Vision, 2 Wheels, Q-Charge |
| +0x3c | interval, ticks | see below |
| +0x40 / +0x44 | sound bank (4) / sound stem | "SFX_%sFire", "SFX_<stem>Reload", "SFX_<stem>Init" |
| +0x48 / +0x4c | fire / reload barrel cursor | |
| +0x50 | shots fired | |

Intervals, 0x33a030 [ELF data], seconds → ticks at 60: Gun 1/6 → 10, Rocket 0.25 → 15, Missile
2/3 → 40, Q-Booster 4 → 240, the mines/Detonator/Q-Pulse/Radar/Cannon/Grenade/Q-Charge 1 → 60,
Spikes/Q-Slick/Q-Smoke/Q-Worm/Laser/Q-Vision/2 Wheels 2 → 120, Torpedo 3 → 180, Remote Torpedo
0.25 → 15, P2K/Snowmobile gun/Snowmobile Rocket/Sniper/FOD Gun/CH-6 1/12 → 5, Ultralight gun 0.05
→ 3, Ultralight rocket 1 → 60, FOD Rocket 5 → 300.

Barrels: 0x33a0b0 holds eight words per weapon, 0-terminated: the vehicle action ids the shots
and reloads play (`FUN_001a27a0`) [ELF data]. Rocket 5,6; Missile 7,8,0x34,0x35; Torpedo
0x3d,0x3e; Remote Torpedo 0x3f,0x40; one action for most others; none for Detonator, Q-Worm,
Q-Vision, 2 Wheels, Q-Charge.

Cycle order: 0x33a4b0 = 0,1,2,3,14,4,5,…,31 (the Q-Pulse ranks after the Q-Booster) [ELF data].

## 2. Firing (`FUN_0020de20`, vehicle path)

In order [M]: a lock whose object carries ONLY_TARGET_WITH_EMP turns every weapon but the gun and
the Q-Pulse into "SFX_DryFire" (the bomb van, `paris_bombvan.atr` [D]); ammo < 1 stops; no fire
while `now <= switch_until`; no fire while `now < fired_at[cursor] + interval`; no fire while the
barrel action runs (flag 2); ids outside {0,1,2,4,5,6,7,10,12,16,18,20} also wait for the action
state to read 2. Then the current barrel fires, the clip loses a round and the cursor wraps at the
list's terminator. The Missile fires two barrels per press (clip −2 when its clip size is 4); the
manager's salvo word (+0x4c) makes it four, and gives the Torpedo both tubes. Reset sets that
word from the global bit 0x1009a0 & 8, which nothing traced here writes.

Ammo cost: the gun is free; Ultralight rocket 2; Missile = MISSILES_FIRED_SIMULTANEOUSLY (absent
→ 2), doubled in a salvo with enough ammo; Rocket = ROCKETS_FIRED_SIMULTANEOUSLY (absent → 2);
Torpedo 1, or 2 in a salvo; every other id 1. `DAT_0033a008` skips the cost and the reload flag;
`0x1009a0 & 2` skips the cost only. Every shot sets the reloading flag.

`FUN_0020bf80` sets the Missile clip to 2 × MISSILES_FIRED_SIMULTANEOUSLY when the car has the
attr (`jungle_hench.atr` / `jungle_truck.atr` = 1 [D]).

## 3. Reloading (`FUN_0020c078`)

Each tick, for the primary, the selection and the gadget with the reloading flag: while the
reload cursor's barrel fired more than `interval − 5` ticks ago and `ammo − clip > 0`, play its
reload action, add a round (the Missile adds one only on odd barrels unless its clip size is 4),
step the cursor; the flag clears when the reload cursor reaches the fire cursor [M].

## 4. Selection, pickups, loadout

- `FUN_0020bdf8`: the cycle list holds this mode's ids (`w <= 0x16` in the car, `w > 0x16` in POV
  mode) that are the selection or owned with ammo, never the gadget, sorted by rank. Next/prev
  (`FUN_0020d608`/`FUN_0020d598`) wrap; a change sets `switch_until = now + fps/3`
  (`FUN_0020d4c0`) [M].
- `FUN_0020d378(w, amount, clip, damage)`: owned, ammo += amount; a damage > 0 replaces +0x34; the
  clip arguments act only in POV mode [M].
- `FUN_0020bff0`: a new gadget zeroes the old gadget's ammo; gadget 2 is turned into 200 missiles
  on the selection [M].
- `FUN_00205498`: with the built-in default record (car "vanquish") the start is 99 each of Gun,
  Rocket, Missile, Q-Pulse, Q-Slick, Q-Smoke, Q-Booster, Radar with Rocket selected, or on
  `uw_map` Torpedo, Remote Torpedo, Q-Booster, Floating Mine with Torpedo selected. A mission
  record lists per-weapon enable/ammo/clip/damage; the first enabled weapon after the gun with
  ammo is selected [M].

## 5. What a shot carries (`FUN_0014ca60`)

Damage = the *player's* table +0x34 × the shooter scale [M]. So a pickup that overrides a weapon's
damage changes the enemies' shots with that id too. Exceptions: Gun/P2K from a kind-10 shooter
are 5 × scale; Gun/P2K from the player's POV or remote shot read the Snowmobile gun row (50); Snowmobile
gun / Ultralight gun / FOD Gun read the Ultralight gun row (30) unscaled; the Sniper reads its own
row, unscaled. An AI car's scale is `clamp(100 + driver byte +0x131, 0, 200) / 100`.

AI aim: skill = `clamp(driver +0x130 + FUN_00208428(mission), 0, 100)`; the shot goes to the exact
aim point when `rand·100 >> 16 < skill`, else to a point wobbled by `2·amp·sin(now·0.02 + seed)`
with amp 0.01 / 0.005 / 0.015 by the difficulty word (mission +0x3b8 = 1 / 2 / other) [M].
AI_GOODAIM (junglesnow.atr [D]) makes the gun aim at the player's position.

Delivery [M]: Gun/P2K a shell `FUN_001fffa8(dmg, 0)`; Cannon `FUN_001fffa8(0.75·dmg, 0.25·dmg)`;
Snowmobile gun, Ultralight gun, FOD Gun a RayShell (first float 220); Sniper a RayShell (400);
rockets, missiles, Q-Pulse, torpedoes, CH-6 a `Missile` object; Floating Mine, Grenade (first
float 55, `dir.y·1.6 + 0.2`) their own spawners.

Bullet streaks: BULLET_STREAK_TYPE indexes Henchmen, Vanquish, SuperSnowmobile, SUV, Captain,
Ultralight, Sentinel (0x3349b0, `FUN_001d7620`) [M]; vanquish/vanquishalps 1, supersnow 2, the
jungle cars 3, ultralight 5, sentry 6 [D]. Each shooter steps a counter that wraps at
BulletStreakCount{type}; only the shot that finds it at 0 draws a streak [M]. The FX tun carries
the colour/texture/width/length/speed/count per type and the Lighting tun the projectile glow
Colour/Size{Missile,Rocket,SnowmobileRocket,EMPInflight,EMPTarget} [D]; drawing them is not part
of this module.

## 6. The projectile (`FUN_00180bc8`, `FUN_00181e70`, `FUN_00182978`)

Per kind [M] (speed / thrust / turn / launch / homing delay / life):

| kind | cruise | thrust | turn | launch | delay | life | other |
|---|---|---|---|---|---|---|---|
| Rocket | 250 | 125 | 1 | shooter + 5 | 0.2 s | 5 s | no lock; first contact check 10 ticks later |
| Missile | 250 | 50 | 1 | shooter + 7 | 0.2 s | 4 s | takes the lock, starts unaligned |
| Q-Pulse | 175 | 17.5 | 1 | shooter + 4 | 0.33 s | 5 s | takes the lock; lingers fps ticks |
| Torpedo (player) | 50 | 25 | 0.1 | shooter + 1 | 0 | 5 s | +0x7c = −1 |
| Torpedo (AI, lock < 100) | 60 | 30 | 1 | | | 8 s | damage 2000 |
| Torpedo (AI, else) | 40 | 12 | 0.01 | | | 8 s | damage 750 |
| Torpedo (alpha_sub) | 20 | 10 | 0.03 | | | 5 s | |
| Remote Torpedo | 40 | 4 | 1 | 8 | 0.33 s | 10 s | |
| Snowmobile Rocket | 250 | 125 | 1 | shooter + 5 | 0 | 5 s | |
| Ultralight rocket | 800 | 80 | 1 | 10 | 0.5 s | 5 s | takes the lock |
| FOD Rocket | 300 | 0 | 1 | 300 | 0.1 s | 5 s | |
| CH-6 | 34.5 | 7.2 | 0.1 | shooter + 3 | 0.1 s | 6 s | numbers from 0x331930.. [ELF data] |
| other | 200 | 0 | 1 | 200 | 0.1 s | 5 s | |

The step [M]: the cruise target is ×0.3 while a lock is held and the body is not yet lined up;
before the delay the body turns 0.1 toward its flight direction; after it, the aim point is the
target plus 3 ticks of its velocity, and beyond √4 = 2 units the body turns toward it by
`turn` (halved until lined up) × `min(1, (age − delay)·dt·0.5)` with the velocity carried along;
the cruise target then scales by `max(aim·dir, 0.1)`; dot > 0.99 means lined up. A ground target
(target +0x34 set) within 0.5 units detonates it; otherwise within √2 the lock drops; a torpedo
facing more than 60° off drops its lock and lives fps/8 more ticks. Thrust is ±accel along the
forward axis (+ below the target speed, − above). Life counts down one per tick; 0 detonates.

Hit [M `FUN_00182978`]: a contact with an object calls its vtable +0x1ac with the damage, then
`FUN_00200088` spawns the explosion with the damage; a world hit calls
`FUN_0019ddb8(dmg, dmg·0.008, …)` (not traced) and spawns the explosion; a Q-Pulse calls the car's EMP method (vtable +0x254) instead.

## 7. The car's side

- `FUN_00187608` (car vtable +0x1ac): `taken += dmg`, `+0x374 = fps`. A kind-3 car or an
  IS_POLICE car (`alpine_police.atr` [D]) with more than 499 taken, or more than its +0x54 value
  (default 1.0), raises mission request 2 (`FUN_00208540`, the +0x254 word) [M].
- `FUN_0018ac28` (vtable +0x254): the EMP — once per hit window, until `now + EMP_LIFETIME` (120
  ticks, 0x3319b8) — plays "SFX_EMPdetonate"; on a car named `paris_bombvan` it also raises mission request 0
  and calls `FUN_00206900(mission, 0)`; `FUN_00188200` clears it after the tick passes [M].
- HITPOINTS sits in the smackable schema (`FUN_00194e18`); shipped values 1..5000 [D].

## Unproven / not ported

- The POV/on-foot weapon object (`DAT_00327670`): ids 0x17..0x1f in POV mode, its clip logic and
  the sniper zoom gate.
- The body integration (`FUN_001fd448`'s physics object, args 0.3/0.6 and 4.0): the port steps
  `pos += vel·dt`. The launch velocity is modelled as `dir · launch`; `FUN_0025a7a0` is not read,
  and the player's rockets also add the shooter's velocity afterwards in `FUN_0014ca60`.
- The game turns a quaternion (`FUN_002cc018`); the port turns the forward axis only.
- The Missile's sideways wobble (+0x720..+0x728, AI-fired only) and the kind-10/kind-11 target
  branches are not ported.
- Explosion radius damage (`FUN_00159fb8` → `FUN_001847e8`), what mission request 2 does, the
  car's +0x54 value, the three text ids, what +0x7c of a torpedo does, the meaning of the second
  shell value and of the RayShell's first float.
- Which global writes `DAT_0033a008`, the 0x1009a0 bits and the salvo word.
