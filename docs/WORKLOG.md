
## spawn-pick-and-dart-stamps (2026-11-06, s/spawn-pick-and-dart-stamps)

Task: port two drone-AI divergences found reading the decompile, each with a fail-first test:
1. The spawner pick: the game draws Rand_Rand over ALL templates and spawns nothing when the
   drawn template is used or too close [M 0x137cb0:21, 0x137aa0:32]; the port draws
   rand(usable.len()) and always spawns (spawners.rs:194). The port also makes none of the
   game's spawn-chain draws [M 0x14b300:382-461, 0x137cb0:58]. Port both; live PCSX2 wave-frame
   comparison on 0700001b goes to the report as a live check.
2. Darts on the Tower levels: on_dart_impact skips the stamp block the tase runs
   [M 0x1469b8:322-348 vs :89-128]. Port and pin.

Cut from origin/master c4c00c7ec (same tree the civ-unignore tasks ran). Plan: read the four
decompile sites + the port's spawners.rs/drone.rs dart path, write the fail-first tests, port,
then re-pin any walk the new draws shift BY ROUTE (never by removing draws), and run every live
binary that touches spawners.
