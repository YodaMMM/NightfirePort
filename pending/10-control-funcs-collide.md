# control_funcs collide column against the port

Same table, the collide slot. For every class with a non-null collide handler find the port's contact path: bullets, blasts, player, drones. The Destroy handler has quirks that only reading it shows (a timed leaf negates twice and never fires). Check each handler's branches, not just that one exists.

Read the game code first, count on every level with `Level::parse` over `nf_view::dev::LEVELS`, and test on the real level, red on the old code. A count of zero on every level closes an item with that count as proof. Do not trust a port comment that says inert. Values measured at load can differ from the same global in play (FRAME_RATE reads 30 while a level builds, 60 after). Branch from origin/master, push to `s/control-funcs-collide`, report in docs/reports/control-funcs-collide.md. No driving levels. Draw work: leave a stub in the hand-off list and say so.
