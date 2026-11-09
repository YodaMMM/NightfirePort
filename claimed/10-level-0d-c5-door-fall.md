# 0700000d: the drop on the way to the C5 door

walk_arrival_to_the_c5_door_real_damage_agent (walkreal_0700000d_live, ignored) fails 'the ~10 m fall is survivable [M 0x1949a8], got 0'. The fall rule was re-read and matches (see the 0d fall-damage work merged in master). Measure the actual drop height and landing speed on this route, compare with what the rule gives, and find whether the route falls further than intended or the port's landing differs. Fix the port or the route; un-ignore when it passes.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-0d-c5-door-fall`, report in docs/reports/level-0d-c5-door-fall.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox1 at 2026-11-09T02:15Z
