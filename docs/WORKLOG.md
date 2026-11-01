
## level-11-start-corridor-guard — 2026-11-01

Task: on a fresh 07000011 load with Bond standing at the start, the port has a red-suited guard
walk into the corridor beyond the unlocked door from about tick 200, hit Bond by tick 509 and kill
him by tick 3172. On the PS2 the same corridor stays empty over the same frames [L PCSX2 capture].
Find which drone this is (placement, group, wake channel, route), why it wakes or walks in the port
[M the wake/ENTER gates, Drone_Create 0x136be0's difficulty gate, the patrol/route class], fix the
gap with the address cited, and pin it: an idle Bond at the start is not approached in the first
3200 ticks (sister: the guard IS present in drones.list). Run every *07000011* binary and nf-game;
quote the totals.

Session 2 (2026-11-01, continued): sandbox reset, setup redone. Progress:
- Reproduced the symptom: on a fresh d3 load p127 (skin 0x0500003A, swch 0, mode 7) walks to Bond,
  drawn at ~tick 200, first hit ~417, dead ~700 (d1: p126 only; p127 exists from d2 — [D p2]).
- Ruled out the wake gate: [M 0x15b830:22-43] swch 0 → not WaitSwitch; [M 0x14b300:611-613] p1
  0x06000000 zeroes +0x554 — the game activates them into 0x85 exactly like the port.
- THE FIX (pushed ce3558f8): the ClosestAIPoint owner is Drone+0x144 = placement param 4 [M
  0x14b300:472]; the port read props bit 0x11. The disc pairs p126 (p4=31) ↔ AIPoint owner 31 at
  (1.40, 16.71) and p127 (p4=30) ↔ owner 30 at (−1.48, 18.21) [D]; after the fix the port's goals
  are exactly those points.
- New [M]: the game seeds +0x50c/+0x500 from prop 0x20 via DoModeSettingsNEW 0x14c600:60-68 (the
  table [D ACTION.ELF 0x2f5cc8] = 0, 0, 0.66, 1.0) — the port matches (p126/p127 alert 1.0);
  the Tower resume at alert 1.0 is ATTACK [M 0x168910:23-31] — the port matches; EnemyLookForOpponent's
  0.66/prop-0x51 gate is ported [M 0x177ed0:57-64]. No locked-door node pair sits between the
  guards' spawns and their points [D nav node flags] — the route is open on the disc graph.
- Remaining gap: with all of the above the port's guards still reach combat at ~7 units and kill
  Bond; the PS2 capture shows the corridor empty f200-f3172. Strongest open hypothesis: the game's
  per-cel object activation (control_handle_cel_change 0x133288 / the DynamicObjList membership)
  holds these guards' control until their cel activates — needs live evidence. Live checks listed
  in the report.
