# Placement kinds with no port arm

List every case of `parsemap_create_dynamic_objects` 0x1d04f0. For each kind, count placements per level and find the port's create arm. Report the table. Port any kind found on an on-foot level that has none, smallest first; queue-size ones go in the report's list with their counts.

Read the game code first, count on every level with `Level::parse` over `nf_view::dev::LEVELS`, and test on the real level, red on the old code. A count of zero on every level closes an item with that count as proof. Do not trust a port comment that says inert. Values measured at load can differ from the same global in play (FRAME_RATE reads 30 while a level builds, 60 after). Branch from origin/master, push to `s/placement-kind-census`, report in docs/reports/placement-kind-census.md. No driving levels. Draw work: leave a stub in the hand-off list and say so.
