The game hard-codes per-level behaviour: 127 comparisons of `GameState._12_4_` against a level code in the decompile. One unported case was found by accident: jump refused on 07000007 before channel 0x48, now on master. Others are likely.

For every branch in your set, read it, trace what it does, and find the port's code. Report one line each: PORTED with the port location, or MISSING. A comment describing the branch does not count as ported; look for the code. Fix each MISSING one with a test on the real level, proved red on the old code. Do not rely on hand-built geometry.

Your set: every `Drone*`, `NDrone2_*`, `DroneFunc_*`, `DroneVision_*` function except the talk functions (`*Talk*`).

---
claimed-by: sandbox3 at 2026-11-06T17:17Z
