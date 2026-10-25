
### level-14-empty-panels (started 07:23 UTC, 2026-10-25)

Task: 07000014 openings and panels with nothing behind them — at the two panel spots (vent/hatch
54.21,1.03,46.15; wall panel -88.58,-1.10,46.15) dump what the port draws and collides and what
the level data places there [D], find the missing piece (undrawn object, missing cel, LOD/variant
not swapped, script entity not created) [M for the creator], port it, pin draw lists + collision
with tests, name headless renders in the report.

Plan: branch from origin/master 38f1bc93; start the fast suite in the background early; census
the placements around both spots; find the creators; port what's missing; touched tests + suite;
report.
