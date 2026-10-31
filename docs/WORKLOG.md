# WORKLOG

### level-0c-trench-exit (start, 03:05 UTC, 2026-10-31)

Task: `walkreal_0700000c_live` still carries the trench-exit `bond.pos` write
`[5.0, 12.35, -112.9]`. The glass lintel is at y 13.36 so the crouch capsule (top ~13.16
[M Player_Collision 0x194df0 case 3]) should clear it, yet the crouch exit stalls at
z -111.81 (grounded false) while every ray at y >= 11.9 there comes back clear — so the
block is below ray height or a port-side push-out on the case-floor seam. Find it [D the
cels under z -112.0..-111.6; M the capsule contact path]; and check whether the game's vent
drop lands inside the case (~7.3) rather than in the trench (the port's drop) [D the
vent/duct collision; M the drop]. Fix the port gap with evidence and walk the exit (delete
the write).

### level-0c-trench-exit (finish, 04:41 UTC, 2026-10-31)

The trench-exit stall PINNED, no port gap: the shaft's south crawl pocket (z −111.89..−112.34,
ceiling 13.34 [D]) is sealed by the disc's own wall slab z −112.34..−112.363 (outer face y
11.62..13.57 [D block dump]); the crouch capsule (r 0.55 [M 0x194df0 case 3]) stops ON it at
−111.813 = face + r — the observed −111.81; the body-height rays cannot see the slab (no north
face at capsule height; front-face rays [M 0x10a8c8:168]) — the game holds the body too
[L 2026-10-26 hatch-3]; no 0x20 breakables on 0c [D census] — no glass exit. The old "duct
delivers into the case ~7.3" comment REFUTED (hatch-1's unmapped-leg question, not a finding;
[L hatch-2] the drop rests on the bay floor 11.84 — the port's drop matches). The REAL crouch
walk into the pocket + the deterministic stall pin replaces the "UNPINNED" note; the stand-in
stays, documented as crossing the disc's own south seal. Touched binaries:
walkreal_0700000c_live 2 passed / 0 failed, interactables_0700000c_live 5 passed / 0 failed,
plus the walkreal sweep 07000001..07000011 (13 binaries, all green — see the report; 12 more
walkreal binaries + the suite total line remain for the next session; this sandbox kills every
background job between tool calls, so the fast runner never survived — the report has the
foreground re-invocation recipe). Report: docs/reports/level-0c-trench-exit.md.
