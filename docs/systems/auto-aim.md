# Bond's auto-aim

How ACTION.ELF bends Bond's shots onto a target, and how the port follows it. Tags: [M] decompile or
disassembly, [D] disc/ELF data, [L] live capture, [S] the port's own choice.

## Call order per tick [M]

`Player_Update` 0x193600 calls `Player_Weapon` 0x1a24c8 (jal 0x194064), whose last call is
`Player_AutoAim` 0x1a7c98, and then `Player_WeaponFiring` 0x1a5460 (jal 0x194070). A shot fired this
tick therefore reads the offset `Player_AutoAim` left this tick. `Player_GetAimingPoint` 0x1a4a98
(called from `Player_WeaponFiring` and `Player_WeaponInitBullet` 0x1a4bf0) turns it into the aim point.

## The state (BLData) [M]

| Field | Meaning |
|---|---|
| +0x114 | the lock pointer (the last target object) |
| +0x118 / +0x11c | the aim-mode cursor (the port's `Sim::cursor`) |
| +0x120 / +0x124 | the crosshair offset, in screen units: x right, y up, ±1 at the screen edges. `HUD_UpdateCrossHair` 0x19e6d0:87-92 draws the crosshair at (W/2·(1 + x), H/2·(1 − y)). The port's `Drones::aim_offset`. |

## `Player_AutoAim` 0x1a7c98 [M disasm 0x1a7c98-0x1a7ef0]

1. The option byte: `PlayerSetting[p]+1` in single player, `+2` in multiplayer (`MPSettings+0x180`).
   The Q-Claw (0x50/0x51) forces it to 1 (`movz` at 0x1a7d14).
2. The aim bit (`obj+0xdc → +0x60` bit 0) is the gate. `Player_Weapon` 0x1a24c8:46-61 writes it
   from the aim button, clears it through the raise states, and `Player_SetWeaponAnimObj`
   0x1a2e38:557 sets it when the raise ends.
   - Bit set: +0x120 += (+0x118 − +0x120)·0.5 and the same for y. No scan, the lock stays.
   - Bit clear and the option on: the scan (below), then all four offsets halve.
   - Bit clear and the option off: all four offsets halve. The lock stays.
3. The scan's cap starts at `Autoaim_Range` (25.0) or, for the Q-Claw, at the held record's +0x1c
   (50 / 100 [L ee_start_05]). The Q-Claw walks the class-'Q' objects (`control_first_object`
   chain); every other weapon walks the drone list `DAT_00298e14`, then `glb_players[0..3]`.
4. The release: after the scan, if the cap still equals `Autoaim_Range`, +0x114 = 0. A Q-Claw scan
   starts above 25, so it never releases the lock.
5. `Check_Target` 0x1a7810 runs last.

## `Check_AutoAim` 0x1a73f8 [M disasm 0x1a73f8-0x1a780c]

- Skips: no object, `obj+0xfc` (drawn) 0, class 0x12. Class 2 (drones): health +0xac ≤ 0, voice set
  +0xd8 = 0xb, `obj+0xf4` = 1, and in single player team +0x44 ≠ 1 (MP: `MP_areObjectsOnSameTeam`).
  Class 3 (players): same team, the same viewer, health +0x894 ≤ 0.
- The point: classes 2/3 use bone 2 (`AnimGetBoneWorldTrans(obj, 0x80000002)`), others `obj+0x30`.
  The head is `Player_GetHeadPos` 0x195aa0.
- dist = `Vec_Dist3D` 0x1e2e48 (VU0 sub and squares, `add.s`, `sqrt.s`). If cap < dist: skip (so an
  equal distance later in the walk wins).
- The yaw test: `Vec_AngleDifference`(`atan2f`(dx, dz), obj+0x54).
- k = ((w+0x14 · 0.01) · (1 − dist/`Autoaim_Range`)) · the difficulty mul (GameState+0x28: 1 Easy
  1.9, 3/4 Hard 0, else Normal 1). The Q-Claw forces k = 1.
- V = `Autoaim_Angle_V` (0.22)·k, H = `Autoaim_Angle_H` (0.12)·k, both ×`Autoaim_LockOnMul` (1.4)
  when the object is the lock. Skipped when both are 0, so Hard has no gun auto-aim.
- |yaw diff| < H, then |`Vec_AngleDifference`(`asinf`(dy/dist), BL+0x8a8·π/2)| < V, then
  `Collide_LineOfSight` (mask 0xa) from the head.
- A pass: if `View_3DPoint2Screen` 0x1e7708 returns > 0 (the point is not behind the eye),
  +0x120 = (sx − W/2)/(W/4), +0x124 = (sy − H/2)/(H/4) and +0x114 = the object. Either way the cap
  takes dist. The stored value is twice the target's screen position; the halving in
  `Player_AutoAim` makes it exactly the screen position, so the crosshair sits on the target.

## `View_3DPoint2Screen` 0x1e7708 [M disasm]

v = the viewer matrix (+0x160/+0x170/+0x180) · (p − eye +0xc0). If v.z < 0 it negates z and returns
−1. sx = W − (v.x·(fx/z) + W·0.5), sy = v.y·(fy/z) + H·0.5, with fx/fy = viewer +0x100/+0x104 from
`Camera_CalcViewAngles` 0x1baf70 (60°, aspect 4/3, the `MAX_VIEW_CONE` 2000 factor). The zoom
(+0x108) is not used here.

## `Player_GetAimingPoint` 0x1a4a98 [M]

With BL+0x950 clear: bx = ((W·x)/zoom)/(fx + fx), by = −((H·y)/zoom)/(fy + fy), zoom = viewer
+0x108. The aim point is head + (−256)·M·(bx, by, −1) with M = `obj+0x90` · RotMatrixX(−pitch).
W/(2fx) = 0.76966 and H/(2fy) = 0.57724, so the ray runs through the crosshair, divided by the zoom.

## `asinf` 0x232d48 [M disasm 0x232d48-0x232e40, 0x233658-0x2339f0]

fdlibm's float `asinf` with the `w_asinf` wrapper. `__fdlib_version` 0x30ba00 is 1 [D, L], so for
|x| > 1 the wrapper's domain branch returns 0.0 (not NaN, not ±π/2). Port: `trig::asinf_libm`,
op for op on the EE models; the root is `__ieee754_sqrtf` 0x2345c8, integer code, correctly rounded.

## The port

- `nf_game::autoaim`: `focal`, `view_basis`, `project_offset` (View_3DPoint2Screen and the store),
  `aim_dir` (Player_GetAimingPoint), `dist3d` (Vec_Dist3D).
- `Drones::auto_aim_view` (drone.rs): the scan arm. `Drones::auto_aim` is a wrapper for probes that
  only hold a direction.
- `Sim::run`'s dir pick (scene.rs): the three arms of step 2, once per tick, on the aim bit.

### Unproven or not ported

- The view frame: the port builds the projection frame and the aim frame from Bond's yaw and view
  pitch with the port's own sin/cos. The game's two matrices are not checked bit for bit [S].
- The `tanf` in `focal` is the host's; the libm `tanf` chain is not ported [S].
- The drawn crosshair keeps the port's earlier position (about half way to a fresh lock). The game
  draws +0x120 as is, so its crosshair sits on the target. That is a visual change and needs a look
  before it moves.
- The drone walk order is the port's list order, not the `DAT_00298e14` chain. It only matters for
  two targets at exactly the same distance.
- Grapple points: the drawn gate (`obj+0xfc`) is not checked and the point is the placement plus the
  sphere centre, not `obj+0x30`.
- MP: the players walk and `MP_areObjectsOnSameTeam` are not modelled here.
- Zero-g: the view frame comes from yaw/pitch, not the zero-g rotation.
