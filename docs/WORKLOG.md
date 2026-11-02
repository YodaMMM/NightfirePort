
## level-0c-yard-climb-real-health (started 2026-11-02 16:48 UTC)

Branch cut from origin/master edd5748b. Task: walkreal_0700000c_live's real-health run (ignored)
fails the yard climb — "never mantled past climb_top 12.139 [M 0x196118/0x196310]" at
(9.95, 7.95, -186.93) after Bond took 80 damage; the top-up walk mantles there. Plan: (1) find
whether damage (pain/flinch/knock-back/speed) gates the port's mantle and whether the game's
climb has the same gate [M 0x196118/0x196310 + the pain path], (2) fix the port where it differs
(cited) or route the walk to the climb after the fight if the game blocks it too, (3) run
walkreal_0700000c_live both variants, every climb/mantle test, the full suite.
