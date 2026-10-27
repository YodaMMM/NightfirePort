## level-4a-ride-cast-b (2026-10-27)

Branch s/level-4a-ride-cast-b from origin/s/level-4a-ride-cast (the plan-only session;
master base ec917093). Task: find the mechanism that places the 0700004a ride landing
cut-scene's cast (pack 0x060006cf, SP #289) in the car despite the car-local sequence roots
(0x040005fd root (2.35, 0.85, -0.64) [D]) — Script_AnimStart 0x1d8f40 creates the actor at
the origin and Script_Interp 0x1d9a08 case 2 composes no base [M] — port it exactly, keep
every other cut-scene's cast where it is.
