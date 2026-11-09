# 0700000c: guard #1's rounds at the ring

On the current tree 0c guard #1 keeps firing at the ring where he used to run dry. Read the guard's clip/reload/ammo rules for his weapon (drone weapon record, reload state, whether guards have finite ammo) in the decompile and pin his rounds over a fight through real input; fix the port if he fires more or less than the game allows.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-0c-guard-ammo`, report in docs/reports/level-0c-guard-ammo.md. No draw changes: name what is wrong if it is a draw issue.
