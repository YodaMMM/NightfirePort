# MP_HitBy's real contact test (mp-hitby-contacts)

The MP scenario objects (flags, bases, uplinks, the hill, the sites, the GoldenEye items, the
blueprint) are touched through the game's `MP_HitBy` overlap walk. This doc pins the walk, the
contact geometry that fills the list it walks, and the spawn window. GAPS G-053 item 2.

## The list the game walks

`MP_HitBy(obj, team, exclude, out_team)` 0x1830e0 walks the object's HITDATA contact list at
`obj+0xD0`:

- each node's `+0x58` is the object at the other end of the contact [M 0x1830e0:20-27];
- the node qualifies only when that other object's kind byte (`+0xff`) is 2 or 3 — the player
  kinds [M 0x1830e0:28];
- the matched `MPGame + slot·0x30 + 0x1c` must equal the node's object, and the team word
  `MPSettings + slot·0x30 + 0x20` must equal `team`, unless `team == 2` (any) [M 0x1830e0:32-35];
- the first qualifying node's body wins; `out_team` gets the slot's team word [M 0x1830e0:76-79].

The list is rebuilt every frame by `Collide_Update` 0x1ea398: it frees every object's `+0xD0`
list [M :25-53], runs `Collide_Pick` 0x1eb320 per object with a HITTEST (`obj+0xD4`) [M :84],
refines the candidates with `Collide_Intersect` 0x1ebaa8 [M :93], drops the misses (the `1e8`
distance marker) [M :119-139] and copies each survivor to the other object's list with the
copy's `+0x58` pointed back at the mover [M :160-215]. The nodes on a scenario object's list
are therefore exactly this frame's contacts with moving bodies.

## The contact law (the pick's sphere test)

The body is the mover: its HITTEST type word (`HITTEST+0x94`) is 0x100 (sphere) and only a
0x100 HITTEST reaches the pick's sphere branch [M Collide_PickObj 0x1eae80:143-148]. The test
against a candidate object is sphere vs sphere:

```
sqmag(HITTEST+0x60 − obj+0x80) < (HITTEST+0x6c + obj+0x8c)²      // STRICT [M 0x1eae80:221-228]
```

`Collide_Intersect`'s fresh-node 0x100 path recomputes the same pair from `obj+0x80`/`+0x8c`
[M 0x1ebaa8:259-272] — no finer geometry for the body sphere (the per-sphere/triangle
`Intersect_*Geom` walks are the bullet/point/ray/cylinder types' paths [M 0x1ebaa8:181-297]).

## The object's world sphere

`Control_BuildWorldSph` 0x133070 builds `obj+0x80`/`+0x8c` from the placement:

- `obj+0x90` holds the placement matrix: `MP_RegisterMPObject` 0x185990 does
  `RotMatrix(rotation)` + `Mat_SetTrans(pos)` for every type [M 0x185990:65-66, 88-90 and each
  case's pair], and `MP_CreateObject` 0x185de8 copies it to `obj+0x90` [M 0x185de8:115];
- with the celglist at `obj+0xD8` [M 0x185de8:118] and the joint bit clear (`obj+0xfa & 8`),
  the centre is **the local sphere centre through the matrix**:
  `obj+0x80 = cel+0x10 · M(obj+0x90) + obj+0xC0` (the row-vector products
  `_vmulabc/_vmaddabc/_vmaddbc` of the three matrix rows plus the translation row) [M
  0x133070:25-36];
- the radius is `cel+0x1C · |obj+0xe8|` (the placement scale; `control_init_object` 0x134630:19
  sets `+0xe8 = 1.0`) [M 0x133070:37-38, 0x134630:19].

`RotMatrix` 0x1e2368 builds the matrix from the placement's Euler vector with three `PS2Sinf3`
sine lanes per call [M 0x1e2368:28-40]:

```
m[0]=cy·cz   m[1]=sx·sy·cz+cx·sz  m[2]=sx·sz−cx·sy·cz
m[4]=−cy·sz  m[5]=cx·cz−sx·sy·sz  m[6]=sx·cz+cx·sy·sz
m[8]=sy      m[9]=−sx·cy          m[10]=cx·cy
```

(x = pitch, y = yaw, z = roll; cx = sin(x+π/2) etc. through `PS2Sinf3` 0x11fc20 — range
reduction then the sine polynomial.) The world centre offset is the local centre
`c` against the rows: `off_k = c.x·m[0][k] + c.y·m[1][k] + c.z·m[2][k]` (the translation row
stays the object's `obj+0xC0` position) [M 0x133070:25-36].

So a rotated placement moves the contact sphere: the port's old stand-in (`pos + unrotated
centre`) is wrong for exactly the placements whose rotation is non-identity and whose local
sphere centre is off the rotation axis.

## The body's sphere

`HITTEST+0x60`/`+0x6c` — the body's world sphere. The port keeps its measured stand-in:
radius 1.0576 (`M_CONTROL+0x8C` [L ee_start_05]) and the centre at
`Bond::pos + (0, −0.1267, 0)` (the measured `M_CONTROL+0x80` y [L ee_start_05]; the game's
centre also carries a small yaw-dependent horizontal offset the match state does not keep
[S stand-in — `MpMatch::slot_centre`]).

## The spawn window

A body within 4 frames of its spawn stamp can never be the hit:

```
uVar6 = GameState+0x52 − obj+0xec;  if (|uVar6| < 4) return 0;   [M 0x1830e0:36-39]
```

`obj+0xec` is stamped with `GameState+0x52` (the frame counter) at object init
(`control_init_object` 0x134630:15), at every respawn (`MP_ReSpawn` 0x185368:90) and at the
bot's spawn (`BOT_respawn` 0x124e38:91). The port maps the counter to `MpMatch::tick`
(advanced once per `update_objects` — the fixed 60 Hz) and stamps `MpSlot::spawn_tick` at the
match's start (the `Player_Init` frame [S alignment]) and in `do_respawn` [M 0x185368:90].

## The remaining state gates

After the window, `MP_HitBy` rejects: the dead/dying kinds through the player struct's
`+0x4f8` bits 0x600/0x100 and `+0xac > 0` [M 0x1830e0:41-64], kind 0x11/0x12 outright [M
0x1830e0:66-72] and kind 3 with `+0xf4` in {2,3} [M 0x1830e0:73-75]. The port keeps its
`MpSlot::dead` marker for this half (the players have no BLData pipeline yet *Deviation*).

## What the port does now

- `MpObjBounds` carries the disc's celglist record [D `MapObject` centre/radius/min/max] plus
  the port's derived `sphere_off` — the local centre through the placement rotation [M
  0x133070 + 0x1e2368], computed once in `register_mp_object_bounds` (the real registration
  path); `[0,0,0]` when the placement is unrotated or registered bare.
- `obj_hit_by` tests the body sphere against `pos + sphere_off` [M 0x1eae80:221-228 law].
- `hit_by` applies the team filter [M 0x1830e0:32-35], the spawn window [M :36-39], the dead
  gate [M :41-75] and the strict-`<` sphere law.
- The walk order across several simultaneous bodies stays the port's slot order [S — every
  overlapping body contact's refined distance is 0 [M 0x1ebaa8:269-271], so the frame's sort
  cannot order them; the single-body cases the rules react to are unaffected].
- The KOTH hill box test keeps its own pinned law (`MP_isPosOnHill` 0x18bc20 — the unrotated
  point-in-box, radius 0.0) — untouched here.
