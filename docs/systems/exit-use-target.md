# The level exit's use target (`triggers::exit_use_target`)

How the port resolves "the thing at the exit the player uses", for the campaign/flow legs
(`campaign_flow_live`) and any caller that wants to stand where the player stands. Added by
`exit-aim-01` after the 0x07000001 row carried "E at the exit; no exit mesh (unbounded bbox) —
E-aim skipped".

## The game's facts

- **The E pick is a sphere vs collision geometry.** `Player_Activate` 0x1a5170:100-108 builds
  head + look (`Player_GetHeadPos` + `Mat_GetDir` of the viewer matrix) and calls
  `Collide_SphereIntersect(1.0, …)`; the hit list's objects dispatch on their class byte —
  doors 0x1c, **triggers 0x1d**, SS 0x25, car 0x28, monitor 0x38, lock 0x3a, gunimp 0x47,
  PCQWorm 0x49, and `obj+0xE4 != 0 && class == 0` → `SP_Activate` [M 0x1a5170:112-175].
  `Trigger_Create` stamps class 0x1d on every trigger object [M 0x1b1720:93].
- **A trigger with no collision mesh is not E-reachable** — the pick hits collision cels only,
  and the disc can genuinely carry none: 07000001's #18 "Load Point" (0xE8 LOAD_LEVEL, param 0 =
  0x07000002) resolves to a map object with a gfx box but `collision None` [D placements dump;
  the port's `triggers::build` then has no tris and its bbox is unbounded (lo = inf / hi = −inf)].
- **E on a Touch trigger needs the `mask & 4` bit.** `Trigger_Activate` 0x1b2318's type-2 arm
  returns before any write unless `ext+0x36 & 4` [M 0x1b2318:23-25]; the mask word comes from the
  placement's +0x50 param, defaulting to 2 [M 0x1b1720:80-84]. 07000001's #180 "LoadPointActivator"
  (0xEB Touch, out 0x10) carries mask 0x18 [D placement p1] — **E on the plate is refused by the
  game too**.
- **The exit fires by presence.** The plate's `Trigger_Hit` (Bond AND the contact — mask 0x18,
  filter skin 0x4F [D]) raises ch 0x10 = the 0xE8's gate channel, and `Trigger_Update` case 10
  auto-fires `Trigger_Activate` when the gate is on — no touch check [M 0x1b1a78:162-166]; the
  type-10 arm loads the level through `ResetMap_LevelToLoad` in BOTH gate arms (success write when
  the gate is on, fallback write when off) [M 0x1b2318:96-111]. `walkreal_07000001_live` walks the
  full escort through that chain (ch 0x10 → level_exit 0x07000002).

## The resolution rule

```rust
pub fn exit_use_target(triggers: &[Trigger]) -> Option<(&Trigger, bool)>
```

1. The LOAD_LEVEL trigger itself (`next_level != 0`) when it owns collision → `(exit, true)`:
   E can fire it (the class-0x1d hit runs the load arm) — 07000002/03's exits are this shape.
2. Else the Touch plate whose `out` equals the 0xE8's `gate` channel and whose bounds are real
   → `(plate, false)`: the level's real use target is the plate the player stands on; E is
   refused there ([M mask & 4], the plate's mask lacks the bit) and the exit fires by presence
   once both bodies stand on it. 07000001 is this shape: #18 (no collision [D]) → plate #180
   (44 tris [D], out 0x10 == gate [D]).
3. `None` — no LOAD_LEVEL exit, or a mesh-less one with no reachable gate plate.

`// [S] unverified` — the fallback choice itself is the port's survey mechanic (the game has no
"resolve the exit target" step; a player just walks to the plate). Every fact it rests on is the
[M]/[D] above. Callers must not treat `(plate, false)` as an E target: the E-refusal is pinned by
`e_on_the_loadpointactivator_shape_is_refused` (nf-game) and on the real path by
`exit_aim_01_live` (240-tick E hold on the plate → ch 0x10 stays off, no exit).
