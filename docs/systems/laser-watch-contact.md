# The laser watch's beam contact (laser-watch-contact)

The laser watch (the Laser 0x4E/0x4F — the disc's name label 0x05000043 "Laser" [L USATxt.dat;
`weapon_data+0x6C = 0x900` [L ee_start_05.bin @ weapon_data 0x2bf150 + id·0x10C]) burns locks
through a CONTACT LIST, not through the aim line. This doc is the record of the port's fix that
replaced the aim-line stand-in (`gadget::laser_tick`'s `BEAM_RADIUS = 0.25` closest-approach test,
removed) with the game's chain.

## The game's chain, link by link

1. **The fire spawns a real bullet every refire tick.** `Player_WeaponFiring` 0x1a5460's fire tail:
   the refire accumulator `BLData+0x92C` resets to 1.0 (the record's `+0x40` refire is 0.0 [L], so
   the `1.0 >= fVar19` arm picks 1.0 [M 0x1a5460:653-658]) and the record's `+0x44` short is 0 [L],
   which routes the shot to `Player_WeaponInitBullet` 0x1a4bf0 [M 0x1a5460:688-690] — one round per
   tick while held (`Player_RoundToFire` 0x1a18e0 spends `+0x91` = 1 [L]). `Bullet_init` 0x12c5a0
   creates a **class-5 object** (`+0xff = 5`) whose link (`obj+0xE0` → `obj+0x110`,
   `control_create_object` 0x1344e0's `addiu $s0, $a0, 0x110` + the `obj+0xE0` store at 0x1345a0)
   carries:
   - `link+0x44` → the weapon record — the id the lock walk reads (`sw r21, 0x44(r17)` at 0x12c678,
     the store the decompiler dropped);
   - the HITTEST at `link+0x50` (`*(float **)(obj+0xD4) = link+0x50` [M 0x12c5a0:102]);
   - `BU+0xFC` (the HITTEST range) = **210.0** [M 0x12c5a0 — `pfVar3[0x3f] = 210.0`; the per-id
     overrides are 0x3b/0x69/0x37 only]. The watch's record `+0x1C` is 4.0/8.0 [L] — that is the
     USE/fire distance gate, NOT the bullet's range (the decryptor's gate reads `+0x1C` at
     0x1a5460:574-577). `Bullet_update`'s disasm reads `BU+0xFC` per tick at 0x12efa4 for the
     death cap. The port's old `+0x1C` range kept the watch's rounds alive for 4.0 units — the
     beam's bullet stream never reached the locks at all (the visible symptom: the laser seemed inaccurate).
   - the per-tick segment: A = `obj+0x30`, B = `obj+0x30 + dir·max(step,1)`, range-clamped;
     the HITTEST type **0x201** (`+0x94`, `Bullet_update`'s tail disasm 0x12efd8-0x12f188 — the
     0x101 point arm only for raw steps under 0.125; the watch's step is 33.33 [L `+0x20`]).

2. **The pick.** `Collide_PickObj` 0x1eae80:92-108's 0x201 arm: the closest point on the segment
   to the candidate's WORLD SPHERE (`obj+0x80/0x84/0x88` centre, `obj+0x8C` radius —
   `Control_BuildWorldSph` 0x133070's posed model sphere) via `vecutil_calculate_closest_point_on_line`
   0x1df7e0; **`dist² < r²`** picks. The pick entry (`0x60` bytes) lands on the picker's `+0xD0`
   list with the PICKED object at entry+0x58 and the squared distance at entry+0xC (the sort key).

3. **The distribution.** `Collide_Update` 0x1ea398's pass-2 walk sorts the picker's `+0xD0` entries
   by the distance and, per entry, appends a contact onto the PICKED OBJECT's `+0xD0` list:
   entry+0x58 = the picker (the bullet), entry+0x08 = the HITTEST's `+0x90` damage
   (`weapon_data+0x0C` = 0.0 for the watch [L]), entry+0x51 |= 1. One entry per (bullet, lock) per
   tick; the bullet stream's segments tile the ray (one bullet per tick, 33.33 units each), so a
   held beam is **one contact per tick** on the aimed-at lock. The world intersect that kills the
   bullet runs in the same pass after the picks — the contact of the wall-killed round still lands
   (this is why the beam burns through the safe door's front plane: the bolt centres sit behind it
   [D the 08 probe grid]).

4. **The burn.** `Lock_Update` 0x1ae3d8 case 0, kind 1 (record u16[+0] = 1 — the placement's
   param 0 [D #207/#208/#139-142]): the lock walks its `obj+0xD0` contact list — head at
   `lock+0xD0`, `entry+0x4` = next, `entry+0x58` = the picker object — and takes entries whose
   picker's class byte (`+0xff`) is 5 and whose `picker link+0x44 → record[0]` id is **0x4E (−3 per
   contact) or 0x4F (−10)** [M :57-66]. The contact subtracts the amount from BOTH burn bytes
   `lock+0x104`/`lock+0x105` (u8, clamped at 0) [M :82-95] and stores the grace short
   `lock+0xF6` = 2 [M :79]. When `+0x105` drops **below 4** the lock completes: sound 0x249 at the
   lock [M :96-99], then `Lock_Unlock` 0x1b0650 writes `switch_channels[record u16[+8]]` [M
   :193-196] — the out channel (08's bolts 134/135 [D], muxed into 136 by #278 [D]; 0c's hinges
   14/15/16/17 [D], muxed into 18 by #505 [D]).
   The bytes start **0xff**: `control_init_object` 0x134630 sets `+0x100..0x106` = 0xff for every
   created object and `Lock_Create` 0x1ae1f8 writes neither byte. So the 0x4E burn takes
   (0xff − 3k < 4) → **84 contact ticks ≈ 1.4 s** of continuous beam; the 0x4F takes 26.

5. **The heal.** With no contact and `+0xF6` == 0, BOTH bytes refill **+1 per tick to the 0xff
   cap** [M :107-121]; while `+0xF6` != 0 it decrements instead — the refill resumes 2 ticks after
   the last contact. A partial burn heals; the burn requires continuous contact.

## The port's mapping

- `crates/nf-game/src/lock.rs` — `Lock.burn_a/burn_b` (the `+0x104`/`+0x105` bytes, 0xff at
  create) and `Lock.grace` (the `+0xF6` grace); `burn(amount)` is the contact tick, `idle_tick()`
  the heal arm, `burn_through` the test-premise shortcut (premise tests only, documented).
- `crates/nf-game/src/drone.rs` — `fire_player` gives the watch's bullets the real `BU+0xFC`
  210.0 range; `update_player_bullets`' watch arm is the contact pass (one `burn(3/10)` per live
  bullet per tick against the lock's world sphere as loaded — the same
  `Collide_PickObj`-shaped test the searchlight's ported feed uses); `Drone::tick_inner` runs the
  idle arm after the bullet pass.
- `crates/nf-game/src/gadget.rs` — `laser_tick` is the beam MESH's endpoints only (the ray, the
  world-capped hit [M `Player_LaserPointer` 0x1a65d8]); the burn stand-in is gone.
- The charge meter (`Sim::laser_charge`) keeps the [D]-measured ~31/s drain
  (docs/live/2026-10-14-weapon-measurements-pcsx2.md). The game's writer of the pane's fields is
  unlocated (see the report's live checks); the idle refill stays [S] and out of pass/fail.

## Tests

- `nf-game` gadget/lock unit tests: the 84/26-tick burns, the grace, the heal cap, the endpoints.
- `gadget_laser_live` (08): the real-path bolt burn with the fail-first mid-hold pin (60 ticks <
  84 must NOT complete; the byte descended) and the heal sister; the burn follows the button;
  the away-from-the-bolts sister.
- `fix_level_0c_live` (0c): the four hinges burn through the real path (~85 ticks each, the clip
  500 [L `+0x92`] covers the 4 × ~85 spend) and open the display vent.
- `fix_safe08_live` (08): the fusebox → both bolts → ch 136 → the pickups chain.

## Correction (bullet-range-210, 2026-10-23): `BU+0xFC` is not the death cap

The disassembly of `Bullet_update` 0x12e7b0 refutes the "the HITTEST range `BU+0xFC` is the
death cap" claim above (see docs/systems/bullet-flight-range.md for the full chain):

- Case 1 (the flying state — state 1 → 0x12e95c via the state jump table at 0x2f4300) kills the
  round when **`weapon_data+0x1C` < `BU+0xF4`** (`lwc1 $f1, 0x1c($v1)` / `c.olt.s` at
  0x12e95c-0x12e984), with the step clamped to the +0x1C boundary (0x12f014-0x12f02c) and
  `BU+0xF4` keeping the overshoot for the next tick's kill. The "range-clamped" segment above is
  clamped by `+0x1C`, not `BU+0xFC`.
- The 0x12efa4 read of `BU+0xFC` this doc cited as "the death cap per tick" is the **id-0x45-only
  arm** (`bnel $v1, $a0(0x45), 0x12efd8` at 0x12ef98 gates it): a 1.0/tick countdown (the
  constant at 0x30D0D8 [D ELF]) that kills the round at ≤ 0 — the mine/stick arm timer, not a
  flight range. The other `BU+0xFC` consumer is the `+0x70 & 0x80000` fuse countdown at
  0x12ee08 (explode at ≤ 0). `Bullet_init`'s 210.0/300.0/0x3b/0x69/0x37 values [M 0x12c5a0:103,
  :255-283] are that clock's initial values, in TICKS.
- The watch's record `+0x1C` = 4.0/8.0 is therefore its real per-segment flight range, and the
  game's beam still burns locks because the chain above fires **one round per tick** [M
  0x1a5460:653-658, :688-690]: the 4.0-unit segments tile the ray at 240 units/s, the front
  reaches the lock in ~distance/4 ticks, and the 84 contact ticks follow. The port keeps the
  210.0 range for the watch's bullets for now [S unverified — the port's choice; the
  laser tests ride it] and the live check (gdbwatch a held watch's `BU+0xF4`: predicted
  ~4.0/segment with `BU+0xFC` untouched at 210) settles which model the game runs.
