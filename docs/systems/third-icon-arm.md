# The third-icon arm (`BL+0x95F`) — the game's exact rule

The context icons (placement kind 0xFA, class 0x42) arm the interaction context the HUD shows and
`Player_HandleJump`'s wire grab reads. This doc pins the whole chain with the evidence classes
([M] decompile, [D] disc, [L] live EE snapshot) that the port's implementation
(`crates/nf-view/src/scene.rs`, the per-tick arm loop) follows. Numbers only; the strings/labels
are the tests' business.

## The chain, per tick

1. **Clear.** `Player_CollisionHandler` 0x1949a8:22 writes `BL+0x95F = 0xFF` unconditionally at
   the top — the arm never persists across ticks. [M]
2. **Pick.** `control_movement_object_handler` 0x1336f8:238 calls `Collide_Update` per object
   list; for every object carrying a HITTEST at obj+0xD4 whose query mask (+0x94) is non-zero,
   `Collide_Pick` 0x1eb320 runs [M 0x1ea398:72-84]. The player's HITTEST lives at BL+0x740
   [M Player_Init 0x1910f8:40].
3. **Mirror.** Every object the player's pick hit gets a node prepended to ITS OWN obj+0xD0 chain
   with `+0x58 = the pick owner (the player)` [M 0x1ea398:160-233].
4. **Arm.** `ThirdIcon_Update` 0x196e80:28-53 walks the icon's obj+0xD0 chain; on a node whose
   +0x58 object is class 3 it writes the icon's type (`*piVar2`, param 0) into the player's
   `BL+0x95F`, and (type 6 only, player substate != 7, instance timer 0) posts the "Press Action"
   label on the 4-tick timer. [M]
5. **Read.** `Player_HandleJump` 0x1916b0:118-148: on the jump press (action 7, mask 4) with
   `BL+0x95F == 3` it grabs the nearest class-0x2C wire within 1000 u (`Vec_Dist3D` scan, no other
   gate) via `Player_CollWire`. [M]

## The pick test (the arm radius)

`Collide_Pick` 0x1eb320 scopes candidates to the pick's straddle cels and dispatches on the query
mask; `Collide_PickObj` 0x1eae80 tests each object. The player's mask is **0x800** in every
`Player_Collision` 0x194df0 arm branch [:205,229,257,282,315,363,381] and 0 when disabled [:308].
The 0x800 branch is a segment (capsule) test [M Collide_PickObj 0x1eae80:152-166; the cel-scoped
twin Collide_Pick 0x1eb320:224-241]:

```
dist(segment[HITTEST+0x20, HITTEST+0x30], obj+0x80)  <  obj+0x8C·1.5 + HITTEST+0x8C
(hit rejected when reach² <= sqdist — the strict `<`)
```

The ×1.5 rides the OBJECT (icon) radius at 0x1eae80:160 / 0x1eb320:228; the picker adds its own
+0x8C raw. No branch multiplies the picker. The other masks are not the arm: 0x200/0x201 = segment
with `obj+0x8C` alone (no 1.5, no picker), 0x100/0x400/0x1000 = sphere
`HITTEST+0x6C + obj+0x8C`. [M]

## The picker's inputs (the per-substate hittest)

`Player_Collision` 0x194df0 rebuilds the hittest per tick, switched on substate − 1
(:201, disasm 0x195070). All arms write mask 0x800; the shapes:

| substates | +0x8C (picker) | seg A (+0x20) | seg B (+0x30) |
|---|---|---|---|
| 1 (climb), 7 (creep) | 0.275 | eye + 0.55·up | eye + (0.275 − cc)·up |
| 4 (crouch) | 0.55 | eye + (0.1 − (cc − 0.55))·up | eye + (0.45 − cc)·up |
| 0 (walk) and all others (default) | 0.55 | eye + 0.275·up | eye + (0.55 − cc)·up |
| 13, 14 (swim) | min(cc, 0.55) | bone (0x80000005) | eye + (0.55 − hand-mid)·up |

`eye` = obj+0x30 — the position copy, not an independent joint [M Control_CreateObjEx
0x1343c8:24-27 copies obj+0xC0 there]. `cc` = `*(obj+0xDC)+0xCC` — the root height the walk code
carries; **[L] ee_start_05 player 0 (obj 0x16cefe0, M_CONTROL 0x16cb8a0)**: cc = 1.0347183
(standing), 0.55 crouched — the port's `ROOT_HEIGHT`/`CROUCH_ROOT_HEIGHT`.

**[L] the walk hittest at spawn (substate 0)**: mask 0x800, +0x8C = 0.5500000, seg A = eye+0.275,
seg B = eye−0.4847 = eye + (0.55 − 1.0347183). The port's `pick_arm` + `pre_collide` anchor
reproduce this exactly.

## The icon's world sphere (the pick's object side)

`Control_BuildWorldSph` 0x133070: centre obj+0x80 = the object matrix × glist+0x10 (+ obj+0xC0),
radius **obj+0x8C = glist+0x1C × |obj+0xE8|** [:48]. For placement-created icons obj+0xE8 is
1.0 and never overwritten: `control_init_object` 0x134630:17 sets 1.0 and `ThirdIcon_Create`
0x197070 writes only the params/class/glist/flags. So **the icon's pick radius is the glist's
+0x1C raw — the placement scale does not apply** (the scale words at placement+0x34..+0x40 go to
other creators, never to icons).

[D probe `icon_radius_probe.rs`] every 0xFA placement of 07000005/07000008 carries scale
(1.0, 1.0, 1.0) and the resolved record radius equals the glist bounds radius (the entity-params
+0x14 == celglist+0x1C layout, docs/fields/glist.md).
**[L] ee_start_05 icon obj 0x16dbf10** (class 0x42, +0xF0 = 0x1050 = glist flags 0x50 | 0x1000
per ThirdIcon_Create:33): obj+0xE8 = 1.0, obj+0x8C = 2.2757196, glist 0x170b7b0+0x1C =
2.2757196 — identical.

## The port's numbers on 07000008 (the cable icon)

The level's only 0xFA placement [#105, main map 51]: "Hand_Over_Hand", ctx 3, ch 0, at
(133.4257, 48.3264, −101.2065), record radius 1.1638, scale (1,1,1) [D probe]. The walk pick's
reach = 1.1638·1.5 + 0.55 = **2.2957** from the segment anchored at the position.

The west ledge stands [probe `ledge_standable_probe.rs`, the port's own collision world]:
stood_y 46.49 at x 130.4 rising to 48.72 at x 133.4 (the ledge climbs toward the anchor). The
old probe spot (132.0, −104.0) sits seg_dist 3.14 from the icon — **outside the game's reach**;
it had only armed through the [S] `icon.radius + 2.5` sphere stand-in. Standable spots inside
2.2957: (132.0,−102.5) 1.990, (132.0,−102.0) 1.656, (132.5,−103.0) 2.019, (132.5,−102.5) 1.651,
(133.0,−103.0) 1.843, (133.0,−102.0) 1.243 — the cable crossing arms on the game's rule there.
`fix_08_walls_live`'s grab spot is (132.5, −102.5).

## Residuals

- The straddle-cel gate stays the port's documented stand-in (`icon_cels`, the icon's own
  sphere-straddle set vs the game's pick-side straddle) — fix-08-walls' evidence stands.
- The swim pick (substates 13/14) is bone-anchored in the game; the port approximates with the
  position anchor (`pick_arm`'s walk arm) — the swim tests pass; a live
  bone-position check is still open.
- [S] unverified: nothing in the arm path; every constant above carries [M]/[L]/[D].
