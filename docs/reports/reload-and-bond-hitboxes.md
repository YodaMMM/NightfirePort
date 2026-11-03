# reload-and-bond-hitboxes

2026-10-24, branch `s/reload-and-bond-hitboxes`. The two unverified items: the guards' reload
animation, and Bond's own 16 hit boxes (the port had no code for them by name — the guard rounds
hit a documented cylinder deviation instead). Both are now ported/pinned with tests.

## 1. The guards' reload path [M]

The state table (docs/fields/dronetype-tables.md) maps four SP reload states, all in the port's
`drone.rs`:

| state | function | anim | clip refill |
|---|---|---|---|
| 0x69 AimStandReload | `NDrone2_DSTATE_AimStandReload` 0x1666d8 | 0x52, or 0x4a on a 1-in-30 draw gated by `DroneAnim_CanDoAnimState(0x4a, 0)` [M :21-27, disasm 0x166738-0x16677c] | at ENTER, `+0xBBC = +0xBBE` [M :30, after the anim call] |
| 0x71 AimCrouchReload | `NDrone2_DSTATE_AimCrouchReload` 0x167370 | 0x52 always — no draw, no alert in the case [M case 1] | at ENTER [M :17, before the anim call] |
| 0x2c SniperReload | `NDrone2_DSTATE_SniperReload` 0x15ff80 | 0x52 → SniperFire 0x2b on timer C (20 ticks) [M case 0xc] | at EXIT (msg 2) [M case 2] |
| 0x98 UnderCoverSniperReload | `NDrone2_DSTATE_UnderCoverSniperReload` 0x16a8b0 | 0x52 → 0x97 [M case 1] | at ENTER [M case 1, after the anim call] |

The path in: `DroneFunc_CombatState` 0x148338 case 0x10 returns state 0x69 when the live clip
`+0xBBC` drops under 1 [M :196-197]; the fields are `+0xBBC` = the clip (decremented per shot in
`DroneWeap_FireWeapon` 0x178f68:265-270 [M]) and `+0xBBE` = min(reserve, clip size), maintained by
`BOTWEAP_ReloadAmmoType` 0x129d28 (the reserve-pool transfer, with the weapon_data+0x68 bit-0x100
one-shell-at-a-time gate [M :34-36]) and `BOTWEAP_changeWeapon` 0x12a028:60-66 [M]. The reload-anim
predicate `BOTWEAP_reloadAnimForWeapon` 0x12b200 returns a has-reload-anim boolean over the weapon
id ranges [M], matching the port's `mp/botweap.rs` reading. The MP bot's reload (state 0xdd
`BotAttackReload` 0x170190, `ReloadAmmoType` at msg 1, anim 0x52 with the 0xcf/0xd7 follow-ons) was
already ported with tests in `mp/botweap.rs`; this task pinned the SP states.

### Differences found and fixed (drone.rs)

- **AimStandReload ENTER now draws `Rand_Rand(30)` unconditionally** [M disasm 0x166738-0x166740:
  `addiu a0,0x1e; jal Rand_Rand` — the draw precedes every branch], and a 0 draw with
  `CanDoAnimState(0x4a, 0)` plays anim 0x4a (the reach-down reload) with the same 0x68 follow-on
  [M 0x1666d8:21-27]. The port previously played RELOAD (0x52) only and consumed NO draw — every
  AimStandReload entry desynced the shared stream from the game's. Level fact (test-pinned): no
  07000005 guard's anim set can do 0x4a, so the branch is dead at this level — but the draw is not.
- **AimStandReload ENTER now calls `Drone_AlertStatusSet(1)`** [M 0x1666d8 case 1; the game's
  AlertStatusSet 0x13a8d8 writes +0x51C/+0x51D/+0x51E/+0x520], as does **UnderCoverSniperReload
  ENTER** [M 0x16a8b0 case 1]. The port had neither.
- **AimCrouchReload keeps neither** [M 0x167370 case 1 has no Rand_Rand and no AlertStatusSet] —
  the old shared arm was right by accident; now pinned both ways.
- The refill amount `self.clip = self.clip_max` mirrors `+0xBBC = +0xBBE` exactly — the port maps
  0xBBC/0xBBE to `clip`/`clip_max` (the mapping is documented at drone.rs's weapon-change arms);
  the per-guard reserve pool behind 0xBBE is not modelled ([S] fold: the port's guards carry full
  reserves, where 0xBBE = clip size; the MP bot pool transfer is the exact port in mp/botweap.rs).
- The +0x4F8 seen flags in the ENTER arms stay unmodelled (the port's established convention; the
  flag is documented as such at several other ENTER sites).

## 2. Bond's 16 hit boxes

**The game's data.** `HumanCollisionBones` at 0x2c6be8 (340 bytes = 17 × 20-byte entries + the
terminator) [D 0x2c6be8, ACTION.ELF file offset 0x1c0ae8]. Entry = {s16 bone a (the part id),
s16 bone b (the partner, −1 = use the parent), f32 c0 (not read here), f32 half x/y/z} — the game
reads the halves at entry+8/+12/+16 (`psVar3+4/+6/+8` shorts, `AnimGetCollData` 0x1c4ac8:70-76 [M]).

| # | part | partner | half (x, y, z) |
|---|---|---|---|
| 0 | 5 head | parent(5)=4 | 0.11, 0.11, 0.11 |
| 1 | 4 torso | 1 | 0.2, 0.25, 0.15 |
| 2 | 53 hip L | parent=0 | 0.12, 0.1, 0.12 |
| 3-5 | 54/55/56 thigh/calf/foot L | chain | 0.1,0.25,0.1 / 0.1,0.22,0.1 / 0.07,0.07,0.15 |
| 6-9 | 49/50/51/52 hip R chain | chain | same as L |
| 10-11 | 20/21 upper arm/forearm L | parents | 0.2,0.07,0.07 both |
| 12 | 23 hand L | 21 | 0.1, 0.05, 0.05 |
| 13-15 | 32/35/39 arm R | chain / 35 | same as L |

**Construction [M].** `AnimGetCollData` 0x1c4ac8: requires obj+0xDC (the AnimObject) with +0xC8
flags bit 0 and bit 1 set; builds the palette (`psiBuildMatrixPalette` 0x111498 — the local pose
composed with the obj matrix, world space [M]); picks `AstronautCollisionBones` 0x2c6d40 only for
model hash 0x5000078, else the human table [M :31-34]; each box: matrix = the partner bone's
palette rotation, centre = the midpoint of the two bones' translations (entry 0: the head bone plus
(0, bone.y − other.y, 0)), halves × the skin scale (rec+0x10/14/18), part id = bone a.

**The test that runs on Bond [M].** `Collide_Intersect` 0x1ebaa8:101-105 routes the bullet's
kind-0x201 HITTEST to `Collide_Jointy` 0x1ec180 when the target obj has glist (obj+0xD8) and
obj+0xFA bit 3; Jointy calls `AnimGetCollData` on the hit object and walks the boxes in LIST ORDER,
returning on the first box hit (not the nearest); HITDATA+0x52 = the winning box's part, +0x40 =
the hit point, +0xC = radius·t (the sort key the port's nearest-of replaces — the radius is common
to one target's candidates, so the ordering is unchanged).

**[L] evidence (work/ee_start_05.bin and the 07000001/0700000a starts; file offset = EE address).**
Bond's player obj (class byte +0xFF = 3) found at 0x16cefe0 in 05: glist live (0x17381d0), +0xFA =
0x8 (bit 3 set — the Jointy gate passes), AnimObject 0x16cb8a0 with +0xC8 = 0x407 (bits 0/1 set —
the AnimGetCollData gate passes), model record 0x1714050 hash 0x5000007 (the human table), scale
(1.0, 1.0, 1.0), palette at 0x16c9b60 POPULATED and world-anchored (bone 0 = Bond's pos). The
object matrix undone, the three levels' local poses agree to ±0.011 (the idle sway) — the standing
idle pose is stable and is embedded as `damage::BOND_IDLE_POSE`.

**Ported (damage.rs `bond_boxes`, wired at drone.rs's bullet tick).** The 16 boxes over the [L]
idle pose rotated by Bond's yaw; `BulletEvent::Player` carries the part id. The old cylinder
(`PlayerVolume`, feet −0.4 / h 1.8 / r 0.25 — the documented deviation at bullet.rs:127) is gone
from the guard-round path; the type stays for the paths that genuinely use coarse volumes
(Check_Target's crosshair test, the MP combat stand-in, the viewer occlusion stand-ins).

**Damage scale per part: none in SP.** `Player_HandlePain` 0x1902c8's SP arm scales by difficulty
only [M :28-49]; the per-part multipliers (Plr_DMod_Head / UpperLimb / LowerLimb on parts 5,
20/21/32/35, 49-56) sit under the MPSettings gate at :51-78 [M]. The head-vs-body sound: SP always
plays 0x412 [M 0x194458:229-235 — `bVar4` (part == 5) is only set in the MP branch, :53/:181]; the
port's existing behaviour was already correct. SP damage has no per-part scale — pinned by a test.

## 3. Tests (crates/nf-game/tests/reload_and_bond_hitboxes_live.rs, all 6 green)

- `bond_boxes_match_the_snapshot_palette` — 16 boxes, the disc's part ids in table order, the box
  centres vs the snapshot's measured world bone translations (head 104.881/25.537/108.630, torso,
  foot, hand) at the 05 start pose [L], the head halves = the disc entry [D].
- `rounds_hit_each_body_region` — a round into each region's outer face reports that region's part
  through the full list-order walk; every box but 39 has a clean lane. Part 39's box lies INSIDE
  the torso box at the idle pose — the game's own coarse layout (verified in a sweep: no clean
  direction exists); it becomes hittable when the arm animates. The port holds the idle pose, so
  the exception is pinned with the reason.
- `sp_pain_has_no_part_scale` — the same weapon damage through any part costs the same [M 0x1902c8:28-49].
- `aim_stand_reload_draws_rand30_and_refills_at_enter` — across 400 seeds: exactly ONE draw at
  ENTER, the clip refilled at ENTER, firing dropped, the anim call (0x52, → AIM_STAND_FIRE), the
  alert status 1.
- `aim_crouch_reload_draws_nothing` — zero draws, refill at ENTER, (0x52 → AIM_CROUCH_FIRE), no
  alert status write.
- `guard_rounds_carry_a_body_part` — in the sim, a guard's rounds into Bond land
  `BulletEvent::Player` parts from the disc's set.

What ran before the push (all green): `nf-game --lib` 546, `ai_live` 36, the new file 6,
`bullet_range_live` 3, `fix_foliage_bullets_live` 3, `walkreal_07000008_live` 4,
`copter_05_flight_live` 1, `copter_paths_live` 4, `walkreal_07000005_live` 6.

## 4. What's left

- The anim-driven palette: the game's boxes follow Bond's walk/crouch animation (the palette is
  rebuilt from the live anim every `AnimGetCollData` call); the port holds the standing idle pose
  at any stance. A crouching Bond is hittable a touch high (the boxes stand where the idle stands;
  the old cylinder had the same class of error, coarser). Needs Bond's skeleton ticked in the sim
  or a live-pinned per-stance pose table.
- `AstronautCollisionBones` is parsed in nf-formats and selectable via `for_skin`, but no test
  drives it (the astronaut skins' levels only).
- The MP per-part multipliers (Plr_DMod_*, MPSettings._456_4_) and the MP assassin one-hit rules
  in `Player_DealWithObjHit` 0x194458:118-183 are not ported (the port's MP combat is the bots'
  stand-in path).

## Live checks for the main agent

1. Fresh boot of any level with guards (e.g. 07000005), Bond standing: breakpoint
   `AnimGetCollData` 0x1c4ac8 while a guard fires at Bond — confirm it returns count 16 and
   `&HitList2` boxes whose +0x50 part ids come from the human table; confirm the palette pointer
   (AnimObj+0xB8) differs per frame while walking (the anim-driven palette the port does not model).
2. The same, Bond crouched: dump bones 5 (head) and 52/56 (feet) translations — the expected
   drop of the head box (the port's idle-pose limitation's size).
3. `gdbwatch.py` on Bond obj+0xFA (expect bit 3 set in play) and AnimObj+0xC8 (expect 0x407 bits
   0/1) during a firefight — the [L] snapshot reading above is from level start; live confirmation
   mid-fight closes the gap.

## Usage

not visible to me (this session exposes no token, cost or context-window figures to copy).
