# Driving tracks: scene objects, smackables, visibility actions, effect and collision parts

Code: `crates/nf-formats/src/trackobj.rs` (`track_objects`, `object_actions`,
`placement_part_kinds`, `article_phys_names`, `SmackableAttrib`), `crates/nf-game/src/driving/props.rs`
(`TrackProps`: drawn / in the sim / knocked loose). Tests: `crates/nf-formats/tests/trackobj_disc.rs`,
`crates/nf-game/tests/drive_props_disc.rs`, unit tests in both modules.
Evidence tags: [M] DRIVING.ELF decompile (function + EE address), [D] disc data.

## 1. Static world vs scene objects [M]

"Init World" `FUN_0022faf8` walks the `Map in#0` items (geoprim.md §2) twice:

- **Flag 0x10** (`+0x0c`) items look up their `ps#0` record (`+0x0e`, 0x20 bytes). When the
  record's `+7` is not 0xff and the item has a reference, the item's flag bit 0 is set and a
  0x40-byte `RSceneObj` is built (`FUN_001c7098`, label "RSceneObj"), its model loaded from the
  item's article (`FUN_001c7658`, `+7` passed on) and its matrix set (`FUN_001c9258`); the record's
  `+8` becomes that object. Item flag 0x20 adds a `WTargetable` (`FUN_001c8998`), flag 0x08 puts
  the object in the world grid and sets its `+0x18` bit 26 (`FUN_001c7940`). The record's `+4`
  u32 `>> 8` is a behaviour word: bit 4 registers the object with the sim at once
  (`FUN_002015d8`, skipped when the model's `+0x28` is 0x696).
- The other items stay in the static world. Their part's effect list (`as` payload `+4` → `ef`,
  0x40-byte entries until a 0 type byte at `+0x14`) gets flag 1 when `+0x10` has bit 0x10 or
  the type is 2; type 6 with a name at `+0x18` starts that particle system
  (`FUN_002330b0` → `FUN_00232af8` on `DAT_0033aad0`, the "GFXParticleSystems" manager built by
  `FUN_00233618`).
- `FUN_001b1988` builds path objects from flag-0x10 items whose record `+0` is 2.

The scene object is drawn while its `+0x18` bit 24 is set: the ctor sets it, `FUN_001c8b78`
clears it, `FUN_001c8b90` sets it, the draw `FUN_001c8c70` tests it. Its grid radius is item word
15: `(w & 0x3ff) × {0.25, 16.0}[bit 30]` (table 0x386b80), 10.0 when that is 0 [M 0x1c7098]. The
restart `FUN_00230730` rebuilds every scene object from the saved copies.

On the disc every flag-0x10 item of the 8 tracks has `+7 != 0xff` (all are scene objects) [D].

## 2. Trigger actions on placements [M + D]

The `Trgr` action ids (creator table 0x36fda0, trackstart.md §1) that act on items:

| id | creator → handler | effect |
|---|---|---|
| 0x17 | `FUN_0014f2c8` → `FUN_0014f218` | hide the scene object, item flag bit 0 set |
| 0x18 | `FUN_00159be8` → `FUN_00159b38` | show it, bit 0 cleared |
| 0x95 | `FUN_0014b6e0` → `FUN_0014b630` | hide + leave the sim (`FUN_00201690`) |
| 0x96 | `FUN_0014bc28` → `FUN_0014bb58` | show + rejoin the sim (`FUN_002015d8`) |
| 0x23 | `FUN_0015e4b0` → `FUN_0015de48` | `EWakeupSmackable` |

The argument block starts with the id; `+4` (0x23: `+8` item, `+0xc` record) holds an `sr`
reference whose Map string is `CARP::<<Map>>::{in  0000}::<byte offset>` (`{ps  0000}` for the
record) [D]. A null reference is a no-op (each handler tests it) — MIS11 trigger 32 has one.
For a static item, 0x17/0x18 only flip its flag bit 0 (and `FUN_001cb0d0` may redirect it); whether
the static draw honours that bit is **not traced**.

The `EStartMission` trigger fires during Init World (`FUN_0022cd30`), so its actions apply before
the first frame. Measured (`trackobj_disc.rs`, `drive_props_disc.rs`):

| track | flag-0x10 items | smackable (bit 2) | sim at init (bit 4) | wake actions | hide | disable | enable | hidden at the first frame |
|---|---|---|---|---|---|---|---|---|
| MIS01 | 557 | 544 | 4 | 544 | 0 | 4 | 0 | none |
| MIS11 | 168 | 0 | 9 | 0 | 9 | 1 | 0 | none |
| MIS13A | 83 | 16 | 9 | 16 | 0 | 4 | 1 | 571-573 (`007logo`), 643 (`shield`) |
| MIS13B | 73 | 32 | 1 | 32 | 0 | 1 | 0 | none |
| MIS13C | 41 | 16 | 0 | 16 | 0 | 0 | 0 | none |
| MIS3 | 348 | 46 | 7 | 46 | 0 | 5 | 2 | none |
| MIS4 | 270 | 213 | 16 | 213 | 0 | 1 | 0 | 3196 (`missilebox`) |
| RACE | 253 | 212 | 0 | 212 | 0 | 0 | 0 | none |

No track uses id 0x18; behaviour bit 0x40 and item flag 0x20 occur on none. Every wake action
names a scene object and its own record. Which conditions fire the other triggers (the proximity
test of `FUN_0022c890`'s callers) is not traced; `TrackProps::apply` runs an action on request.

### RACE's start barriers

`POS2_Cautionguard` items 3166-3169 (z 173.32-173.45, 4.5-4.7 behind the car start at z 168.78)
are flag-0x10 scene objects with record `{kind 1, damage 0, behaviour 0x203, +7 = 0}` and grid
radius 2.75. Triggers 9-12 sit on them (same x/z) and each holds one wake action for one barrier.
RACE has no hide/show/disable/enable action at all, and the start trigger (212) holds none: **the
game does not remove them at the start**. They are drawn, are not in the sim until woken, and
are not part of `CDat`. Whether the chase camera's ray (`FUN_002127f0` → `FUN_002166f8`) sees
scene objects is not traced: `FUN_002166f8` tests the static world (`FUN_00216d30`) and, when the
camera's `+0x5c` bit 3 is set, a second list of `{object, ?}` pairs at the caller's `+0x30` via
`FUN_00232638` / `FUN_00214c08`; what that list holds is unknown.

## 3. Smackables [M]

`EWakeupSmackable` (`FUN_0015de48`): the scene object comes from the action or the record's
`+8`. Behaviour bit 2 → `FUN_001ffa90`: if the sim has room (`FUN_002012d0` with cap 0x24 = 36:
when the sim's count `+8` is above 36 the active smackable farthest from the new one is retired
by `FUN_0019e610`, skipping those whose `+0x6e` is 2; the spawn is allowed when the count is then
below 36) an `ASmackable` (0x88 bytes, `FUN_0019e100`) starts with the velocity the scene object
reports; with a triggering vehicle an impulse follows (`FUN_00211098` / vtable `+0x1c`, not
decomposed). Bit 0x40 takes `FUN_002002e0` instead. A
refused or empty wake sets the trigger's `+0x12` bit 0.

`ASmackable` attributes: namespace "smackable", key from the scene object, else "default"
(`FUN_001696f0`). Hit points = `HITPOINTS − ps[+4] / 256 × HITPOINTS` (`FUN_00230c48` finds the
record) [M 0x19e100]. On retire, `FUN_0019e610` either hides the scene object or marks the trigger
and stores the remaining damage into `ps[+4]` (`FUN_0022cad8` decides; not decomposed).

The `PhysicsData` schema (`FUN_00194e18`, defaults `FUN_00194cd0`):

| key | offset | parser | default |
|---|---|---|---|
| MASS | +0x04 | float | 100.0 |
| HITPOINTS | +0x08 | float | 500.0 |
| FORCE_DETACH | +0x0c | bool | 0 |
| MOMENT | +0x0c | vector | — |
| DEFAULTSOUND / WARNINGSOUND | +0x10 / +0x14 | sound | −1 |
| IMPACTSOUNDLO / MED / HI | +0x18 / +0x1c / +0x20 | sound | −1 |
| SCRAPESOUND | +0x24 | sound | −1 |
| DESCRIPTION | +0x28 | float | 0 |

FORCE_DETACH and MOMENT are registered at the same offset; which value survives a file that sets
both (all of them do) is **not traced**. The lookup helper `FUN_001950a0` retries with the key
"Bench" (0x37a6a0) when a name is not found; that the smackable path goes through it is not traced.

On the disc [D]: articles name their attribute with an `sr` string `PHYS::<Name>`; every name has
an `.atr` in the same VIV whose `[Section]` matches case-insensitively, except `StreetTable` /
`streettable` on MIS4 and RACE (and `PHYS::undefined`, the exporter's empty value). Keys appear as
`MASS` and `Mass`. MIS13A/B/C ship no `default.atr`. The power-ups (`PowerUp007`,
`PowerUpMissiles`, `PowerUpShield`, `PowerUpRockets`) are smackables too.

## 4. Parts geoprim does not draw [D]

Every placement geoprim leaves without a model (geoprim.md §3) is one of two kinds, all 8 tracks:

| track | collision-only `{as  ffff}` | effect parts (ef kinds) |
|---|---|---|
| MIS01 | 30 | 56 (type 2: 43, type 6: 13) |
| MIS11 | 1 | 106 (type 2) |
| MIS13A | 55 | 21 (2: 4, 6: 17) |
| MIS13B | 0 | 24 (6) |
| MIS13C | 2 | 19 (6) |
| MIS3 | 0 | 65 (2: 26, 6: 39) |
| MIS4 | 45 | 0 |
| RACE | 45 | 0 |

- **Collision-only**: the article has no `as` parts, only `ca` collision pieces (`WALLFIX*`,
  `CUSTBADWALL*`, `WFInv*`, `Invisible_Barrier_*`, `River*`, `Frozenjump*`…). Nothing to draw.
- **Effect parts**: the `as` payload's `+4` names an `ef` list instead of a model
  (`trafficlight` part 1, `EiffelLight`, `Construction_Lights`, `University`…). Model parts
  carry `ef` lists too. Type 2 = a glare sprite, type 6 = a Giotto effect (`GIOT::<name>` in the
  effects gallery): decoded in `nf_formats::trackfx`, docs/systems/driving-effects.md section 7.

## 5. Not done

- The `V4-32` layout of `Rock_Collapse`, `JA_SAL`, `JB_SAL`, `LookOutTowerExplosion` (geoprim.md §4):
  not decoded; its strip unit is still unknown.
- The trigger fire conditions, the smackable physics body.
- Item data flag bit 0 (set on 17 RACE items as 0x51): its meaning before Init World sets it.
