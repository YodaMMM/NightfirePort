# A placed gadget shot from range (the stuck tripbomb / satchel)

What a player round does to a stuck class-5 gadget (the laser tripbomb 0x3a, the satchel 0x3b).
Port: `Grenade::round_pick` + `grenade::destruction_gate` (crates/nf-game/src/grenade.rs), the
candidate pass in `Drones::update_player_bullets` (crates/nf-game/src/drone.rs). Pins:
`crates/nf-view/tests/tripwire_shot_live.rs` (07000014, a real throw + real P2K rounds from 16+ u)
and the `grenade::tests` unit tests (the radius edge, the record gate).

## The game's chain [M ACTION.ELF]

1. **The gadget is a pickable object.** `Bullet_update` 0x12e7b0 state 0 sets the object's glist
   from the record's `+0x5C` (`hashtable_set_object_to_entity_gfx` 0x1ca6d8 → `Control_SetGList`
   0x134960: obj+0xD8 = glist, obj+0xF0 = glist+0x44). For records without `+0x6C & 0x100`
   obj+0xE8 stays 1.0 [:122] (0x3a/0x3b carry +0x6C = 0xc08 [L]).
2. **The round's pick.** The bullet's HITTEST mask word +0x98 is 0 [M 0x12e7b0:389], so no class
   is skipped; `Collide_PickObj` 0x1eae80:57-90 admits an object with obj+0xF0 & 0x20 clear and
   a glist carrying collision (`glist+8`). Its 0x200/0x201 arm (:92-107) keeps the object when the
   flight segment's closest point to the world sphere centre obj+0x80 lies strictly inside
   obj+0x8C. `Control_BuildWorldSph` 0x133070:36-48: centre = obj+0x90 · glist+0x10, radius =
   glist+0x1C · |obj+0xE8|.
3. **The refine.** `Collide_Update` 0x1ea398:84-100 calls `Collide_Intersect` 0x1ebaa8 per node;
   the object arm (+0x54 == 0, obj+0xD8 != 0) tests the segment in object space against the
   glist's collision with the one-sided ray (`Intersect_RayGeom` 0x10a8c8:168). A miss leaves
   the node at 1e8 and it is freed (:122-125); the rest sort by fraction (:130-150).
4. **The take.** `Bullet_CollisionHandler` 0x12ce80's object arm (disasm 0x12cff0-0x12d004): the
   round dies (state 2); with no pass bit (obj+0xF0 & 0x60, HITDATA+0x50 & 0x40 / random 0x80)
   and the round's `weapon+0x0C` > 0 it calls **`Bullet_handle_object_destruction` 0x12cd98** on
   the hit object.
5. **0x12cd98.** Class 5 only; the VICTIM's record must carry `+0x68 & 0x4000` and `+0x70 &
   0x80204`. Then obj+0xFE |= 1 (deleted), `Explode_Create(rec+0x08, rec+0x08, rec+0x0C, victim,
   obj+0x30, …, 0x0600_0052, …, owner BU+0x30)`, and for `+0x70 & 0x80000` the beam object at
   BU+0x38 is deleted too.

## The data [D]

The tripbomb's glist 0x0200074f (07000013/14/15/16/41 and the MP arenas; NOT in 0700000a or
0700000b's archives): bounds centre (0, 0, −0.1), radius **0.24266**, box x ±0.2278, y ±0.1042,
z −0.2064..0.0064, flags 0x880 (no 0x20 pick skip, no 0x40/0x20 pass), 1312 collision tris, every
surface byte 0. Record 0x3a: +0x08 = 8, +0x0C = 100, +0x68 = 0x14080, +0x6C = 0xc08, +0x70 =
0x80100 — the gate passes with the laser tail. P2K (record 2): +0x0C = 3.5 > 0.

The stuck pose: `Mat_Align2Dir(−normal, (0,1,0), (1,0,0))` [M 0x12ce80:256-266] at obj+0x30,
so the body's local −z (the face turned out of the wall) points along +normal and the sphere
centre sits 0.1 u out from the stuck point.

## Port notes

- The candidate set is the port's stuck grenades (`Grenade.stuck`). The game's pick does not need
  the stick — an in-flight gadget with its glist is pickable too; the port leaves in-flight
  projectiles out of the round's candidates [deviation, documented].
- The nearest solid node takes the round: the gadget competes with the static world stop and the
  0xFE mines by segment fraction.
- The deleted tripbomb's beam: the port rebuilds `tripbomb_beams` from the live armed tripbombs
  each tick, so a deleted gadget draws no beam from the same tick's grenade pass on.

## The level beams on 0700000a / 0700000b / 07000014

They are 0xEB `laserbeam` / `laserbeam-large` touch triggers (mask 8, the player touch), not
gadgets [D docs/levels/0700000a.md, 0700000b.md]: `Trigger_Hit` 0x1b1fc8 reacts to class-5 (bullet)
contacts only with mask bit 2, so a round through one of them does nothing in the game either.
