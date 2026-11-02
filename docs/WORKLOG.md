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

Re-verified by raw-word decode this session (capstone misdecodes the ELF's 16-byte load/store
family as MSA/DSP; op 0x1F = the quad store, 0x1E = the quad load — prologue/epilogue save/restore
is symmetric: SQ $ra,0x100($sp) 0x136bfc / LQ $ra,0x100($sp) 0x136ccc). Drone_Create(VECTOR*
trans a0→$s1, VECTOR* rot a1→$s3, level_tag* a2→$s0): `LQ $v1, 0($s3)` 0x136c40 loads the caller's
rotation, `SQ $v1, 0x20($sp)` 0x136c50 parks it in the DIVars frame, `LQ $v1, 0x10($sp)` 0x136ca0
/ `SQ $v0, 0xc10($s0)` 0x136ca4 + `LQ $v1, 0x20($sp)` 0x136ca8 / `SQ $v1, 0xc20($s0)` 0x136cb0
(delay slot of the amemcpy jal) land trans/rot verbatim at D+0xc10/D+0xc20 — no float op touches
them (D = obj->0xe0; `sw $s1, 0xc00($s0)` 0x136c94 stores the obj back-pointer). PostLoad_Init
`lw $s1, 0xe0($s0); addiu $a0, $s1, 0xc00; jal 0x14b300` [M 0x14d8a8-0x14d8b4]; DefaultInit's
second call: `addiu $a1, $s2, 0x50; addiu $a2, $s2, 0x60; jal Vec_Copy2; addiu $a0, $s3, 0x20`
[M 0x14b350-0x14b35c]; Vec_Copy2 = three verbatim lwc1/swc1 pairs to BOTH dsts [M 0x1e2cf8].
Dispatcher prologue copies the placement row verbatim: `lwc1 $f11, 0xc($a0)`→`swc1 $f11, 0x30($sp)`
… `lwc1 $f8, 0x18($a0)`→`swc1 $f8, 0x40($sp)`, rot[1] = `lwc1 $f0, 0x1c($a0)`→`swc1 $f0, 0x44($sp)`
[M 0x1d0520-0x1d0564]; case 0xf: `move $a2, $s1; addiu $a0, $sp, 0x30; jal 0x136be0; addiu $a1,
$sp, 0x40` [M 0x1d0c30-0x1d0c40]. AI facing: DroneMove_NoBunching `lwc1 $f12, 0x54($s1)` then
+π/2 [M 0x150b44-0x150b58]. Port spawn: `Drone::new_with_stats(.., p.rotation[1], ..)` [M
crates/nf-game/src/drone.rs:11754]. Disk was full (100%) at session start; freed ~5.3G of stale
target/release deps executables + examples and re-ran the suite.

Verification runs (all green): the pin `drone_yaw_convention_live` 1 passed in 1.59 s;
corridor_guards_07000012_live 1; enemies_07000012_live 3; interactables_07000012_live 12;
stealth_ai_live 9; and all 22 walkreal level walks 01→4a (115 passed, 4 ignored, 0 failed —
0700001b 17, 07000016 14, 07000005 12, 0700000a 10+3 ign, 07000001 6 clean). Total 141 passed,
0 failed, 4 ignored. The ~1000-binary whole-suite build cannot fit the 9.9 GB sandbox disk
(two attempts died on rustc "No space left on device"; freed ~5.3 GB of stale executables
twice) — ran the task's named groups as `--test <name>` foreground slices, deleting each
slice's executables before the next. Report: docs/reports/drone-yaw-convention.md.
