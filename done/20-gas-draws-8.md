# Gas draws: the Drake pair

Branch from `origin/s/gas-rng-draws-7`, merge current `origin/master`, push to `s/gas-rng-draws-8`. Keep every gas draw. Read docs/reports/gas-rng-draws-7.md on that branch first.

1. Check Drake's hover against the decompile before touching the walk: on the merged stream he rises from (-7.3, -1.4, 0.1) to x 9-17.8, y 4.2-7.6 in states 0xc0/0xc1. Find what drives that height in the game and say whether the port's band is right. If it is wrong, fix the game code and cite it.
2. If the hover is right, fix `walk_arrival_to_drakes_defeat` and its real-health sister: fight him during the rise (t0-360; `last_tap = 0` blocks the first 12 ticks of that window) or add a pursuit that climbs with him. A static hold cannot work, the report's scan shows that. Lifting the level-look fire gate for this fight is fine, a player aims up.
3. The 16 chasers pair and the 01 d3 walk are also red here. `10-guard-lines-finish` works on them, so merge `s/guard-lines-finish` if it has landed, and do not redo that work.

Quote walkreal 1b, 01 and 16 whole.

---
claimed-by: sandbox3 at 2026-11-06T07:50Z

---
## Report (done, 2026-11-06T08:52Z)

