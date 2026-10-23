# The player camera trio: orbit, underwater shear, weapon swing

The per-frame camera pipeline the game runs for a viewer: `Camera_UpdateAll` 0x1bbd68
loops `glb_viewer[0..10]` → `Camera_Update` 0x1bb6d8 calls the viewer's updator fnptr
(viewer+0x68, set by `Camera_SetUpdator` 0x1bc370 — on-foot it is
`Player_PositionCamera` 0x18f5d8), then shears when the eye is under water, adds shake,
and finalizes the matrices. This doc covers the three pieces ported here; the dead
modes 3/4/5 and the rest of the pipeline are in docs/port-map/unreached.md and
docs/systems/uncited-audit.md.

## Camera_Orbit 0x1bb828 (cam_mode 1 — the GoldenEye-strike orbit) — ported

The only mode-1 writer is `MP_GoldenEyeUpdate` 0x187568:396-398 →
`Player_SetCamMode(BL, 1)` 0x195928 (case 1 inits the CAM_ORBIT at BLData+0xe0..0x104:
max 1.5, radius 0.15, target 1.5, phase 0 [M 0x195928:17-24]) [M]. Each frame
`Player_PositionCamera` case 1 [M 0x18f5d8:135-144] sets zoom = 1.0 (viewer+0x108),
refreshes the look from the head bone (`AnimGetBoneWorldTrans(obj, −5, 0, BLData+0xe0, 0)`)
and calls `Camera_Orbit(viewer, BLData+0xe0)` [M 0x1bb828]: eye = look + (sin(φ)·r,
look.y + (0.75 when r < 0.75 else min(r, max)), cos(φ)·r) with the pre-advance phase and
radius; then φ += (1/60)·0.7; the eye sphere (`Collide_SphereIntersect(1.5, viewer+0x150, …)`)
pulls the target to the closest hit floored at 0.1 (or the max when clear); the radius
eases (target − radius)·(1/60)·5 without crossing. Port: `Bond::orbit` (the CAM_ORBIT
state), `Bond::set_cam_mode_orbit` (the SetCamMode case-1 arm), `Bond::orbit_camera`
(the math; `sphere_hit` param = the eye sphere's closest hit), `Sim::orbit_camera`
(the per-frame arm: head-look refresh + the port's world-sphere query). Tests:
`crates/nf-view/tests/camera_menus3.rs` (9 tests, numbers only).

## Camera_Shear 0x1bba38 (underwater wobble) — ported

`Camera_Update` 0x1bb6d8:25-34 runs it whenever the eye Y (viewer+0xc4) sits below the
eye cel's water surface (cel+0x90) [M]. Per frame: xs = 0.025·sin(φ+π/2), ys =
0.025·sin(φ) [M 0x1bba38:20-24]; φ (viewer+0x1e0) += 1/60, wrapped into [−π, π] [M :26-34];
an identity+skew matrix (scale 1+xs, 1+ys, 1+xs) concats into the view matrix and
`GlobalBlur = 0x91` [M :36-49]. Port: `scene::camera_shear_step` (the pure step),
`Sim::camera_shear` (sets the blur byte), `Sim.shear_phase` / `Sim.global_blur` (viewer
state), wired in the Sim tick keyed on the port's cel+0x90 comparison
(`Bond::underwater`). The skew matrix concat and the blur render are the draw side's: `global_blur != 0` marks the underwater frame, `shear_phase` is the
oscillator, `(xs, ys)` the per-frame skew factors.

## Camera_ApplySwing 0x1bbf40 (weapon-aim sway) — ported

`Player_Update` 0x193600:439-441 calls it every tick with the current weapon's swing
amount `weapon_data[weapon·0x10c + 0x8c]` (`lwc1 f12,140(a1)` [M 0x194134]) and the
viewer's zoom factor (viewer+0x108, fed BLData+0x8d0 [M 0x18f5d8:134]). No-op when
amount == 0 [M 0x1bbf40:8]; per frame: φ (viewer+0x204) += 2π/(FRAME_RATE·4) (FRAME_RATE
= 60; the 4-second cycle), pitch += (zoom−1)·amount·sin(2φ), yaw += sin(φ), roll +=
sin(3φ) [M :10-24], then the view matrix rebuilds. Port: `scene::camera_swing_step` (the
pure step), `Sim::camera_apply_swing`, `Sim.swing_phase`, wired per tick in the Sim with
the weapon's `swing_amount` (`weapon.rs` +0x8c, now modelled — was "not modeled",
docs/fields/weapon_data.md:59) and the Sim's zoom. The three deltas are the view-angle
offsets the draw side applies (the port's Camera has no roll slot yet — [S] the draw
side may fold pitch/yaw only).

## Evidence tags

All game behaviour above is [M] (decompile function + line). The port's stand-ins
(head position via `Player_GetHeadPos` instead of the bone trans; the world-sphere query
instead of `Collide_SphereIntersect`; `Sim.zoom` instead of viewer+0x108) are port
choices noted at each site. Nothing here was drawn when written; suggested
compare spots: (1) eye under water in any water level — the frame should wobble at 1 Hz
(xs/ys ±0.025) and blur; (2) a GoldenEye-strike MP match — the holder's view orbits the
head at ~0.7 rad/s starting at r = 0.15 easing out to 1.5; (3) zoomed sniper fire — the
view sways at the 4-second cycle scaled by (zoom−1)·amount.
