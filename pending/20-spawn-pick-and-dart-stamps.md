# Spawner pick and dart stamps

Two divergences found by reading drone AI against the decompile. Each needs a fail-first test.

1. Spawn pick: the game draws `Rand_Rand(total templates)` and spawns nothing when the draw's template is used or too close [M 0x137cb0:21, 0x137aa0:32]. The port draws `rand(usable.len())` (spawners.rs:194) and always spawns, so spawner waves arrive at the wrong rate. The port also makes none of the game's spawn-chain draws [M 0x14b300:382-461, 0x137cb0:58]. Port both, then compare a wave's spawn frames with a live PCSX2 run on 0700001b.

2. Darts on the Tower levels: `on_dart_impact` skips the stamp block the tase runs [M 0x1469b8:322-348 against :89-128]. Port it and pin it.

Run every live binary that touches spawners. Re-pin walks by route, never by removing draws.
