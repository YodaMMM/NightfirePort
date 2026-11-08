# agent-lvl-05-06-b: the raw-plate contact on The Exchange

Builds on `agent-lvl-05-06` (its two commits are cherry-picked first). That branch fired 07000006's
arrival plates with a tuned reach (`LOW_REACH` 0.265, grounded standing Bond, directly over the
plate). This branch replaces it with the game's own contact test.

## The game's rule [M]

- `Collide_Intersect` 0x1ebaa8:93-99: a glist object whose flags carry bit 0x40 sets
  HITDATA+0x51 |= 2.
- `Intersect_CylGeom` 0x109da0: the anchor point is the segment's plane crossing, clamped to its
  ends (:302-320). `ASH_vecutil_Dist2Tri` 0x109770 returns the true distance from that point to
  the tri, or a negative value (code 99) behind the tri's own plane record. With +0x51 & 6 set
  (:421) the push-out arm is skipped and the tri is a contact when that distance is under the
  HITTEST radius (:419, :523-531). There is no other reach limit.
- The walk HITTEST is Bond's collide segment: standing A = pos + 0.275, B = pos + 0.55 − cc,
  r 0.55; the crouch, climb and creep arms differ (`Player_Collision` 0x194df0). Its +0x98 is
  cleared every tick (:88), so the 0x1ebaa8:301-319 drop masks never apply. Nothing in the path
  reads whether Bond is grounded.

So a flat bit-0x40 plate is touched whenever it is within 0.55 under B over its footprint, standing
or falling. The port's `seg_touches_tri_raw` already implements this refine; the new `Toucher::raw`
feeds it Bond's own collide segment (`Player::body`) for every `raw_plate` trigger, every stance
except the wire/zipline (the wire arm clears the pick mask, 0x194df0:308). `LOW_REACH`, the
directly-over test and the grounded gate are gone.

## What a player notices

- 07000006, the tunnel stair: plate #674 (ch 0x38 → OR #708 → sound #665) now fires from the
  42.4 step. The tuned reach missed it there.
- 07000006, the balcony drop into the searchlight courtyard: plate #664 (ch 0x34, the lamps' reveal)
  fires as Bond's segment reaches it, about five ticks earlier in the fall than before.
- The upper-door and tunnel routes keep what the first branch fixed (ch 0x46 breach, #524).
- The fall-death plates over the 05 and 08 valleys (05 #285/#293-#296, 08 #53/#55) and 011's atrium
  plate #314 can now fire a little earlier on a fall: they are flat bit-0x40 quads with terrain
  0.07-0.5 under parts of them [D floor census over the port's collision world].

## Tests

Red on the previous branch's tuned rule, green now (`arrival_plates_06_live`):
`the_balcony_drop_touches_664_at_the_segment_reach` (old: first contact at pos 46.46),
`the_top_tunnel_step_touches_674_and_not_524` (old: no ch 0x38). Sister:
`standing_on_the_balcony_leaves_664_alone`. The first branch's three tests stay green.

Green: walkreal 02, 05, 06, 07, 08, 0a, 0d, 11, 11_route, 12; interact and interactables 05/06;
searchlight_reveal_06, ss_operate_sound_06.

`walkreal_07000006_live::walk_the_castle_interior_to_the_party_real_damage_agent` is now ignored.
Its pass ended at 3.7 hp. With #664 firing five ticks sooner the reveal and the courtyard guards
shift, the walker runs out of ammunition at the mouth and dies at tick 17089. It is a walker
margin; the default-difficulty real-damage walk on the same route passes.

The walk harness's rise attribution (`common/walker.rs`) now also tests `Toucher::raw`, so a
raw-only contact names its plate.

## Audit of 05/06 beyond the first branch

- Trigger-graph census of both levels, logic inputs with no trigger writer. 05: 0x1d/0x1e/0x1f
  (the exits' gate/fallback), 0x6f (`Trigger_Calc` 0x1b1900's alarm latch), 0x70/0x71/0x73 (the
  truck chain), all with writers in docs/levels/07000005.md. 06: 0x39 (the fusebox #747), and
  0x30/0x31/0x1c, the oDrone death channels (param 12) of the courtyard balcony guards
  #446/#731, #448/#732, #719/#729. Open: 0x29/0x2a, the inputs of OR #706 (→ ch 0x13). The only
  placements carrying those values are explodeables #321/#322 (p0 0x29) and #69 (p0 0x2a), and
  the level doc reads that slot as the blast class; whether `Explode` writes it as a channel is
  not read yet.
- That last chain had no test: AND #709 → ch 0x2c (the spotlight plates' lock) → OR #752 →
  ch 0xb4 (the tunnel sound plates' lock). Walked in from the real writer (TouchOnce #531 → ch 0x12
  → spawners #730/#733/#734, one guard per pair), one kill per group raises 0x2c and 0xb4; two
  groups leave both off. `kill_chain_06_live`, green on arrival: coverage, not a fix.
- Objectives: 05's "Breach the castle walls" (reveal 0x45 by #36 on the boot tick, complete 0x46 on
  06 by every route) and 06's "Find a way inside to the party" (ch 0x02 through LoadPoint #746) are
  both on the real path (`fix_mission_live`, `arrival_plates_06_live`, walkreal 06).
- Found nothing else to fix. The first branch's open items (pane #376 not solid, the
  `ally::on_init_death` killer inversion off these levels) stand as written there.

## Scope left [S]

Bit-0x40 glists that are not flat keep the box (the 0x645 beams already run the raw refine through
`seg_beam`). The census of flat bit-0x40 trigger plates over the story levels: 02 #500/#501, 05 #36
and the plunge set #285-#296, 06 (17 plates), 07 #443, 08 #53/#55/#287, 0a #754, 0d #116, 011
#235/#256/#314-#318, 012 #788-#791. 08 #287's planes face down, so the raw arm never reaches it.

## Unproven

- Whether the game's segment is built from this tick's position or the previous one
  (`Player_Collision` runs inside the collide pass); a one-tick difference at most.
- None of the new contact points is a live capture; they come from the code path above.
