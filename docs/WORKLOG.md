# WORKLOG

### level-11-route-teleports-to-walks (start, 00:42 UTC, 2026-10-31)

Task: `walkreal_07000011_route_live.rs` still moves Bond with three `bond.pos` writes (around
lines 51, 65, 74) — walk stand-ins. For each: read its comment (why it was a stand-in), find
the game's own route there [D collision/placements/nav; M any class the route uses: doors,
lifts, ladders, wires, movers], walk it on real input, and delete the write. Where the data
shows no route, keep the write but pin the gap with a sister assert and say exactly what live
check would settle it. Test-setup placements whose subject is not the route (a measurement
spot) may stay — say which and why. Every test in the file green. Touched-binary totals and
the full-suite total line to quote.
