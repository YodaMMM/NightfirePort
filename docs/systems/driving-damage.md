# Driving: vehicle damage and health

Port: `crates/nf-game/src/driving/damage.rs` (unit tests in the same file), disc test
`crates/nf-game/tests/drive_damage_disc.rs` (NF_ISO). The `SRuleDamage` / `SRuleHealth` mission
rules in `driving/mission.rs` now use the ported tests.

Tags: [M] DRIVING.ELF decompile (work/decomp/DRIVING.ELF/), [M disasm] read from the raw ELF,
[D] disc data (`/DRIVING/*.VIV`), "ELF data" = initialised words in DRIVING.ELF. Nothing here is
a live capture. "mission" is the object at `DAT_0033a024` (0x1b50 bytes, `FUN_00204de8`).

## 1. Where the player's health lives

- mission+0x378 / +0x37c / +0x380 / +0x384 = health, health max, armour, armour max (floats).
  The HUD reads these four (`FUN_0023da30`) [M].
- The player car's hit-point pointer (car+0x54) is set to mission+0x378 by `FUN_00195a08` from
  the two mission set-up paths (`FUN_001566b0:70`, `FUN_00176658:232`); the same paths call the
  weapon manager's owner set `FUN_0020bf80`, whose `FUN_00205a90` points the car's armour pointer
  (car+0x434, vtable +0x20c = `FUN_001931f0`) at mission+0x380 [M].
- Every other car points +0x54 at its own +0x42c = 700.0 and has no armour pointer (+0x434 = 0)
  [M `FUN_001862c0:211-218`].

## 2. The starting numbers (`FUN_00206ec0`)

The words come from mission+0x1c: the track's `MSet` payload, or the built-in record at 0x4162b8
when the track has none (`FUN_00205220`). `FUN_00204de8` fills that record with 3000 / 3000 /
1000 / 0 at +0x27c / +0x280 / +0x284 / +0x288 [M].

In order [M]:
1. +0x280 == 0: health = max = 10000, armour max 1000, armour 0. Else health = +0x280, max =
   +0x27c, armour = +0x288, armour max = +0x284.
2. `DAT_0033a008` or `DAT_0033a00c` set: health and max ×100 (no writer of either in the
   decompile).
3. PTP+0x61c > 0: health × it. PTP+0x620 > 0: armour = max(armour max × it, armour). These are
   the ratios `FUN_00205c30` writes at the end of a drive.
4. mission+0x5f4 ≥ 1 (a load before this one, `FUN_00205220:99`): health = max(max / 2, health).
5. +0x294 > 0: by difficulty (mission+0x3b8 = PTP+0x984): 1 → +0x298 / +0x2a4, 2 → +0x29c /
   +0x2a8, else +0x294 / +0x2a0, as percents: health and max × the health percent, armour and
   armour max × the armour percent.
6. PTP+0x9a0 bits 0x200 / 0x400 / 0x800: armour = armour max = +0x284 × 2 / 3 / 4 (`FUN_00206890`;
   the last set bit wins). The in-drive cheat handler `FUN_00241bb8` calls the same ×2/×3/×4.

## 3. Function table

| function | what | port |
|---|---|---|
| `FUN_00206ec0` | §2 | `PlayerHealth::start` |
| `FUN_00206890` | armour = max = +0x284 × k | `PlayerHealth::set_armour_multiple` |
| `FUN_00206810` | health += (int)(f × max), capped (`EPowerUpHealth` `FUN_001558d0`, f = percent/100; also calls the car repair vtable +0x19c and plays "SFX_pickuparmor") | `add_health` |
| `FUN_00206850` | armour likewise (`FUN_00155b20`, same sound) | `add_armour` |
| `FUN_00206b20` / `FUN_00206b40` | health (÷ max) / armour (÷ max, 0 when max ≤ 0) | `health_ratio` / `armour_ratio` |
| `FUN_002068b8` | +0x3c8 += amount, +0x3e0 = now, unless lock bits +0x3cc | `book_damage` |
| `FUN_00206960` | kills +0x3c4 += n, ≥ 0 | `add_kills` |
| `FUN_002068e0` / `FUN_00206900` | group-target counters +0x3bc / +0x3c0 / +0x5f0 | `add_group_targets` / `credit_group` |
| `FUN_00207228` | per tick, HUD present with +0x2a0 == 0: bit 1 = health +1 % of max per second below a third; bit 4 = health = max = 10000 | `regen_tick` |
| `FUN_0023e270` | HUD hit shake (+0x328 / +0x32c, cap 20) | `hud_hit` |
| `FUN_002db798` / `FUN_002db930` | zone of a car-local point; neighbour mask (table 0x3c1fe8) | `zone_of` / `zone_neighbours` |
| `FUN_001862c0` (part) | category, 700 hit points, tyre vulnerability, +0x2dc = −1, +0x440 = 1 | `CarDamage::new` |
| `FUN_00187350` (part) | tyre points = `TYRE_DAMAGE_POINTS`; spread level 0x332094 = 0 | `CarDamage::new`, `SPREAD_LEVEL_IN_PLAY` |
| `FUN_00187578` | zones cleared, +0x42c = 700 | `CarDamage::repair` |
| `FUN_00195a30` / `FUN_00195a58` | read / subtract (≥ 0) the hit points | `hit_points` / `take` |
| `FUN_00189598` | `PBondCar` / `PTank` damage virtual (vtable +0x1c) | `inflict` |
| `FUN_001fc198` | world contact → damage | `world_contact` |
| `FUN_001fbbd8` (damage step) | car against car → damage, per side | `car_contact` |
| `FUN_00188200` (flip part) | flipped-car counter and damage | `flip_tick` |
| `FUN_00209ff0` | `SRuleDamage` test | `DamageRule::test` |
| `FUN_0020a1f8` | `SRuleHealth` test | `HealthRule::test` |
| `FUN_00209820` | the rule's watched value (0 player health, 4 a target's hit points) | `RuleSubject` |
| `FUN_00154cb0` | `EPlayerLose`: PTP+0x978 = 4 when health ≤ 0, else 1; "SFX_BondDeath" | `player_lose_exit` |

## 4. The damage virtual (`FUN_00189598`)

Arguments: (amount, dent, car, at, from, kind, source). Callers by kind [M]: 0 = `FUN_00139be8`
(1000, 0.5), `FUN_00150390` / `FUN_00150748` (1e6, 0.5); 1 = the contacts (§5); 2 = bullets
(`FUN_0015de48`, dent 0.25), the projectile's direct hit (`FUN_00182978`, 32, 0), `FUN_0019a710`,
`FUN_0019d080`; 3 = `EDamagePlayer` (`FUN_0014b3e0`), `EInflictDamage` (`FUN_0014fd20`, car
index < 0x30) and the flip damage (100, 0.5).

In order:
1. Category < 2: book the damage on the mission (§3); lock bits set → treated as invulnerable.
2. Kind 2/3, amount > 1, category < 2, mission+0x350 set, not invulnerable: HUD shake.
3. Armour (+0x434): amount < armour → armour −= amount, return 0x90. Else amount −= armour,
   armour = 0.
4. The at→from segment against the car's box (`FUN_002d3950`): no intersection → return 0.
5. Group credit: +0x440 set (cleared for kind 1, and for kind 2 not from the player), category
   2, amount ≥ 5, hit points > 0 → `FUN_00206900(mission, kind == 2)`.
6. +0x2dc < 0 → 32 / hit points.
7. Zone = `zone_of(box, (p.x, p.y / 2 + model offset, p.z))`.
8. Tyres (car+0x37c set and kind 2): the first live tyre on the struck side within (WHEEL_RADIUS
   × 1.25)² takes (uint)amount; at 0 it blows (event 1..4 with 3, and an effect); amount = 0.
9. Hit points −= amount unless invulnerable.
10. Wrecked = hit points ≤ 0 (a `SUB_PHYSICS` car: only an already-dead one hit by kind 1).
    Just wrecked, category 2 with +0x440: kills + 1, +0x440 = 0, and `FUN_00206900(mission, 0)`
    for each car of the same +0x2e0 group with +0x710 == 0 (that group loop is not ported).
    A `SUB_PHYSICS` car that just died: kill credit (no category check), full visual damage
    (vtable +0x1a4 with four 1.0), three random spin draws.
11. Kind 2: push along (from − at) normalised × amount × 1000, y = 2 · max(y, amount × 100),
    plus a random torque seeded `(!r1 << 16) | r2` (the torque box (−0.5, −0.5, −0.5)..(6000,
    7000, 6000) is not ported); skipped on the player car while mission+0x350 is set.
12. scuff = amount · (1 − dent) · scale, dent' = amount · dent · scale.
13. Not wrecked: severity event 6 / 5 / 4 / 0x19 above 0.75 / 0.5 / 0.25 / 0.001 (when any zone
    changed). Mask = the zone, plus its neighbours when kind 3 or the spread level (0.0 in play)
    is below the zone's a + b. Each masked zone: a = min(a + scuff · w, max(a, 1)), b = min(b +
    dent' · w, max(b, 1 or 2 for kind 3)); caps halve while invulnerable; w = 1.0 for the struck
    zone, 0.2 for a neighbour. Stage k (0.15 / 0.35 / 0.65 / 0.85) fires event 0x13 + k (0x16
    not on `SUB_PHYSICS`); model call 10 above 0.35, call 9 above 0.75, over every zone.
14. Wrecked: events 2, 6 (7 for kind 3), stages 0x13..0x16 all 0xffff, model call 10 all; kind 0
    adds to all 16 zones, others to the zone and its neighbours, capped at a 1, b 2.
15. Return 0x10; not wrecked, one random draw r: zone bit & 0x3f18, category 1 and r < 0.5 →
    0x80; else zone bit & 0x3ca5 and r < 0.5 → 0x70.

The zones are the pairs the car microcode's `DamageZones` reads (car-model.md §10.5). Which pair
element lands in which component of the VU1 upload is not traced.

## 5. Contacts

- World (`FUN_001fc198`, impulse, body, point): gate 0.05, or 0.2 for a car body (rigid+0x6d <
  4). Damage = 2 · impulse · 5.0; a car body × 6.0, and the owner books it first (vtable +0x1ac,
  `weapons::CarHits::hit`) only while its +0x374 byte is set. Kind 1 (the player's rigid) with
  vtable +0x64 true → 0. Then the virtual with dent 0.35, kind 1. 5.0 / 6.0 / 0.35 are .data words
  0x33916c / 0x339170 / 0x33915c with no tuner name [M].
- Car against car (`FUN_001fbbd8`): above an impulse of 0.05 each side takes impulse (×2 for a car
  body) × its share × 5.0 (× 3 for a car body), dent 0.35 (0x339168), kind 1. The shares come
  from the contact solver `FUN_001f9d98`, not ported. When this side is a car body and the other
  is kind 1, a +0x1ac booking also runs on the object `FUN_001f7860` returns; which object that
  is (register-mangled call) is not proven.
- `DAMAGE_SCALE_COLLISION` (0x3319b4 = 1.0) is registered in Physics:Physical and read nowhere
  [M-negative: decompile grep and a scan of all loads/stores with low half 0x19b4].

## 6. Flipped cars (`FUN_00188200`)

Each tick, with the body's up·y < 0.3 and rigid+0x6c set, +1000 counts up; else it resets. The
player (category 0) and category 1 stop at 30 and take nothing. Other categories cap at 120 and
take (100, 0.5, kind 3) every tick the count is non-zero and the car has hit points [M disasm
0x1887ec-0x18887c], so a flipped 700-point car is wrecked in 7 ticks.

## 7. Mission rules and the exit

- `SRuleDamage` (type 2): subject (entry+0x10), index (+0x14, subject 4 only), threshold
  (+0x18). Fires when the value fell by more than the threshold since the last test (the last
  value starts at 0). Subject 3 compares the `FUN_0012c440` pair's +8 float instead [M].
- `SRuleHealth` (type 4): direction (+0x18), threshold (+0x1c). Fires on the crossing: below the
  threshold (direction 0) or above it (else); the last value starts one past the threshold so a
  value already past it does not fire [M].
- The rule fires into its channel; what raises `EPlayerLose` is the mission script.
  `EPlayerLose` (`FUN_00154cb0`, skipped once the mission state +0x34c is 3) sets PTP+0x978 = 4
  when the health is ≤ 0, else 1, and plays "SFX_BondDeath" at health ≤ 0 [M].

## 8. Disc data

`drive_damage_disc.rs` reads every archive's `MSet` health words and all 66 pvehicle `.atr`
files. The health numbers are per mission set, not per vehicle: no pvehicle `.atr` carries a
health or armour value [D].

| archive | health (max) | armour / max | health % d0/d1/d2 | armour % d0/d1/d2 |
|---|---|---|---|---|
| MIS01 | 3500 | 0 / 1000 | 100/75/50 | 100/75/50 |
| MIS11 | 3000 | 0 / 1000 | 100/88/75 | 100/88/75 |
| MIS13A | 6000 | 0 / 2000 | 100/90/85 | 100/75/60 |
| MIS13B | 7500 | 0 / 2500 | 100/95/90 | 100/95/90 |
| MIS13C | 5000 | 1000 / 1000 | 100/90/80 | 100/75/50 |
| MIS3 | 8000 | 0 / 2667 | 100/95/90 | 100/95/90 |
| MIS4 | 3000 | 0 / 1000 | 100/88/75 | 100/88/75 |
| RACE | 3000 | 0 / 1000 | 100/88/75 | 100/88/75 |

Per vehicle: only `default.atr` has `TYRE_DAMAGE_POINTS` (30); `TYRES_INVUNERABLE` is true on
`paris_bombvan`, `jungle_bombvan` and `paradis_car`; no file sets `IS_TANK` (the jungle tank's
tyres are therefore not excluded by that flag); `IS_HELICOPTER` only on `helicopter`; `IS_SUB`
on the four subs, `IS_SNOWMOBILE` on the two snowmobiles; `IS_POLICE` on `alpine_police`.

## 9. Unproven / not ported

- The box test `FUN_002d3950`, the world→car transform, the tyre positions and the side test:
  inputs to `inflict`.
- `PHelicopter` (`FUN_00193938`), `Sentry` (`FUN_0019b5a8`) and `Smackable` (`FUN_0019e8c0`)
  have their own damage virtuals; not ported (the helicopter uses 0.15 for model call 10, caps
  both zone values at 1 and three stages).
- The explosion radius damage (`FUN_00200088`), the wreck's explosion/removal in `FUN_00188200`
  (`FUN_0013e678` path), the model-side meaning of events 2..0x19 and model calls 9 / 10.
- The writers of PTP+0x9a0 bits 1, 4, 0x200, 0x400, 0x800 on the ACTION side, of
  `DAT_0033a008` / `DAT_0033a00c`, and what mission+0x350 is.
- Subjects 3 and 4 of the rules read 0 in the port (no target hit points in the snapshot).
