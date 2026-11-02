Branch cut from origin/master c628a2fa (2026-11-02 09:35 UTC). WORKLOG started fresh (master tracks no docs/WORKLOG.md; recreated per prior-task convention).

## Task
drone-yaw-convention: settle which way placed guards face. The port spawns every drone facing `p.rotation[1]` raw (drone.rs ~11754, the placement row's +0x1c f32). On 07000012 that faces the two corridor guards (placements 12/13) away from Bond's spawn, so their sight cone never scores him. Disassemble Drone_Create 0x136be0 and whatever writes the drone's facing from the rotation vector; name the exact component and sign [M with addresses]. Fix the port if it differs; pin the law with a test + an opposite-sign sister; turn corridor_guards_07000012_live into a real pin if the guards wake.

## drone-yaw-convention (2026-11-02)

The facing law is a verbatim float copy end to end [M ACTION.ELF]: parsemap_create_dynamic_objects
case 0xf passes the placement's translation/rotation vectors (0x1d0c30-0x1d0c40, the dispatcher's
stack copies of +0x0C/+0x18 [M 0x1d04f0:50]); Drone_Create 0x136be0 zeroes the DIVars (memset
0xc0), amemcpys the params (level_tag+0x2c, 0x84) to Drone+0xc34 (0x136cac) and stores the rot
vector into DIVars+0x20 = Drone+0xc20 (the EE SQ at 0x136cb0); NDrone2_PostLoad_Init 0x14d890
hands Drone+0xc00 to NDrone2_DefaultInit 0x14b300 whose Vec_Copy2 0x1e2cf8 (three lwc1/swc1 pairs)
copies DIVars+0x20 to obj+0x50 AND obj+0x60 (0x14b350-0x14b35c); the AI facing reads obj+0x54
(0x150ad0:27-35). Facing = placement rotation[1] raw — the port's law MATCHES; no code change.
The 07000012 corridor guards face away by the disc's own authoring (guard yaw -1.89/-1.38 vs
yaw_to_opp +1.50/+1.29; guard #6 is already ALERT 0.66 at load — the wake is the alarm/hearing
path). Pin test: crates/nf-view/tests/drone_yaw_convention_live.rs (parses the level's rows from
the ISO, matches sim drones by position, asserts yaw == rotation[1] and NOT -rotation[1]).
