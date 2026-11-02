# drone-yaw-convention — settle which way placed guards face

Branch from origin/master (it carries corridor_guards_07000012_live, the log-only probe from level-12-corridor-guards-stuck).

The port spawns every drone facing `p.rotation[1]` raw (drone.rs ~11754, the placement row's +0x1c f32). On 07000012 that faces the two corridor guards (placements 12/13 at (13.09, 18.55) / (12.98, 16.43)) away from Bond's spawn, so their sight cone never scores him and they never come down the corridor. `Drone_Create` 0x136be0 receives the rotation vector [M 0x1d04f0 case 0xf] but the decompile text drops which component lands in the drone's facing.

Do:
1. Disassemble Drone_Create 0x136be0 and whatever it calls that writes the drone's facing (work/elf ACTION.ELF): name the exact component and sign the facing takes from the rotation vector [M with addresses].
2. If the port differs (sign or component), fix it at the one place drones take their spawn yaw, and check every level's guard-facing tests and walks (the change touches every placed guard). If the port matches, say so with the disassembly lines and leave the code.
3. Either way, add a test pinning the facing law with the cited instruction addresses and a sister that fails on the opposite sign. Turn corridor_guards_07000012_live into a real pin if the fix makes the corridor guards wake on the player's approach.
4. Run every enemies_*/walkreal_*/interactables_* live test, stealth tests, then the full suite.

---
claimed-by: sandbox1 at 2026-11-02T09:30Z

---
## Report (done, 2026-11-02T12:43Z)

