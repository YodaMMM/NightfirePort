# Drone rounds on other drones

This moves the shared random stream: a round that now lands on a drone runs that drone's msg-8
handling (pain talk, flinch roll) at the hit, and a round that now ends on a body skips the
world-impact draws it used to make further on.

## What the game does

- `Bullet_update` 0x12e7b0:389-421 arms the flight HITTEST with object filter word +0x98 = 0 and puts
  `BU+0x34` in exclusion slot +0x74. `DroneWeap_FireWeapon` 0x178f68:253 passes the drone as both
  owner arguments of `Bullet_init`, so only the shooter is left out. `Collide_PickObj` 0x1eae80:57-82
  then offers every other drone not masked by `obj+0xF0 & 0x20`.
- `Bullet_CollisionHandler` 0x12ce80 sets state 2 on an object contact, so the round ends on the first
  body it reaches, whichever side that body is on.
- The struck drone's `NDrone2_DealWithObjHit` 0x13fff0 case 5:
  - weapon kinds 0x1b, 0x2a, 0x2b, 0x2e-0x31, 0x34-0x37, 0x3a, 0x46, 0x48, 0x49, 0x4e, 0x50, 0x51, 0x5f
    never land [:154-173]; 0x6e does not land on voice set 0x10 [:174-178];
  - sides [:181-232]: a team-1 shooter lands on teams 2 and 3, a team 2/3 shooter lands on team 1 (and on
    any other team); same side never; the victim must be alive [:229-230];
  - the message: kinds 1/0x4b/0x4d punch (msg 6), 0x4a/0x4c tase (7), 0x43/0x44 dart (0x19), else
    `Drone_BulletHit` 0x13a9f8 (msg 8, only when HITDATA+8 > 0 [:20]). Both arms stamp +0x2B4 with the
    shooter [0x13a9f8:23, 0x13fff0:302-304].
- `NDrone2_HitDamage` 0x145558:58 logs the shot-hit stat only when +0x2B4 == +0x170. A team-1 guard's
  +0x170 is Bond, so a drone's round on a guard does not count toward Bond's accuracy.

## Port

- `Bullet::update_with` takes a drone picker. The guard-bullet pass in `drone.rs` supplies it for
  drone-owned rounds. Copter, turret and space-laser rounds (owner `usize::MAX`) and the satchel family
  are unchanged. The picker is the same body test as Bond's rounds (`drone_ray_part`, factored out
  unchanged) behind the same enable gates.
- New `BulletEvent::Drone`: the nearest of world, Bond and drone wins. `drone_round_applies` holds the
  gates above. The hit fills the victim's `Hit` with `bullet_contact_damage` (HITTEST+0x90), clears
  `hit_by_player` and sends the game's message, using the same flinch and aim lending as Bond's rounds.
  The hit sound uses the same gate as on Bond's rounds.
- `do_hit_effects` logs the shot-hit stat only for `hit_by_player`.
- `Drones::drone_round_hits` records (tick, shooter, victim).

## What a player would notice

On 07000012 a guard who fires through the office civilian now kills her, and that fails the mission
(param 12 = 0x63). An armed ally's rounds now wound guards. A guard's round now stops on another guard
standing in its line instead of passing through.

## Tests

`crates/nf-view/tests/ally_bullets_live.rs`, real levels, rounds placed 2.5 units off the victim
on a clear bearing:
- `a_guards_round_kills_the_12_civilian`: 07000012 guard #12's round on civilian #1, then the 0x63
  fail.
- `a_guards_round_stops_on_another_guard_without_harm`: 07000012, a team-1 round on guard #13. No
  damage, and the round is gone after one tick.
- `the_08_ally_and_the_guards_hit_each_other`: 07000008 ally #265's round wounds guard #251 with no
  shot-hit stat. The guard's round reaches her (she has no damage of her own).

On the old `drone.rs`/`bullet.rs` all three fail. On the new code all three pass.

Other runs on the new code:
- Green: fix_ally_death_live, fix_actors_live, fix_guard_weapons_live, fix_guard_first_shot_live,
  fix_guard_misc_live, enemy_accuracy_live, fix_bond_damage_live, bullet_range_live,
  fix_foliage_bullets_live, guard_stand_numbers_live, ninja_melee_live, the walkreal_07000012 real-damage
  pair, walkreal_07000008 (4 of 6), and the nf-game lib tests (689).
- **Red: walkreal_07000008 `…real_damage_agent` and `…real_damage_00agent`.** Both are green on the base.
  The walk has one drone-on-drone contact: at frame 2021 guard #8's round stops on guard #7 (same side,
  no damage). Every draw after that is shifted. Bond dies at tick 27319 (agent) / 34047 (00 agent), with
  drone #20 landing 9 hits from 61 units. This is a stream shift, not a rule found wrong. The walk needs
  re-running after the re-pin.
- **Red: walkreal_07000012 `walk_spawn_to_the_shaft_exit`.** Guard #4's round kills civilian
  placement 24 (team 3, state 0x64, hp 1) at frame 2539. The mission then fails, and the walk's
  restart leaves door 545 shut at its check. The hit follows the game's side rule as read
  (0x13fff0:181-232 lands team 1 on team 3). Which prop of placement 24 turns her death into the fail
  was not traced, and it has not been checked live that a guard's crossfire kills her. The walk needs a route that
  keeps her out of the line, or a live check.

## Death channels

The ally-death report says the death channel was raised on `Death_Anim`'s UPDATE. That is already
changed on `s/agent-lvl-01-02` (5410f2212: the UPDATE raise is removed, and the channel lands at the
fall's end or the 07000002 90-tick timer). It is not duplicated here, and this branch does not
contain it.

## Not done, not proven

- The victim's blood puff (`Gas_Init` 0x20008bc / 0x200026e, 0x13fff0 case 5) is a visual and is not
  added for drone rounds. Bond's rounds make it. Drone hits on drones show no puff yet.
- The +0x5D0 hit push (0.35 · round velocity, capped 0.175) is not ported for either shooter.
- Copter and turret rounds still pass through drones. In the game they should end on a body without
  harming it (their owner is not class 2, so `bVar3` stays false). This is not checked live.
- No test shows a fight from the AI alone. On 07000012 the civilian runs before a guard's line
  crosses her. On 07000008 the ally stays idle until Bond comes. The tests place the round directly.
- The same-side stop and the hit order are read from the decompile only. Neither is checked live.
- The blast arm's shot-hit stat (`explosive_impact`) still logs for any killer. The game uses the
  same +0x2B4 == +0x170 test there [0x145558:121]. It is left unchanged.
