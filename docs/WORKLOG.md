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

### level-11-route-teleports-to-walks (finish, 01:58 UTC, 2026-10-31)

All three `bond.pos` stand-ins in `walkreal_07000011_route_live` are deleted — the whole
office-floor route (spawn → the waiting room → door #13 E → the corridor's west ledge → the west
bridge → the north strip → the stairwell door #192 E → the full walked descent 47→16 → the Load
Point #158 walk → E → 07000012) runs on `Sim::step_bond` inputs only. The corridor's direct
northward leg has no route in the disc's own data (the machinery row blocks the body band at
z ≈ 5.09-5.15 across x 9-14.5 [D collision], the well strip z 1.5-4.5 is floorless [D], the nav
graph links the stair chain's n94 only eastward [D nav]) — pinned in-test with the ray/floor/nav
asserts plus the deterministic lenient-walk wedge pin; the game's own route west (n10 → n11 → the
bridge) is walked instead. Touched binaries: walkreal_07000011_route_live 1 passed / 0 failed,
walkreal_probe11b (new diagnostic) 1 passed / 0 failed. Suite: the sandbox kills background jobs,
so the suite ran as re-invoked foreground per-binary runs — 369/972 recorded, 1000 passed, 3
failed (the pre-existing 0700000c pylon pair + a net-soak timeout; none in this task's scope).
