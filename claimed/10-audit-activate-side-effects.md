# audit-activate-side-effects — every object's Activate and Deactivate against the port

A real gap sat unnoticed for weeks: GunImp_Activate 0x132f00 calls Camera_CalcViewAngles(30 degrees) and the port never did, so the mounted view was twice too wide. The sim tests all passed because nothing compared the side effects.

Walk every `*_Activate`, `*_Deactivate`, `*_Create` and `*_Delete` in the decompile for on-foot objects (monitors, switches, turrets, locks, Q-worm, ladders, lifts, doors, pickups, cameras). For each call in the body (camera, HUD, input, weapon, sound, player state) find the port's counterpart. Report a table: game call, address, port site or MISSING. Fix each MISSING a player can see or hear, with the address and a real-input test. Skip driving. Done = the table in the report, fixes tested, every walkreal_*_live green, the full suite total line.

---
claimed-by: sandbox3 at 2026-11-04T18:12Z
