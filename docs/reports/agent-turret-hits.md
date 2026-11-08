# Turret hits use the real collision models; guard rounds damage turrets

Random stream: compared with master, no draw is added or removed per round. A round that hits a
turret part follows the same world-stop path master runs (impact effects, penetration roll,
ricochet), and the hit now also counts against the gun. The stream can still move, because a gun
now dies at a different time (guard rounds and ricochets count, and sphere-only grazes don't).
Compared with the base branch `s/agent-updates-lbl` the stream does move. That branch killed a
round silently at a turret's sphere and skipped the world impact draws. This branch drops that
early stop, so those rounds draw their impact effects again, and grazes fly on. Keep this branch
off master until the stream batch.

Scope: `Collide_Update` 0x1ea398, `Collide_Pick` 0x1eb320, `Collide_PickObj` 0x1eae80,
`Collide_Intersect` 0x1ebaa8, `Collide_GetDamageNObjects` 0x1ee220, `Bullet_init` 0x12c5a0 and
`Bullet_update` 0x12e7b0 (HITTEST set-up), `GT_Update` 0x180498 (contact sums, fire), `GT_Create`
0x17ffa0 (ext+0xD0).

## What the game does [M]

- A round's HITTEST is +0x94 = 0x201 (a ray) on every flying tick. +0x90 is the weapon's +0x0C,
  with the explosive and throwable overrides (`bullet_contact_damage`). +0x98 = 0.
  +0x74 = `Bullet_init`'s third argument [0x12e7b0:415-432].
- Broad phase `Collide_PickObj`: an object passes when obj+0xF0 & 0x20 is clear and its glist has
  collision data (glist+8). The line's closest point must also fall inside the world sphere
  obj+0x80/+0x8C [:57-108]. The round's own objects (+0x70 the bullet, +0x74, +0x78) are skipped.
- Narrow phase `Collide_Intersect`: obj+0xD8 = 0 returns no contact [:29-31]. Otherwise the
  segment goes through the inverse of obj+0x90 and `Intersect_RayGeom` tests it against the glist
  triangles, one-sided. A candidate still at 1e8 is freed [Collide_Update :119-158].
- `Collide_Update` sorts the contacts and copies each one onto the struck object's +0xD0 chain,
  with +0x8 = HITTEST+0x90. The walk stops at the first solid contact [:160-233]. The same pass
  runs for every HITTEST owner, so guard rounds land on turret parts exactly as Bond's do.
- `GT_Update` sums every node of script A's objects every tick. It sums script B's only in
  state 4 [:128-176].
- `GT_Update` fires `Bullet_init(-1, ext+0xB8, ext+0xC4 = the barrel, ...)` [:347-348, :519-521].
  So a turret round skips its own barrel object only.

## Fixed

1. Turret contact is the model, not a sphere. Each script-A/B entity of each turret is tested
   in two stages: its world sphere first, then its dynamic-set triangles
   (`CollisionWorld::segment_dynamic_part`, the same one-sided test the flight trace runs). The
   hit counts only when that part is the round's world stop. A round that grazes a part's sphere
   but crosses no triangle no longer hurts the gun. The muzzle-flash glists (0x20005ab on
   0a/0b, 0x2000361 on 12 and the Ronin) have no triangles, and their 0.36 to 0.44 spheres
   counted under the old test. They no longer do [D census: every other part carries 86 to 490
   triangles, surfaces 0, 5 and 6 only, no pass or glass bit].
2. A round that hits a turret keeps the normal world-stop path: impact effects, penetration and
   ricochet. The base branch ended the round silently at the sphere. Because a metal ricochet off
   one part can land on the next part, one round can count more than once, as the game's per-tick
   re-pick allows.
3. Guard rounds damage turrets: the guard bullet pass credits the struck part. Real case: 0b's
   five guns stand in one room, and rounds from guards (and from the other guns) can now wear a
   gun down.
4. The old rule that "record 96 rounds never count against a turret" is gone. A turret's round
   now skips only the turret that fired it (`from_turret` on both bullet kinds). Other guns'
   rounds count.

## Deviation kept [S]

- The game skips only the barrel (ext+0xC4) for a turret's own round. The port skips every part
  of the firing turret, because the port's muzzle is not the game's. The port fires from the
  turret's placement translation, with a muzzle offset of 0 for placed guns. The game fires from
  the barrel's obj+0xC0 + dir·ext+0xD0, where ext+0xD0 = 1.5 × the max glist-bbox z over script
  A's objects [M 0x180498:324-337, 0x17ffa0:121-139]. On 0a the placement sits 1.23 below the
  barrel. With only the barrel skipped, a port round could start inside the gun's own pedestal.
  Fixing the muzzle is a separate job (the glist bbox max is in `MapObject::max` but not in
  `Data::models`), recorded here and not changed.

## Tests

`turret_hits_live` (new, 0700000a's first wall gun, real parts and the real dynamic set):
- a round through a part's sphere that crosses no triangle takes nothing off; a round onto a
  script-A part takes whole contacts off. Red on the sphere code (the graze counted).
- a guard round on the same part damages the gun, and the graze line stays a miss. Red before:
  the guard pass fed no turret.
- a turret round skips its own gun but counts on another. Red on the record-96 rule.

`agent_updates_lbl_live::a_wall_gun_can_be_shot_down_0a`: the own-round sister now passes the
gun's own key instead of relying on the record number.

## Unproven

- obj+0xF0 bit 0x20 on turret parts: nothing in `GT_*` writes it. The writer was not traced
  further, so the port does not gate on it.
- The 0x80 surface random roll in `Collide_Update` :217-220 is not modelled. No turret triangle
  carries 0x80.
- Ricochet counts follow the port's existing ricochet rules, which this branch did not re-read.
