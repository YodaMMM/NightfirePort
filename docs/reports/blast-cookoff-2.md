# blast-cookoff-2

Base: `s/blast-cookoff` (the `Explode_Propagate` case-5 cook-off sweep).

## The 07000014 tripbomb walk

`walkreal_07000014_live::walk_the_tripbomb_route_pick_up_place_and_shoot_it` failed at the
retreat check ("the placed tripbomb is still stuck at the retreat") on the cook-off branch and
passed on master.

Cause: a guard (placement #375, one of the squad that follows Bond into the armoury) throws a
frag 0x34 a few ticks after the tripbomb sticks on the rack wall. The frag lands in the pit at
(-38.22, 0.04, 34.79) and its 8/100 blast fires 6.4 u from the bomb at (-32.10, 1.75, 36.30):
falloff 100·(1 − 6/8) = 25 > 0.0002, so the case-5 arm cooks the bomb and its own 8/100 blast
fires on the next tick. On master the case-5 arm did not exist, so the same frag left the bomb
alone.

The port matches the game here:
- `Explode_Propagate` 0x17c518 sweeps `Collide_SphereIntersect` hits; case 5 sends ids 0x2B,
  0x34-0x37 and 0x3A to `Bullet_handle_object_destruction` 0x12cd98 once the falloff passes
  0.0002. There is no line-of-sight test [M].
- 0x12cd98 checks class 5, then `(rec+0x68 & 0x4000) && (rec+0x70 & 0x80204)`. The 0x3a
  record passes both (the new pin below goes through that gate on disc data) [M, D].
- `Collide_PickObj` 0x1eae80 only skips an object when `obj+0xF0 & 0x20` is set, or by the
  hit-test's class masks. The propagate call passes no ignore object and has no 0x200 (class 5)
  skip. `Bullet_init` creates the round through `control_create_object` (memset 0, `+0xFF = 5`)
  and never sets F0 bit 0x20 [M 0x12c5a0:83-90, 0x1344e0:11-17].

So the walk relied on the old behaviour. The fix is in the walk, not the code: if the placed bomb
goes off during the first retreat leg, the walk goes back to the rack-front node and places
another (the gun still holds 4) at the first throw's aim height. On master the loop exits on
attempt 1, so that path is unchanged. On this branch attempt 1 cooks and attempt 2 survives,
arms, and the P2K shot sets it off with round 1.

## The 0x80000 arm

0x12cd98:28-34 also delete-marks the attached beam object (BU+0x38) when the record's +0x70
has 0x80000 (0x3a alone). The port rebuilds `tripbomb_beams` every tick before the blast drain.
That left a cooked bomb's beam in the list for the cooking tick after the round itself was gone.
Both cook-off sweeps (`spawn_blast` and the beam-blast sweep) now drop that beam in the same
call. Pin: `blast_cooks_off_grenades_live::blast_cooking_a_tripbomb_drops_its_beam_the_same_tick`
fails without the retain and passes with it. The only observable effect is one frame of beam
list content.

## Unproven

- `Collide_SphereIntersect`'s cel scoping (`Collide_StraddleCels`) is not modelled, the same as
  the port's other blast cases. The bomb lies inside the 8 u blast sphere, so its cel overlaps
  the sphere. Whether StraddleCels finds that cel through the pit/lane portals has not been
  checked live.
- Whether a guard really throws a frag at that moment in the game is a property of the AI sim,
  and nothing here checks it.
