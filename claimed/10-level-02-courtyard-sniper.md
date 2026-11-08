# 07000002: the courtyard sniper's hits on Bond

On the castle walk, crossing the courtyard at the stair bottom, the courtyard sniper lands hits of 13.7, 28 and 41.7 on Bond (Agent). Read the sniper's weapon record (damage, accuracy, difficulty scale, range falloff) and its fire rate in the decompile, measure the port's hit chance and damage per round at 15-25 u against what the game's code gives, and fix any difference. Pin the per-round damage and the hit rate at one range through real input.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-02-courtyard-sniper`, report in docs/reports/level-02-courtyard-sniper.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox5 at 2026-11-08T14:52Z
