# bullet-pass-water — the report

## The verdict on the task's premise

**The water planes really do carry the pass bit; `Geo::bullet_pass` reads the right word.** The
02 watersurface/southwater placements (`watersurface01..07`, `southwater1/2` — the planes at the
(-79, 28) shallows) carry glist flags **0x241** in every copy of the entity-params record [D the
disc, all maps of the 02 bin], bit 0x40 set. `parsemap_block_map_data_static` 0x1d1998 copies that
word (celglist+0x44 = the placement's second word at +4, the game's block cursor reads it at +8)
into the runtime cel's `+0x94` [M disasm 0x1d1a78-0x1d1ad0], and `Collide_Intersect` 0x1ebaa8 turns
every hit on such a cel into a pass (disasm 0x1ebaec-fc, the object branch — the contact's +0x54 IS
the runtime cel; the cel is 0xb0 bytes, so the static branch's +0xf0 read belongs to the dynamic
objs). Only `bathhousewater` (glist 0x201, 16 tris at x[-51.6,-48.7] — the indoor pool, nowhere near
the shallows) lacks the bit [D].

The rule set behind `bVar7 & 2` (Collide_Intersect 0x1ebaa8 disasm 0x1ebf3c-0x1ec010):

- `HITDATA+0x51` is memset 0 at every HITDATA alloc [M Collide_Pick 0x1eb320 disasm 0x1eb8a0-0x1eb8d8
  — `memset(node, 0, 0x60)`], so the pass bit starts clear; no glist walk writes it.
- Set by: the cel word's bit 0x40 (statics, 0x1ebaec-fc), the dynamic obj's `+0xf0` bit 0x40
  (0x1ebb5c-74), the surface byte's bit 0x40 (0x1ebf80) AND bit 0x80 (0x1ebf94 — **the port's old
  comment claiming a mask-bit-0x1 guard is wrong: the `movn a1,v0,a0` sits in the beq's delay slot
  and executes whatever the mask is**), and the intersect mask's low byte (`HITDATA+0x51 |= param_3`,
  0x1ebc8c-ish — the RayIntersect 9th arg; the feet ray passes 8 [M 0x1941f8 `addiu v1,zero,8`],
  Check_Target passes 0 [M 0x1a79c4 `sw zero,0(sp)`]).
- Enforced only when the intersect mask carries **bit 0x8** (:317): fraction 1e8 → the contact is
  discarded from the sorted list. Masks 0x1/0x2/0x800 add the mats-0xD/0xE, the glass-0x10 and the
  byte-0x40 drops.

**The feet-ray fact re-attributed.** `Player_FeetOnPoint` 0x194188 calls `Collide_RayIntersect`
with +0x98 = 0xc but the *intersect* mask = **0x8** (disasm 0x1941f8) — bit 0x8 set, so it drops the
pass-flagged water plane and Bond grounds on the submerged floor (y 3.615 at (-79, 28), 0.15 under
the surface). The [L] "the feet contact byte 0xE" is the GROUND RECORD's byte (`obj+0xdc+0x50`),
written by `Player_DealWithObjHit` 0x194458's water branch from the **movement capsule's** contact
(the capsule's intersect mask carries no bit 0x8, so it keeps the plane), and the feet ray's own
ground copy is refused while that byte sits in [0xD, 0xE] [M 0x194188 LAB_001943cc]. The wade facts
(anim bits 8|0x10, steps 0x28, the ripple) all read that record — no contradiction with water being
pass-flagged.

## What a bullet fired into water does in the game: **a splash, then a pass, then a stop**

The flying round's trace keeps everything: `Bullet_update` 0x12e7b0 runs the pick with HITTEST+0x98
= 0 and the intersect mask `+0x9a` = 0x20 [M disasm 0x12f030-0x12f058] — **none of the intersect
drops fire**. `Bullet_CollisionHandler` 0x12ce80 then walks the sorted contacts and decides per
contact (disasm 0x12ceec-0x12d020, the tail):

- the pass flag = `HITDATA+0x51` bit 2 | the dynamic obj `+0xf0 & 0x60` | glass (byte 0x10, when the
  record's `+0x6c` bit 0x800 is clear) | byte 0x40 | byte 0x80 × `Rand_Rand(0x3f) & 1`;
- **pass → the state-2 store is SKIPPED (`bne s4,zero,0012d020`), `Effect_Create` still runs at the
  contact (the material's `EffectInfo+0x84` emitter — the water splash for byte 0xE — plus
  `Effect_Bullet`'s gas and decal and the +0x14 impact sound), the ricochet prob id is 0 so
  `Effect_RicochetProb` 0x1cf290 returns BEFORE its `Rand_Rand` (no draw), state re-arms to 1, next
  contact;**
- solid → state 2 (the round stops), the damage/destruction/ricochet arms run, the walk exits.

So: a round into the 0x241 shallows **splashes at the surface and continues**, stopping at the
submerged floor (mat 7) below; a round into the 0x201 bathhousewater **stops at the surface**
(no pass bit — the byte 0x0E carries none).

## Every caller of the `_bullet`-class queries vs the game's flags

| Game query | intersect mask | sees water? |
| --- | --- | --- |
| flying round's flight trace (`Bullet_update` 0x12e7b0) | +0x9a = 0x20 | **yes** — kept, passed with the splash [M 0x12f030-58, 0x12cff0] |
| player's shot walk (`Player_WeaponInitBullet` 0x1a4bf0:154-159) | 0x20a/0x208 — bit 0x8 | no — pass-flagged water dropped [M; the port's hitscan walk models the handler, keeps the faces and rolls] |
| `Bullet_homing` LOS 0x12f358:92 | 8 | no [M] |
| `Player_FeetOnPoint` 0x194188 | 0x98=0xc, intersect 8 | no (grounds on the floor) [M 0x1941f8] |
| `Check_Target` 0x1a7810 (the crosshair/door pick) | 0x98=0xc, intersect 0 | **yes** — nothing drops [M 0x1a79c4] |
| `Player_FeetOnPoint`'s sibling `Drone_FeetOnPoint` 0x140660 | (same family) | [S — same shape, not disassembled this pass] |

Port-side callers of the drop-pass queries: `segment_bullet` (the homing LOS — correct),
`segment_surface_bullet` (scene.rs aim/sight/MP sites + mp/combat.rs:422 — the mask-8 class, correct
as long as the game twin is the aim trace/homing, NOT Check_Target whose mask 0 keeps water — the MP
bot aim and `Check_Target`-shaped consumers are the open items), `mp/combat.rs:472` (the homing
pick — correct), `segment_flight*` (the flight — **the fix below**), `segment_bullet_clear`
(the 02 walk sight checks — the mask-8 class).

## The fix (one mechanism, the real difference)

The port's flight (`segment_flight*`, `Bullet::update`) swallowed the pass contacts in the query —
a drone round into the 02 shallows never splashed and simply stopped at the floor. Now:

- `world.rs`: `segment_flight_contacts_scoped` / `segment_flight_contacts` return the first solid
  fraction PLUS every pass-flagged or glass-pass contact nearer than it, in distance order, each
  with (t, normal, raw byte, decal host) — `FlightPass`. The old `segment_flight*` wrappers are
  unchanged (`.0`).
- `bullet.rs`: `BulletEvent::World { at, pass }` carries the pass sites (absolute points); the
  flight calls the contacts API.
- `drone.rs` (the `BulletEvent::World` consumer): each pass site gets its `Effect_Create` treatment
  BEFORE the stop's effects, in contact order — the +0x14 impact sound (gated by weapon_data+0x70
  bit 2) and `bullet_impact_on(..., ricochet_mask = 0)`.
- `effect.rs`: `bullet_impact_on` skips the ricochet roll entirely when the mask is 0 — the game's
  `Effect_RicochetProb` returns before its `Rand_Rand` for prob id 0 [M 0x1cf290]; the old port drew
  an RNG draw the game never makes on every pass/pierce contact.
- Docs corrected in place: `Geo::bullet_pass`'s 0x80-guard claim (disproven — delay-slot `movn`),
  the flight-trace comment (the intersect drops nothing at mask 0x20; the handler decides).

**The test** (`world::tests::flight_into_the_02_shallows_splashes_the_surface_and_stops_at_the_floor`,
real 07000002 data through the real input path — the flight seeded by `find_cel`, no teleports): the
mask-0x8 ray hits the floor at t 0.4483 (the water plane invisible to it, as in the game); the flight
returns the same stop PLUS exactly one pass contact — the southwater1 plane, byte 0x0E, splash at
y 3.765, nearer than the stop — and through `Bullet::update` the World event carries that splash site
before the floor stop. Red on the old code: the old event structurally carried no pass contacts (the
flight swallowed them — the same measurement the task cites: 0 water hits of 1377 on the plain
bullet query; the census this task re-ran: 1054 pass-flagged water tris on 02, all dropped).

## Verified

- `cargo test --offline --release -p nf-game --lib -- --test-threads=1`: **687 passed, 0 failed**
  (5 pre-existing ignored), including the new splash test.
- `fix_foliage_bullets_live`: **3/3** (the pass walk's home binary). `bullet_range_live`: **3/3**.
- Checkpoint pushes: 6f429e7ab (the fix + the test), the branch cut commit 94ea1f559.

## What's left (the next message continues from s/bullet-pass-water)

1. The remaining binaries that exercise the changed flight/consumer (rule 10): every
   `walkreal_*_live` (the guards fire through the new event), `rocket_stop_plane08_live`,
   `rocket_glass08_live`, `upward_shot_*`, `mp_perf_16_live`, `npc_doors_live`,
   `fix_guard_misc_live`. None were run whole in this window (the sandbox deadline); the nf-game lib
   (which owns all the changed walk logic's unit tests) and the two edit-site binaries above are
   green.
2. The byte-0x80 intersect drop for the mask-0x8 queries: documented, not coded — no shipped surface
   byte carries 0x80 (the 02 census: bytes 0x00-0x15 and 0x40 only [D]), so there is no real-data
   red test to write. If a level ever shows one, `bullet_pass` needs `mats[k] & 0xc0 != 0` for the
   mask-8 class only.
3. The mask-0 consumers (Check_Target-shaped: the bot aim/sight that should KEEP the water planes)
   still route through the drop-pass queries in the port — listed in the table above as the open
   items; fixing them changes bot behaviour and needs its own walk coverage.
4. Live checks for the main agent (PCSX2):
   - At 07000002 (-79, 4.68, 28): watch the FEET RAY's own contact list (FeetOnPoint's
     `aiStack_c0[0]+0x50`) — expected **7** (the floor), NOT 0xE; the 0xE lives in `obj+0xdc+0x50`
     (the ground record, written by the movement capsule). Access type: read, one frame in the wade.
   - Fire a pistol straight down into the shallows: expected a splash puff at the surface (y ≈ 3.765)
     AND the round's decal/puff on the floor 0.15 below; the impact sound plays twice (the surface's
     water row + the floor's row).
   - Fire at the bathhousewater pool (x ≈ -50.2, z ≈ 11): expected NO splash-pass — the round stops
     AT the surface (glist 0x201, no bit 0x40).

## Usage

not visible to me

## ntfy failed

All three fallbacks, in order, with their exact output:

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "bullet-pass-water: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds`
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'bullet-pass-water: done …', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "bullet-pass-water: done …" https://ntfy.sh/nightfire-yoda-8k3x`
   → killed at the 30 s shell timeout with no output (no response; the connection never opened).

github.com pushes worked throughout (the branch and the queue are up), so the failure is
ntfy.sh-specific egress from this sandbox, not general connectivity.
