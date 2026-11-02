Branch cut from origin/master c628a2fa (2026-11-02 09:35 UTC). WORKLOG started fresh (master tracks no docs/WORKLOG.md; recreated per prior-task convention).

## Task
drone-yaw-convention: settle which way placed guards face. The port spawns every drone facing `p.rotation[1]` raw (drone.rs ~11754, the placement row's +0x1c f32). On 07000012 that faces the two corridor guards (placements 12/13) away from Bond's spawn, so their sight cone never scores him. Disassemble Drone_Create 0x136be0 and whatever writes the drone's facing from the rotation vector; name the exact component and sign [M with addresses]. Fix the port if it differs; pin the law with a test + an opposite-sign sister; turn corridor_guards_07000012_live into a real pin if the guards wake.
