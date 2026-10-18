# The abseil death (killing a guard on the rope) — what the game does

Task abseil-death-11 (playtest 2026-10-18, 07000011: "guards rapelling and not rapelling and not
falling when they die"). The abseil family's states are in docs/reports/drone-abseil-elevator.md
(the 720 task); this page is the hit/kill half: what a bullet or blast does to a guard on the
rope, and which 07000011 guards rope at all.

## Which guards abseil on 07000011

- Exactly five placements in the disc carry placement param 1 = 0x06000892 [D `nf-extract
  drones`]: **07000011 ×2** (skin 0x05000008, at (18.895, 46.516, 1.434) on switch channel 0x13
  and (19.745, 46.580, 0.363) on 0x14, the second gated to difficulty ≥ 2 by its param 2 [M
  `Drone_Create` 0x136be0:49]) and 0700004a ×3. **No placement uses 0x0600089C** (mode 0x1b).
- The placement msg sets the mode byte `Drone+0xC5` = 0x1c (0x0600089C → 0x1b) and starts the
  drone at AbseilInit 0x8e [M `DefaultInit` 0x14b300:615-627, 757-766; L DroneTypeSettings rows
  27/28 = `8e00 …`].
- Everything else on 11 — the stairwell walkers (skins 0x05000004/06/08's other groups, on
  channels 0x11/0x13/0x14/0x36/0x37/0x1D [D docs/levels/07000011.md]) — is a normal dtypenew-1/2
  guard (modes 0/2): `DefaultInit` arms the rope only for mode 0x1b/0x1c [M 0x14b300:757-766],
  so they walk in. Pinned by `abseil_death_live::only_the_disc_abseilers_rope_the_stairwell_walkers_walk`.

## The kill routing — one mode table, three callers

A killing blow selects the death state by the mode byte `Drone+0xC5`:

| mode | bullet [M 0x1461c8:40-52] | blast [M 0x146480 @0x1465e4-0x146660] |
| ---- | ------------------------- | ------------------------------------- |
| 0x1e | BotDeathAnim 0xf2 (not ported — no Bot row in the port's roster) | (same table, SP arm) |
| 0x1b, 0x1c | **AbseilDeath 0x92** | **AbseilDeath 0x92** |
| 0x0d | Death_Anim 0x44 (the table's else) | Death_Anim 0x44 |
| 0x1d | AstronautDeath 0xbf | AstronautDeath 0xbf |
| else | Death_Anim 0x44 | DeathByExplosion 0x45 |

The abseil states themselves take hits through `DroneFunc_HandleImpact(d, msg, 0, 0)` — their
DSTATE case 8/9 pass resume 0, allow 0 [M 0x169610:48-50 / 0x169730:68-70 / 0x169940:50-52 /
0x1738e0:31-33]: the damage applies, a surviving hit changes no state (the guard stays roped).
`Death_Anim 0x44`'s ENTER re-routes an abseil kill that arrived by another death path: mode
0x1b/0x1c → 0x92, 0x1d → 0xbf, before the special-death check [M 0x1637f8:30-38].

Port bug fixed: the port's bullet death selector only knew 0x1d → 0xbf, its blast selector read
the **voice set `+0xD8`** (the port's `kind`) where the game reads the **mode `+0xC5`** —
drone-port-check.md #3 again — and `Death_Anim` had no re-route. A shot abseiler died into
Death_Anim, which leaves `+0x23` set: he stayed on the rope, the playtest's f8063.

## AbseilDeath 0x92 (0x169b28) — the release

- ENTER (case 1) [M]: the corpse flag `obj+0xF0 & 0x20` (SP; on 0700004a the collision bit 0x20
  clears and bit 4 sets instead), `+0xCD8` = 1, `Drone_AlertStatusSet(3)` (the 0x2f5cc8 table
  row 3 = 1.0), `NDrone2_LogDroneDeath` (the player's team-1 kill logs "dispatched" unless the
  KO flag is set [M 0x14ab00:11-31]), **`+0x23` = 0** (the rope releases — gravity resumes),
  `+0xCDC` = 0, danim 0x40, DropWeapon.
- UPDATE (case 3) [M]: on 0700004a wait to be drawn → the soft delete; elsewhere wait for
  `+0xCDC == 0` **and the feet probe** (`anim+0x60 & 8` — the collision handler's own floor
  flag) → danim 10, `+0xCDC` = 1, DEAD 0x47.
- EXIT (case 2) [M]: `+0xCD8` set → SetDeathChannel.

Port bug fixed: the UPDATE polled the port's `death_pending` (the `+0xCD8` death-channel latch)
where the game polls its own `+0xCDC` landing latch — the arm could never fire. The port now has
`abseil_land_latch` for `+0xCDC` (its own field; merge lesson 4).

## The corpse keeps gravity and collision

`DoCollision` (0x149ec8) and `DoGravity` (0x141e58) gate on the **drone flags `+0x4F8`**
(& 0x20 collision / & 0x80 gravity / & 0x200 no-gravity, with the voice 0xc/0x13, +0x538,
near-player, drawn and `+0x570 & 2` refinements) — the corpse flag `obj+0xF0 & 0x20` is read by
NEITHER. A dead drone falls and lands like a live one; that is what carries the released
abseiler to the floor for AbseilDeath's feet probe [M 0x169b28 case 3]. The death drift
`DroneMove_AstronautDeathMove` 0x156008 is the astronaut's own move, not a corpse rule.

Port bug fixed: the port's `Drone_CollisionHandler` had an extra `corpse` short-circuit
freezing every corpse's physics — added in the handler's original port (0078), not in the game.
The `+0x4F8` refinements stay unmodelled in the port ([S], pre-existing: gravity always runs
when `bounded`).

## What a hit looks like end to end (the game's timing)

Bullet on a hanging guard → damage through HandleImpact(0,0); a killing shot → 0x92 → the rope
flag clears that tick → the corpse falls from the rope point under WldGravity 9.8 u/s² [M
0x140968's gravity arm] → the feet probe lands it (a ~10-unit drop ≈ 1.4 s ≈ 86 ticks) → danim
10 + DEAD → the 10 s death timer fades it. Pinned end to end on the real 07000011 stairwell by
`crates/nf-game/tests/abseil_death_live.rs` (5 tests).

## Rope rendering

The port renders no rope line (nothing in ACTION.ELF's symtab draws one from `+0x670`; the
grapple rope `Player_GrappleSetRope` 0x192728 is Bond's own). The F12s' rope is the game's drone
renderer — a live check for the main agent below.
