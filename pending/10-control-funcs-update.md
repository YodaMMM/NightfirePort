# control_funcs update column against the port

`control_funcs` at 0x26f890, 12 bytes per class. For every class with a non-null update: which levels create it, and which port function ticks it? A class with objects on an on-foot level and no port tick is a gap. Port the small ones, list the rest with counts.

Read the game code first, count on every level with `Level::parse` over `nf_view::dev::LEVELS`, and test on the real level, red on the old code. A count of zero on every level closes an item with that count as proof. Do not trust a port comment that says inert. Values measured at load can differ from the same global in play (FRAME_RATE reads 30 while a level builds, 60 after). Branch from origin/master, push to `s/control-funcs-update`, report in docs/reports/control-funcs-update.md. No driving levels. Draw work: leave a stub in the hand-off list and say so.
