# 07000005: the truck exit and the courtyard guard

The level exit by truck now waits while a guard aware of Bond is within 30 u (ch 0x98, the game's EndLevel event 0x11 [D]). Walk the real route to the truck with real input on each difficulty and check whether the courtyard guard notices Bond there; if he always does, find what the game's route or the guard's rules (alert decay, sight, the 30 u test) do differently so a player can leave, and fix the port or document the player's way.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-05-truck-exit`, report in docs/reports/level-05-truck-exit.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox1 at 2026-11-09T16:30Z
