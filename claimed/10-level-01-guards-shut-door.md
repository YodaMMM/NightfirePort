# 07000001: alerted guards and a shut door

In the hall before the south door, alerted guards that heard Bond's shots stay 3-7 u back and side-step, and never open the shut door to reach him. Read the drone's investigate/chase states and its door use (NDrone2 door open calls, LinkCreep_Handler 0x15a2c8, the hearing alert) and find whether the game's guards open a shut door to reach a heard shot. Reproduce through real input from the hall's south side, compare, fix if the port differs.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-01-guards-shut-door`, report in docs/reports/level-01-guards-shut-door.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox7 at 2026-11-08T14:50Z
