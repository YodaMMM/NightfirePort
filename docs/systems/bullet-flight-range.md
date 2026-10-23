# The bullet flight death range: `weapon_data+0x1C`, not `BU+0xFC` (bullet-range-210)

What kills a flying bullet, and what `BU+0xFC` is instead. This doc is the record of the
bullet-range-210 task, which set out to port `BU+0xFC` = 210.0 as "the round's death range" (the
merged laser-watch-contact task 831's reading) and found the disassembly says otherwise.

## The flight death chain [M Bullet_update 0x12e7b0]

The bullet object's state machine dispatches through a jump table at **0x2f4300** (`sltiu $v0,
$v1, 5` → table at 0x12e830-0x12e84c): state 0 → 0x12e854 (the first-tick setup arm), state 1 →
**0x12e95c**, states 2/4 → 0x12ed58 (delete), state 3 → 0x12ed68 (the landed/mine arm).

- **Case 1 (the flying tick) is the death check**: 0x12e95c loads `BU+0x44` (the weapon record
  pointer `Bullet_init` 0x12c5a0 stored at `pfVar3[0x11]`), reads `BU+0xF4` (the travelled
  distance) and `weapon+0x1C`, and `c.olt.s` branches to the kill (`state = 4`, `obj+0xfe |= 1`)
  at 0x12e978-0x12e984 when **`weapon+0x1C < travelled`**. The C decompile shows the same
  comparison as case 1's opening (`*(float *)(weapon+0x1c) < *(float *)(BU+0xf4)`).
- **The step clamps to the +0x1C boundary**: the shared flight tail (0x12f000, reached from the
  state-0 arm's common path and case 1) adds the full step to `BU+0xF4` (0x12f010), then
  0x12f014-0x12f02c loads `weapon+0x1C` again and, when the new travelled exceeds it, reduces the
  DISPLACEMENT so the round's position lands exactly ON the boundary. `BU+0xF4` keeps the
  overshoot, so the next tick's case-1 check kills the round: the final resting position of a
  range-died round is its `+0x1C` distance, not one step beyond.
- **State 0 (the first tick) has no death check**: the (0.5 + FRand(0.5))·speed step flies free
  of the check (it still clamps in the shared tail), matching the port's `Bullet::update`.

## What `BU+0xFC` actually is: the fuse/arm clock

`Bullet_init` 0x12c5a0 writes `pfVar3[0x3f] = 210.0` (`BU+0xFC`, :103) with per-id overrides
(:255-283: the 0x3b family `((w+0x00 − id)·5 + 5)·60`, the `w+0x00 == 0x6c` → 900.0 special, id
0x69 → 5.0, id 0x37 → 420.0/7777.0 by owner class) and a 300.0 arm on `weapon+0x70 & 0x10000`
(:216-222, BEFORE the id chain). `Bullet_update` consumes `BU+0xFC` in exactly two places:

1. **The `+0x70 & 0x80000` fuse countdown** (0x12edf0 gate → 0x12ee08-0x12ee24): `BU+0xFC -=`
   FRAME_RATE_MUL per tick; at ≤ 0 the round spawns the 0x2000133 explosion object. This is the
   grenade/stick tick fuse the port's `scene.rs` grenade block already models (210/300 ticks).
2. **The id-0x45 countdown** (0x12ef98 `bnel $v1, $a0(0x45)` gate → 0x12efa4-0x12efc4):
   `BU+0xFC -=` the 1.0 constant at 0x30D0D8 [D ELF] per tick; at ≤ 0 the round is deleted
   silently. This is the read task 831 cited as "the death cap per tick" — it is the mine/stick
   arm timer, gated to id 0x45 alone.

`Bullet_CollisionHandler` 0x12ce80 also WRITES `BU+0xFC` (the tripbomb stick's 120.0 arm delay
[:330]) — the same clock domain. No flying round's death reads `BU+0xFC`.

## The census [D weapon_data 0x2bf150, 115 × 0x10C]

`crates/nf-formats/tests/bullet_range_census.rs` pins the disc side. Highlights (the flight
death range is the record's own `+0x1C`):

- P2K 2/3: **50.0** — the round dies at 50 units, NOT 210. SG5 0x16/0x17: 450. Spas 0x1c/0x1d:
  25. Sniper family 0x1e-0x29: 1200. The Laser watch 0x4E/0x4F: 4.0/8.0.
- Exactly one record carries `+0x70 & 0x10000`: rec 55 (id 0x37, `+0x70 = 0x18300`) — and the
  id chain overwrites its 300.0 with 420.0/7777.0, so no disc record's FINAL fuse is the flag's
  300.0.
- The fuse values by family: 0x3b records 59-64 → 300/600/900/1200/1500/1800 ticks; rec 108 (id
  0x3b, `w+0x00 = 0x6c`) → 900; rec 105 (0x69) → 5; recs 55-57 (0x37) → 420 (7777.0 when the
  owner is a class-3 drone); every other record → 210.
- The satchel family's `+0x1C` is 5000.0 against its 300-tick fuse — the two fields are plainly
  different quantities (a 300-unit flight cap would die mid-throw).

## The port

- `crates/nf-game/src/bullet.rs` — `Bullet.range` is `weapon_data+0x1C` and `Bullet::update`
  already implements the game's check-then-clamp-then-die exactly (the pre-existing model). The
  unit test `p2k_round_stops_at_plus_1c_while_a_1200_record_passes_150` pins the boundary clamp
  and the death.
- `crates/nf-game/src/drone.rs` — every spawn site feeds `+0x1C`: `fire_player`, `fire_remote_cosmetic`,
  the guards' `fire_weapon` (:9898), the melee `close_combat_impact` (:2761), the gun-imp mount
  (:13202), the GT turret (:14007), and `worldobj.rs`'s Shooter — all already game-exact. The
  one deviation: the Laser watch 0x4E/0x4F keeps 210.0 at `fire_player`/`fire_remote_cosmetic`
  **[S unverified]** — task 831's port choice, kept because the merged laser tests ride it; the
  game rule is +0x1C = 4.0/8.0 with the per-tick refire tiling the beam (see the correction in
  docs/systems/laser-watch-contact.md).
- The MP paths (`mp/botweap.rs`, `mp/combat.rs`) already read `+0x1C` — unchanged.

## Tests

- `crates/nf-formats/tests/bullet_range_census.rs` — the disc census (the fuse chain + the
  named flight-range pins).
- `crates/nf-game/src/bullet.rs` — `p2k_round_stops_at_plus_1c_while_a_1200_record_passes_150`
  (the clamp/boundary/death semantics).
- `crates/nf-view/tests/bullet_range_live.rs` — the real path on 07000005: a P2K round fired
  through the real input path reaches the flagged tree at ~45 (impact on the shell span); from
  ~80 in the same lane it dies in mid-air with NO impact (on the 831 note's 210.0 model it
  would have reached); a 1200-record round flies ~119 to the same tree — far past the P2K's
  boundary.

## Live checks for the main agent

1. **A P2K round's flight on a fresh boot (gdbwatch)**: watch `BU+0xF4` (the travelled float at
   `BU = obj+0xE0`) and `BU+0xFC` through a P2K shot at open space. Predicted: `BU+0xF4` rises
   past 50 and the object dies with its position ON the 50.0 boundary; `BU+0xFC` stays 210.0
   untouched. If instead `BU+0xFC` descends or the round flies past 50, the 831 model survives
   and the port's census-backed `+0x1C` needs a second look.
2. **A held Laser watch (0x4E)**: watch `BU+0xF4` of each spawned round. Predicted (the
   decompile model): each round's segment clamps at ~4.0 and a new round spawns every tick (the
   front advances ~4/tick). If the game's rounds really fly 210, the port's [S] stand-in is the
   game's truth and the correction above needs a re-read.
3. **A fired grenade (frag 0x2c-0x31)**: `BU+0xFC` should descend 1.0/tick from 210 (the
   0x80000 fuse) — confirms the fuse-clock reading of `Bullet_init`'s values.
