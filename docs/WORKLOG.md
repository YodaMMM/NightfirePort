# WORKLOG

### level-0d-jib-walk-off (start, 05:26 UTC, 2026-10-31)

Task: `walkreal_0700000d_live::walk_arrival_to_the_c5_door` still holds a teleport. Finish the
deck -> yard chain on real input: (1) the deck-edge walk-off whose lateral offset lands inside
the walkway strip's width (or the roof-side edge, 18.58); (2) the dock walk + E #460 in-route;
(3) the beam boarding via the north steps and the roof/catwalk onto SP #115's westward pass,
the ride west and the drop into the yard. Context: level-0d-jib-rim-gap found the port's jump
and the script-entity collision posing faithful [M Player_HandleJump 0x1916b0;
Player_SSWalk 0x192310:69]; a mid-swing walk-off reached the strip's r band but landed
~0.13 m outside its width. If a leg needs a port fix, cite the class [M]. Run
walkreal_0700000d_live and interactables_0700000d_live and quote the totals.

### level-0d-jib-walk-off (finish, 06:43 UTC, 2026-10-31)

- The walkreal's deck→yard teleport is split: the DECK→WALKWAY WALK-OFF, the catwalk walk west
  and the dock E #460 are REAL INPUT now. The walk-off: at the swing's END pose the deck's rim
  on the strip bearing is open (the rest-pose rim stalls — probe17 v4), the pre-turn + aim at
  the strip point r 6.0/off +0.75 walks Bond off the rim and the run's carry lands him ON the
  static 12.13 strip (r 5.99/off +0.51, deterministic across runs). The strip is a DIAGONAL
  band (r 5.5-10, the lateral centre sliding +1.0 → −1.5 with r [probe20 rays]) — the old
  r 6.5-8/off-const model was why the rim-gap window missed by 0.13 m. bond.pos rides ~1.1
  above the surface. The jump stays vertical-only [M 0x1916b0; 0x192310:69] — no port fix
  needed anywhere: every physics suspect examined came out faithful.
- The catwalk: x 157..196 at 12.13, z −265.0..−266.25; south face wall z −266.26 (full length),
  west end wall ~153.5-157, the x 162-163 floor slit is ~2 cm (the capsule bridges it).
- E #460 real: ch 0x2b arms SP #115 [D #460 p0; M 0x1b2318:23-33; M 0x1dbda8 state-6 arm].
- Two NARROWED stand-ins remain, with the geometry pinned for the next window:
  (1) the catwalk→bay exit — the catwalk is sealed at walk height; the bay (9.0-9.5, x 168..194)
  sits 3.1 m below; the steps island (12.7 @ z −269) and the 14.7 pad (x 186..192.5, z
  −269..−273) are across the bottomless slot z −266.5..−267.5; the building's interior shows
  3.48/7.98 surfaces at x 186 [probe20 interior scan] — the game's exit is likely inside.
  (2) the beam boarding + west ride — the beam/trolley (0x0200_0a80/81, origin y 11.33, z
  −287.11, top 13.47-14.32) shuttle x-only 137.54 → 177.22 → 137.54 (y constant [probe20 beam
  watch]); the trolley's top (14.28) matches the pad but the shuttle's east limit never brings
  it under the pad; the 16.9 mezzanine (x 178..183, z −270..−283) and the roof (18.58) overhang
  the east-extreme position — the player's boarding climb (the roof's access) is the open leg.
- Totals on the final tree (merged with origin/master 7d0091a0): walkreal_0700000d_live 1
  passed / 0 failed (27.3 s, stuck 0, falls 1 = the walk-off); interactables_0700000d_live 11
  passed / 0 failed (30.6 s). Full suite restarted on the final tree (the total lands in the
  report; the window ends before it finishes — the partial + the touched binaries are quoted).
