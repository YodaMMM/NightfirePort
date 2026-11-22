# s/pullback-cels: the guard route pull-back on cels

Moves the shared random stream (guards on fallback-goal combat routes can stop at a different point).

## What changed

`AINetwork_AlterDestFor_DROUTE_Nearest` 0x156088 (crates/nf-game/src/nav.rs `Route::alter_dest_nearest`):

1. After moving the destination the game re-finds its cel (`route+0x50 = NDrone2_FindCel(route+0x40)`) and sets
   route flag 0x40 [M 0x156088:47-52]. Both are ported (`Route::dest_cel`, `Route::flags`). The flag has no reader:
   a disassembly scan of every byte/half/word load at route offset 0, 0x860 (combat route) and 0x970 (mission
   route) followed by an `andi` keeping 0x40 finds only clears (`AINetwork_InvalidateRoute`, `NDrone2_InitAIPath`)
   and two cover-node reads. Setup clears it (`& 0xffbf`, 0x158948).
2. `NDrone2_MoveTestFromNode` 0x1554a8 hands `AINetwork_BoundsTest` 0x157d80 the node's cel and the destination's
   cel. The new `Network::bounds_test_cels` follows the disassembly: no A cel answers 0; `Collide_StraddleCels`
   walks the ray A+1 to B+1 from A's cel and B's cel must be reached, else 0; then only the reached cels' own wall
   lists are tested (newest first). A wall filed on a cel the ray never reaches no longer pulls the destination
   back. Used by the pull-back only; the other move tests keep the flat all-walls test.
3. BoundsTest's 7th argument (the out flag that would zero MoveTestFromNode's result): BoundsTest never reads t2
   (no instruction in 0x157d80-0x158138 names $10, no callee is handed it), and the caller clears the byte in the
   jal delay slot, so the zeroing can never fire. Nothing to port; recorded in the doc comment.

## Tests

- `nav::tests::the_pull_back_tests_only_the_reached_cels_walls` (new): own-cel wall pulls back (cel re-found, 0x40
  set); the same wall filed on an unreached cel does not; a straddled portal brings it back; a destination outside
  every room cel, or a node with no cel, answers 0. Fails on the old flat test (the unreached-cel case pulled back).
- nav unit tests 14/14, nav_kick_live 5/5, enemy_accuracy_live 5/5, walkreal 01 7/7, 02 5/5, 0d 2/2, 14 9/9, 15 9/9 (ignored tests unchanged).

## Unproven

- The destination's starting cel is `build_FindCel` of the destination; the game uses the target object's linked
  cel (obj+0x20). They can differ at a cel border.
