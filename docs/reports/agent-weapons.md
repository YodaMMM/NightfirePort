# On-foot weapons and gadgets against the game

Scope: `Player_Weapon` 0x1a24c8, `Player_WeaponFiring` 0x1a5460, `Player_WeaponChange` 0x1a6f80,
`Player_SetWeaponAnimObj` 0x1a2e38, `Player_HandleHasNoAmmo` 0x1a2ba8, `Player_GetBestWeapon` 0x1a9680,
`Player_WeaponHasAmmo` 0x1a16c0, `Player_EquipWeapon` / `Player_EquipAmmo`, `Pickup_Handler` 0x18e650,
`Player_WeaponInitBullet` 0x1a4bf0, `Bullet_init` 0x12c5a0, the `weapon_data` table.

This branch moves the shared random stream whenever play reaches a changed rule (a dry gun switching,
a held trigger no longer firing at the end of a draw, the cycle's new first step, a refused press). It stays on its
branch and needs the stream batch's re-pin: the walks listed under "Still red" pass on master and fail here.

## Already matching

Damage, pellets, spread and bloom, refire, clip, ammo caps, zoom ramp and memory are read from the disc's
records through `GunParams` and `player_spread`, and the formulas match. `BestWeapon` ranks, pickup takes,
the clip fill on `+0x68 & 0x10` and the regen gates match, and so did the level loadout arms read.

## Fixed

| Rule | Game | Port before |
|---|---|---|
| Empty gun, nothing carried | on the trigger's release the hand goes to the twin (`+0x68 & 0x10`) or the best weapon; fists, Samurai, recharging gadgets stay [M 0x1a2ba8] | the dry gun stayed in hand |
| Last grenade thrown | case 0xc runs the same handover at the +0xB4 stop [M 0x1a2e38] | drew an empty hand of grenades |
| Best weapon | family base + the twin marker, AIMS-20 on its rifle [M 0x1a9680] | always the base: a silenced P2K lost its silencer |
| Placed mine | keeps the detonator's ammo check true [M 0x1a16c0:28-51] | the scan was a stand-in |
| Cycle start | from the request's base inside the mode's bounds [M 0x1a6f80:145-162] | from the held id: scrolling down from a silenced pistol went nowhere; the first step from another mode opened on the wrong end |
| Cycle with nowhere to go | no change [M :200] | went to hands and redrew |
| Quick presses | allowed during the holster and draw, walking on from the request [M :33-47] | dropped |
| Cycle refusals | aim bit, hands, 0x5D/0x5E, substates 2/10/12/13/16 [M :24-58] | scrolling on a ladder drew a gun; scrolling with the gun raised switched |
| View-state gate | no shot in the holster, draw, or aim raise/lower; the press edge is spent [M 0x1a5460:631-648] | a held trigger fired when the draw ended; shots went off during the scope raise |
| Refire timer | falls through a reload | froze |
| Reload key | trigger up only; refused with a scope raised [M 0x1a5460] | worked in both cases |
| Aim bit | scoped records drop for the reload and rise again with the button held [M 0x1a24c8:66-99] | the scope stayed up through the reload |
| Micro-Camera | fires only from the aim bit | took pictures from the hip |
| Q-Claw | no fire while reeling | fired |
| Guided rocket | a fire press blows it where it is [M 0x1a5460 → 0x12cd98] | no effect |
| Dry laser, trigger held | leaves the held script through `Player_WeaponHasAmmo` [M case 9] | parked in state 9, no regen |

Not fixed: the game fires the Q-Claw only inside a type-2 third icon's arm (`BL+0x95F == 2`) or in MP [M 0x1a5460
`uVar6 − 0x50 < 2`]. All seven campaign maps with grapple points carry such icons, set under or beside each point
(05: the ledge at (-18.46, 37.49, -72.49); 14: one on the floor under each point). With the gate on, the claw tests
pass from the icon spot, but the 07000014 walk's west-to-east crossing latches from (40.8, 2.4, 65.5), 5 units from
the nearest icon, and all eight of its tests go red. The gate is off until that route is re-walked from the icon.

## Tests

`crates/nf-view/tests/weapon_rules_live.rs` (15 tests, the real input path) and four unit tests in `gun.rs`. On the old
sources 13 of the 15 went red; `a_spent_gadget_stays_in_hand` is a sister that passes on both, and
`a_cycle_with_nowhere_to_go_keeps_the_gun` was tightened after that run to catch the old re-draw.

Existing tests changed because their setup leaned on an old rule: guns poked into the hand with an empty clip now
hand over, so they load the clip; held-trigger second throws became taps; claw tests stand in the icon's arm at
(-18.28, 38.19, -71.83); the matrix aims the camera first. The stun chain's bound rose to 4400: the long fall clip is
322 ticks and the removed second throw changed the `Rand_Rand(2)` pick.

## Still red (pass on master, fail here)

- `walkreal_0700000a_live`: 6 tests. After the Q-Worm install an alarm runner (drone 10) is chased and never
  dropped, and the mission fails on the walk back. Not traced to one rule; the gadget cycle's first step from the
  K5 (0x4A now, 0x5E before) changes the walk's timing.
- Real-damage variants, guard fights: `walkreal_07000001` (00agent), `07000003` (agent), `07000006` (agent),
  `07000008` (00agent), `0700000d` (the fall lands with 35.6 health, needs 40), `07000013`, `07000014`
  (infiltration), `07000016` (the chaser's punch). On 06 the SG5 and then the P2K run dry and hand over as the
  game does, and the fight goes differently.

The walk harnesses changed with the rules: `tick_fire` releases the trigger for the reload key and holds no press
through a draw (`common/walker.rs`, `common/walker_b2.rs`); the gadget meters are not reloaded by key.

## Unproven

- Not fixed: the game fires the Q-Worm only at an idle PC in the aim ray within `+0x1C`; the port plays its script
  anywhere. The port has no PC collision for `Check_Target`'s ray, and a stand-in gate refused the 0a install, an objective.
- The grenade refill at the +0xB4 stop moves the round at once; the game only probes there and moves it on the next
  released tick. Left alone.
- The claw bolt's in-flight latch (`BL+0x160`) has no port counterpart: the port latches on the press.
- `Player_EquipWeapon`'s 0x52 tail (taking a Ronin while holding its remote) was not compared.
- No live PCSX2 check was possible here. A question for the real game: on the last round with the trigger held, does
  the handover wait for the release, as the decompile reads?
