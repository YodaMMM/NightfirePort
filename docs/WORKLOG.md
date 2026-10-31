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
