# s/sim-gaps-c: "not ported" notes in nf-game (copter, door, death, bullet, autoaim and the rest)

Scope: every `not (yet) ported / not modelled / unmodelled` note in crates/nf-game/src outside
drone.rs, car.rs, mp/ and casing or driving code. Each was checked against the cited decompile
and a crate-wide grep for the address and function name.

**The pull-back changes guard routes, so the shared random stream moves on several levels.**
Six walks go red with it and green with the call disabled (same build, A/B):
07000001 `walk_from_the_spawn_to_the_exit_with_the_contact` (6000-tick wedge),
07000002 `walk_the_castle_all_objectives_real_health_00agent`, 0700000d
`walk_arrival_to_the_c5_door_real_damage`, 07000014 `walk_the_tripbomb_route_pick_up_place_and_shoot_it`,
07000015 `walk_spawn_to_the_omega_sector_exit_agent` / `_00agent`. Every other walkreal binary
(03-0c, 11-13, 16, 1b) and enemy_accuracy_live pass. The walks need re-pinning before a merge.

## Ported

| file:line | rule | test |
|---|---|---|
| nav.rs `Route::to_position` / `alter_dest_nearest` | `AINetwork_AlterDestFor_DROUTE_Nearest` 0x156088: a route whose A* goal is the fallback node (result 1) move-tests from its last node to the destination; when the line crosses an AI boundary the destination becomes the crossing pulled 1.0 back toward that node [M disasm 0x156088-0x15615c; callers 0x15afa8-0x15afb4 and 0x15afe0:33-35]. Guards chasing a target past an AI boundary now head for the boundary instead of the target behind it. | `nav::tests::a_fallback_route_pulls_its_destination_back_from_the_boundary` (fails with the call removed) |

## Not ported, with the reason

| file:line | rule | verdict |
|---|---|---|
| copter.rs:26-36, :212, :235 | spot check, attack state, timer | already ported in `Copter::update` (stale notes corrected) |
| copter.rs:548 | gun-B part recharge 0x17dbb8:104-113 | draw only: gun B is the two-tick muzzle part Copter_FireGun 0x17d828:94-95 unhides |
| door.rs:193 | portal link bit | already ported (nf-view `build_LinkDoors2Portals`), note corrected |
| door.rs:348 | player skip-bit runtime writers | already ported (`Drones::bond_door_skip`), note corrected |
| door.rs:701 | anti-crush via the collide path | already ported (`Doors::collide_step`), note corrected |
| death.rs:182 | footprint decal | visual |
| bullet.rs:375 | flashbang fade | view-side Camera_SetFade, visual |
| autoaim.rs:24 | libm tanf chain | host tan, last-bit only, nothing a player sees |
| nis.rs:1776 | SP_Hit touch bits 8/0x10 (player/guard contact) | not player-facing: no on-foot SP placement uses mask bit 8 or 0x10 [D census over 01-1b] |
| nis.rs:160, :251, :465, :553 | inert scripts / fades | inert or view-side |
| worldobj.rs:291 | mine spin obj+0x54 | not player-facing: all nine 07000015 mines carry param 2 = 0 [D] |
| pickup.rs:264 | AR grenade pile offset/velocity | MP only: PickupMatrix row 0 (SP) has no AR |
| pickup.rs:731, player.rs:1794, hud_pane.rs:211 | HUD flash / air pane / goggles | HUD/visual; breath sounds already play in nf-view (note corrected) |
| gunimp.rs:11 | Collide_PlaceObject | already ported as `gt::place_object` (note corrected) |
| rigid_body.rs:313 | impact sound | already played (drone.rs rb_sound_events) |
| hurt.rs:246, light.rs:159, sky.rs:415 | sound id / light param / sky skip | inert on disc data |
| gas.rs:25, gt.rs:703, searchlight.rs:545/700, sensor.rs:25 | hidden emitters, smoke puff, LOD swap, lens | visual |
| ladder.rs:132, triggers.rs:272 | glist mesh refine for ladder pick / trigger contact | too big (Collide_Intersect per candidate) |
| triggers.rs:445, world.rs:94/919/2349 | Dist2Tri re-anchor, class 0, 0.7 gate | inert by the cited trace or an open live check |
| rigid_body.rs:422/695, sdk.rs:37, useobj.rs:634, wire.rs:174 | slot widths, never-run printf, test note, creep walls | not player-facing / already ported (creepwall.rs) |

## Unproven

- The port's `bounds_test` hit height (post A's y) is the port's existing reading of
  `AINetwork_LineIntersectsBoundry` 0x157990; the pulled-back destination inherits it.
- The game re-enters `CalcRoute` on a still-valid route (0x15af68 branch); the port builds once per
  target stamp, so the pull-back applies once per build.
