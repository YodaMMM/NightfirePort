# agent-guard-breakables

**This branch moves the shared random stream.** A guard round that kills a breakable now runs
`Break_Kill` 0x12b970: the `Rand_Rand(100)` sound roll for non-glass kinds, then the debris
draws, and every RNG consumer after that shifts.

## What changed

Before this branch, a guard's round checked only the world and Bond (`Bullet::update`). It
never touched a breakable: a vase, a crate, a lamp or a glass pane in its path stayed whole.
The 07000001 walk-through glass drain (`Breaks::drone_step`) is a body contact, not a round.

The game makes no owner distinction anywhere in this chain:

- `Bullet_update` 0x12e7b0:388-443 sets up the same HITTEST pick for every round (+0x94 =
  0x101/0x201, +0x9a = 0x20 when `weapon_data+0x6C & 0x800` is clear). The owner is never read. [M]
- `Collide_Update` 0x1ea398:216-229 builds one sorted contact list and stops at the first solid node. [M]
- `Break_Update` 0x12bfd8:15-37 runs `Collide_FilterBullets(filter | 0x106)` [M 0x1ece30:24-33],
  then subtracts the hitdata damage for every class-5 contact. There is no owner test. [M]
- `Bullet_CollisionHandler` 0x12ce80: a kind-2 pane lets through a gun without `+0x6C & 0x800`
  (the glass arm). Any other object hit sets state 2, so the round dies (0x12cff0-4). [M]

The port now does the same:

- `Bullet::update_objs` (bullet.rs) passes this tick's flight segment, plus the fraction of the
  nearest world or Bond stop, to an object hook.
- The guard bullet loop (drone.rs) tests the live breakables against that segment.
- It uses the same filter word, contact damage (`bullet_contact_damage`, HITTEST+0x90) and
  glass-pass rule as the player walk.
- A solid breakable nearer than the world or Bond ends the round with the new
  `BulletEvent::Object`. A glass pane in front of the stop is drained and the round flies on.
- A kill runs plain `Breaks::kill`. The bark walk is skipped: `Drone_BrokenObject` needs the
  player's weapon system [M 0x1385ec-0x1385f4]. The kill records, script-pane removal and kill
  sound are presented the same way as in the player walk.
- Online (`net_bodies`), the shared world pass keeps ownership, as it does for Bond's rounds.

## Tests

`guard_breakables_live` (07000001, a real guard's own weapon record and owner):

- A round that crosses vase #22 breaks it and dies on it. A round 1.2 units to the side leaves it whole.
- A round that crosses pane #121 shatters it (kind 2, sound 0x2e).
- On master, both tests fail (the master drone.rs plus only the new enum arm). On this branch, both pass.

Neighbour runs on this branch:

- Pass: glass_breach_live, glass_dive_07000001_live, interact_07000007_live, fix_bond_damage_live,
  enemy_accuracy_live, bullet_range_live, fix_foliage_bullets_live, dead_pane_live,
  walkreal_07000007_live, and the nf-game lib bullet/breakable tests (37).
- interact_07000001_live: its unshot-scenery sister saw pane #136 (frame 2175) and vase #152
  (frame 2912) broken by guard rounds during the opening fight. The new `Drones::guard_break_kills`
  record confirms both were guard kills. The sister now allows #130 (the dive) and guard kills
  only, and it is 10/10 green.
- **walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_00agent`
  goes RED.** Bond dies at frame 3515 to drone #5's shotgun (0x1c) at 1.0 units. The walk passes on
  master. The likely causes are the moved random stream and/or panes shattered by guards no longer
  shielding Bond. I have not separated the two. The walk needs a re-pin or a new route when this
  merges.

Breakables per level [D census, `breakable_census`]: 01 55, 02 50, 03 97, 06 1, 07 8, 08 3,
09 17, 0a 45, 12 97, 13 33, 14 3. The other story levels have none.

## Unproven / not done

- No `Effect_Create` impact effect is drawn when a round hits a breakable. That is draw-side, and
  so is its drone-shooter `Rand_Rand(2)` [M 0x1ce94c]. Its RNG position is not modelled.
- The kill's draws land in the bullet pass, not in the next frame's `Break_Update` slot. This is
  the same compression the player walk already documents.
- Guard rounds still stop on world glass (`glass_pass: false`, the existing guard-side
  deviation). A pane that is also static world geometry drains first, then the round ends there.
- Live question for the real game: on 07000001, which hall panes are broken by frame 2175? The
  port breaks #130 at 2133 (the dive) and #136 at 2175 (guard 8's round). The existing [L] note
  ties 2175 to #130.
- I did not run a natural guard-fires-at-Bond-across-a-vase scenario live. The test stages the
  round the way `Drone::fire` builds it.
