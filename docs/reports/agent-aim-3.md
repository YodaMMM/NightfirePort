# agent-aim-3

Base: origin/master + s/aim-15-agent + the door word repair from s/agent-rev-stream (door.rs and its test only).
No new random draws. The aim snap can land on a different point, so a round can hit something else.

## Collide_Pick and segment_aim_scoped read against the decompile
- Ray query [M 0x1ec690]: type 0x201 at HITTEST+0x94, flags 0x20a/0x208 at +0x98 (only those two are stored, disasm 0x1ec738/0x1ec750). +0x9a is 0, so `Intersect_RayGeom` 0x10a8c8 skips the per-triangle `Collide_Filter` and keeps each node's nearest front-facing triangle.
- `Collide_Intersect` 0x1ebaa8 then drops the whole node on that triangle: glass under bit 0x2; the pass bit under bit 0x8 (node word 0x40, byte 0x40, byte 0x80). The byte-0x80 `movn` at 0x1ebf94 is in the delay slot of the bit-0x1 branch, so it always runs. The old world.rs doc said otherwise.
- Fixed: `segment_aim_scoped` and the object and door stages now take the nearest triangle per node, then apply the node drop (`world::aim_drops`). Before, they skipped pass and glass triangles one at a time and could snap onto a wall behind a pane of the same node.
- Cel walk pre-filter `obj+0xF0 & 0x30000000` [M disasm 0x1eb910]: it only hands straddling and forced objects to the ForcedList walk (`build_LinkToRoom` 0x1c9680:62-65, `Controls_StraddleTest` 0x134778), which picks them [M 0x1eba38-0x1eba5c]. It never removes an object from the pick, so the port's skip was removed. Disc: 0 of 17609 objects carry 0x10000000 or 0x20000000 (new `aim_pick_words_probe`), so nothing changes on shipped data. That probe also finds 0 surface bytes with 0x80 on disc.
- PickObj gates [M 0x1eae80]: mask 0x60 matches; the +0x74 shooter skip matches; the +0x70/+0xfd gate does nothing here (+0x70 is the zeroed HITDATA holder); bit 0x200 class-5 does nothing (no bullets among the candidates). Script_EntityStart ORs 0xe4 into +0xfa, which lacks bit 8, so a script entity needs a collision glist. The port's `shape()` requirement already matches that; the comment was wrong and is fixed.
- The door stage reads `Door::f0` (the object's own word).

## 08 shutters
The old arm comment (the aim pick does not apply the 0x60 gate; the shutter passes on its object word) was wrong. PickObj applies the same mask; the shutter words are 0x0/0x201; 0 of 443 doors carry 0x60. The live chain's shutter object can be the glass leaf (DOOR_Shutter_2, every byte 0x10), which passes on the glass rule. The arm stays as the game's rule with a corrected comment. With the true word, the port's straight rocket from the gate-0 harness stops on the wood at z -96.706 (the game's usual stop). Three tests that pinned the pierce from that pose only passed through the stand-in, so they are now ignored with the question stated: rocket_glass08 `the_08_corridor_rocket_explodes_past_the_window`, rocket_stop_plane08 `the_straight_shot_pierces...` and `the_steer_line...`.

## 07000014 delta-exit real-damage walk
Isolated by builds of the walk alone:
- master + aim + door word: green (22385 ticks).
- master + aim + guard-alt-voice-set, no repairs (10e40e78f): red.
- full stack 261612e90: red.
- 261612e90 without the walk variant: red.
All three red runs fail the same way: walkreal_07000014_live.rs:781, the descent lands at (33.707, 1.201, 32.178), not at the leg-2 start. So the door fix and the walk variant are not the cause. The cause is the guard-alt stream shift against aim-15's walk, and it belongs to the stream join.

## Still open
- The game starts the ray in the player's cel (obj+0x20). The port uses `find_cel(camera)`. These can differ when the head and body are in different cels.
- Neither object candidate is gated on cel-chain membership: the game picks only the straddle cels' chains + ForcedList.
- Missing pick candidates: anim objects without a collision glist (`Collide_Jointy`), breakable panes, SS parts and pickups.
- Port-only stand-ins: the wire gfx-box stage, and drone hit boxes in place of Collide_Jointy.
- `Geo::bullet_pass` still leaves out byte 0x80 for the other bit-0x8 queries (none on disc).
- Render check: none of this is a draw change. The 08 corridor rocket stop point needs a live look (does the live round from the gate-0 pose stop on the wood or reach the pane).

## Tests
nf-game lib 689/0/5 (new `aim_refine_drops_the_object_on_its_nearest_triangle`); ai_live auto_aim 2/0; aim_pick_words_probe 1/0; door_object_pass_word_live 1/0; rocket_glass08 2/0/1; rocket_stop_plane08 2/0/2; rocket_origin 1/0; walkreal 01 7/0/1, 08 5/0/1, 0c 3/0/1, 14 9/0/1, 15 6/0/1, 16 14/0/2, 1b 17/0/2.
