# Door guards — pairing and the zone read (07000007)

CivilianDoorGuard 0x31 (`DroneTypeSettings` row 22: start state 0x31 [L the table dump;
docs/fields/dronetype-tables.md]) pairs into mutual nearest-neighbour pairs and reads a
three-plane zone against its opponent every update. The port runs both halves as a tick
pre-pass (`Drones::door_guard_pass`, crates/nf-game/src/drone.rs) because the port's
`handle` lacks the list access the two need (the game pairs inside the 0x31 ENTER
[M 0x160694] and reads the zone inside the 0x31 UPDATE [M 0x160814]).

## NDrone2_SetDronePartner (mode 0x16, dc) 0x141360

Walks the obj list for the nearest eligible mode-0x16 guard:

- skip self [M 0x141360:70-72]; candidate kind byte `obj->drone->0xC5 == 0x16`
  [M :73]; enabled flags `+0x4F8 & 0x600 == 0 && & 0x100 != 0`, health > 0
  (`+0xAC`), class != 0x11 (`+0xFF`), not deleted (`+0xFE & 1 == 0`) [M :74-85];
  partnerless (`+0x2BC == 0`) [M :89].
- **The gate at :90 is a HEIGHT gate, not a yaw gate**: `ABS(me->obj+0x34 −
  cand->obj+0x34) < 2.0`. `+0x30` is the object's position VECTOR (Vec_SqDist2D at
  :91 indexes [0] and [2] = x, z [M 0x1e2ef8]), so `+0x34` is position Y; the yaw
  lives at `+0x54` (that is what :121 reads for `Vec_AngleDifference`). The two
  floats compared at :90 are therefore the two guards' heights.
- nearest by `Vec_SqDist2D(me->obj+0x30, cand->obj+0x30)` with a strict `<=` skip
  (only a strictly closer candidate replaces the best) [M :91-98].
- the pair is mutual: `me->0x2BC = cand_obj; cand->0x2BC = me_obj` [M :96, 113].

Planes (the caller's `Vec_AngleDifference(my_yaw /* +0x54 */, atan2(dir.x, dir.z))`
decides, `dir = normalise(cand_pos − me_pos)` [M :115-121]):

- diff < 0 → the CALLER's `+0x2CC = 0xff` and the planes are built on the CANDIDATE
  [M :123-133, disasm 0x1414f8-0x141554]; diff ≥ 0 → the CANDIDATE's `+0x2CC = 0xff`,
  planes on the CALLER [M :151-157, disasm 0x1415e4-0x141614]. The guard WITHOUT 0xff
  is the planes' owner (the evaluator).
- Both builds are the same shape: `v1` = the direction **owner→partner** (the if-branch
  negates `dir`, the else-branch keeps it), anchored at the **owner's own position**
  [M disasm 0x141520 (`$a0 = cand+0x2e0`, point `$s4+0x30`) / 0x141608 (`$a0 = me+0x2e0`,
  point `$s3+0x30`)]. The second vector is **(0, 1, 0)** — the disassembly stores
  0.0, 1.0 (`$f3` from `lui $at, 0x3f80`), 0.0 into the vector slots
  [M disasm 0x14150c-0x141554 and 0x1415f4-0x141614]. NOT (0,0,1).
- `plane_from_two_vectors_and_point` 0x1e4498: `n = cross(v1, v2)`,
  `d = −n·point` [M :11-17]. With v2 = (0,1,0): **plane0.n = (−v1.z, 0, v1.x)** —
  horizontal, perpendicular to the owner→partner axis. There is no degenerate zero
  normal for a pair whose axis runs along z (n = (∓1, 0, 0) there); the old test's
  "the game's own degenerate zero normal" claim came from the wrong (0,0,1) axis.
- The two derived planes [M :134-149 / :158-173; disasm 0x141618-0x14165c]:
  - `plane1 (0x2F0).n = (−n0.z, n0.y, n0.x)` = −v1 (horizontal, along −v1);
    `d1 = d0 + n0.x·owner.x + n0.z·mirror.x + n0.z·owner.z − n0.x·mirror.z`
    — algebraically `−n1·mirror_pos`, i.e. **plane1 passes through the MIRROR**
    (verified: the store chain cancels to that in both branches).
  - `plane2 (0x300).n = (n0.z, n0.y, −n0.x)` = +v1;
    `d2 = d0 + (n0.x − n0.z)·owner.x + (n0.z + n0.x)·owner.z`
    — algebraically `−n2·owner_pos`, i.e. **plane2 passes through the OWNER**.

## NDrone2_DoorGuardHandler (dc) 0x1416d8

- no partner (`+0x2BC == 0`) or no opponent (`+0x170 == 0`) → 0 [M :16-18].
- the 0xff holder returns the partner's `+0x2CC` (obj+0xe0 → drone) — the zone the
  owner stored [M :19-21]; the handler tail writes the zone INTO `+0x2CC`
  (`*(iVar4 + 0x2CC) = uVar5` [M :75-77]), so `+0x2CC` doubles as the mirror marker
  (0xff) and the owner's last zone (0..3).
- **The gate at :22 is a HEIGHT gate**: `2.0 < ABS(opp->obj+0x34 − me->obj+0x34) → 0`
  [M C :22-24; disasm 0x14170c-0x141738: `lwc1 0x34($t0)` / `lwc1 0x34($a2)`,
  `c.olt.s $f3, $f0` + `bc1fl` — continue only when NOT 2.0 < |Δy|]. It compares the
  OPPONENT's height against the evaluator's own; there is no aim-yaw anywhere in the
  function.
- plane sides `s_i = plane_i.n · opp_pos + plane_i.d` (the `_vmul`/`_vaddbc` chain
  against `opp +0x30`) [M :25-54], then [M :55-73]:
  - `s1 < 0` → 0; `s2 < 0` → 0 (the C keeps `s1 >= 0` in `bVar2` and tests `s2` first
    inside); `s0 < −6` → 0; `s0 < 0` → 3; `s0 < 2` → 2; `s0 < 6` → 1; else 0.

## What the zones mean (geometry)

With the corrected planes all normals are horizontal, so every side value is a
HORIZONTAL offset — none of them is a height:

- `s0 = n0·(p − owner)`: the signed lateral offset from the owner↔partner line
  (n0 = v1 rotated 90° in the horizontal plane).
- `s1 ≥ 0` and `s2 ≥ 0` select the slab between the two guards: s1 is the side of the
  plane through the MIRROR facing back toward the owner (n1 = −v1), s2 the side of the
  plane through the OWNER facing toward the mirror (n2 = +v1).
- bands on |s0| < 6: [−6,0) → 3 (mission-fail side), [0,2) → 2 (challenge), [2,6) → 1
  (face the player), |s0| ≥ 6 or outside the slab → 0.

The 0x31 UPDATE arms consume the zone [M 0x160648:38-105]: armed opponent → prop-0x33
shout + Attack; 1 → face + idle-anim replay on the step change; 2 → anim 0x5f variant 1
+ ChooseSFX(0x4f, 0x50) (the planes' owner only) + face; 3 → ChooseSFX(0x8f, 0x92) +
`DroneFunc_SetMissionFailReason(drone, 0xc)` + Attack; 0 → idle-anim replay. The idle
cycle 0x24 → 0x25 → 0x26 re-arms at frame + (Rand(20) + 5)·FRAME_RATE_INT [M :124-150].

## Sequential pairing: the claims overwrite each other

The 0x31 ENTER calls SetDronePartner **unconditionally** [M 0x160648 case 1], the claim
`me->0x2BC = cand` is written INSIDE the scan [M 0x141360:96], and the mutual write
`cand->0x2BC = me` lands after the scan [M :113] — nothing gates the caller and nothing
re-checks an earlier claim. The ENTERs of the guards that start in 0x31 therefore chain in
list order and later claims overwrite earlier ones. On 07000007 (all six guards at Y 44.0
[D the roster dump — the height gate never separates them here]) the four active guards
settle into: 404→417 (404's own claim), 417→442 (417's own re-scan — she is the nearer
partnerless guard), 441→442 (441's scan finds everyone claimed), 442→441 (442's own
re-scan) — only 441↔442 ends mutual, and 404/417's edges are the stale claims the later
scans left behind. The planes follow the same chain: 404 holds his own build (v0 =
404→417), 441 holds the build 442's ENTER made on her (v0 = 441→442), and 442 still holds
the build 417's ENTER made on her (v0 = 442→417) — unread, because 442 is a 0xff mirror.
417 holds no planes at all. The pre-audit port/test showed two clean mutual pairs only
because the old wrong yaw gate filtered the cross-pair candidates.

## Opponent (+0x170) in the port

The game's handler needs the drone's current opponent pointer (+0x170). The port stands
it in with "the guard currently sees Bond" (`los`) [S unverified stand-in — the real
perception write to +0x170 is not ported]; the height gate against Bond's position Y is
the game's [M 0x1416d8:22]. The port's `door_side` keeps only the 0xff marker; the
owner's zone lives in `door_zone` and the mirror copies the partner's stored zone — or
0xff when the partner is itself a 0xff holder, because the game returns the partner's
RAW +0x2CC and the mirror path returns early without storing [M 0x1416d8:19-21, 75-77]:
on 07000007, 417 (0xff) mirrors 442 (0xff) and so reads 0xff — the 0x31 UPDATE dispatch
`(lVar3 == 1 / == 2 / == 3)` falls through for 255 and the guard stands idle.

## Tests

`crates/nf-game/tests/door_guard_live.rs` (07000007, the only story level with door
guards [D the census in the 737 report]) pins: the mutual pairing and the height gate,
the exact plane triple from the game's formulas, the zone bands through the real tick
path, the armed-opponent sister, and the waiting pair (pl 448/449, switch channel 21)
activating through its channel.
