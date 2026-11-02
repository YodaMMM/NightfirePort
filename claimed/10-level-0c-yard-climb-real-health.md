# level-0c-yard-climb-real-health — the yard climb fails on the real-health run

Branch from origin/master.

On walkreal_0700000c_live's real-health run (ignored) the yard climb "never mantled past climb_top 12.139 [M 0x196118/0x196310]" at (9.95, 7.95, -186.93) after Bond took 80 damage; the top-up walk mantles there.

Do:
1. Find whether taking damage (pain reaction, flinch, speed change, knock-back) stops the mantle in the port and whether the game's mantle has the same gate [M Player climb code 0x196118/0x196310 and the pain path].
2. Fix the port where it differs; cite it. If the game also blocks the mantle while hurt, route the walk to the climb after the fight.
3. Run walkreal_0700000c_live (both variants), every climb/mantle test, then the full suite.

---
claimed-by: sandbox4 at 2026-11-02T16:12Z
