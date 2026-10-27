
## level-14-vent-grates (2026-10-27, this window)
Branch from origin/master. The 07000014 vent grates draw hanging inside the ducts, off their
frames (viewed from (32.61, 2.80, 45.27) and (56.03, 0.65, 46.89)). This window: identify the
grate objects [D placements], find the game's pose rule (placement matrix / open state / door or
SimpleScript mover / hinge pivot [M create/update + the draw matrix]), port it, test against the
[D]/[M] pose.

## level-14-vent-grates (2026-10-27, this window) — the pose rule found and ported
The duct-mouth grates are the 0xd9 ScriptPlayer placements of object 0x02000ab4
"Vent_StoreRoom_Graphic" (nine on 07000014; the driver's two: #1173-family pl 848
(30.10, 3.58, 46.42) rot (−π/2, −π/2, 0) and pl 849 (51.58, 3.58, 44.94) rot (−π/2, π/2, 0))
[D placements probe]. The game's pose rule [M]: parsemap_create_dynamic_objects 0x1d04f0 case
0xd9 → SP_CreateScriptPlayer 0x1db9a8 → control_create_object 0x1344e0, whose matrix with
param_4 == 0 is View_RotTransMatrix 0x1e5168(rot triple, pos) — NOT RotMatrix 0x1e2368 (the
script itself, 0x06000a3b, keys IDENTITY and creates the LID 0x02000750 at t0 + the ladder
0x02000751 at t61 — the grate's pose IS the placement matrix). The decompile's PS2Sinf3 output
slots were scrambled; the disassembly (jal 0x11fc20, outputs sp+0/4/8 = (cx,sx,cy),
sp+0xc/0x10/0x14 = (cz,sz,sy)) pins the rows — ported as placement::view_rot_trans_matrix
(bit-exact discipline, the pf helpers). Used in Player::from_placement [M] and the static
draw's 0xd9 family (level.rs instances). Verified: pl 848's panel now spans x 30.12,
z 45.68..47.16, y 2.14..3.58 — FLUSH in the west duct-mouth wall (was 1.44 units proud,
x 30.10..31.54); pl 849 likewise; every yaw-only placement bit-identical (the compositions
collapse to the same rows at rx = rz = 0). nf-formats: 94+ green, the 3 allowed disc_golden
fails. The render check belongs to the main agent.
