# level-4a-cutscene-autoaim — 0700004a (the Night Shift lift ride): the cut-scene and auto-aim from the moving cab

Branch `s/level-4a-cutscene-autoaim` off origin/master (62f70bcc). Sandbox window 17:28–19:27 UTC; this report written ~19:05 UTC.

## 1. The cut-scene: what the level plays, logged from a fresh load

`crates/nf-view/tests/probe_0700004a_cutscenes_live.rs` — fresh `nf_view::load("0700004a.bin")`, no `--bond`, no pos writes, idle input, 7000 ticks. The log:

| t | event |
|---|---|
| 0 | SP #42 starts, pack 0x06000571 (SCRIPTPLAYER_Lift, mode 1 auto-start, out 0x22) — **no camera**: the ride is playable, matching the game [D the level's SPs, docs/levels/0700004a.md §7] |
| 1611 | SP #289 starts, pack **0x060006cf** (out 0x40) — the −52 landing cut-scene; camera ON, `disabled → true` the same tick |
| 1611–2186 | the script camera: shot A fov 0.671977 (key scale ≈ 0.504) pos (2.8885, −61.42→−62.19, −7.96) descending, pitch swinging +0.41→1.1257; cut t1940 to fov 0.375120 (tele, key scale ≈ 0.281) static at (4.3288, −53.2732, −6.8961); cut t2060 fov 0.500389 pitch −1.4442 (looking up the shaft); cut t2090 fov 0.459296 (key scale ≈ 0.344 — the walkreal note's "telephoto scale 0.344" [D 0x060006cf]) descending 0.0116/tick to (2.0938, −63.39, −5.45) |
| 2186 | camera OFF — the hand-back; `disabled → false` t2187. **575 ticks, exactly the [L lift-4] hold** the walkreal notes pin ([D 0x060006cf s25's t575 marker]) |
| 3088 | SP starts, pack 0x06000052 (placement = usize::MAX — a non-SP source, out 0, mode 1) |
| 3540 | level exit (idle ride; the walkreal's ridden run exits later through the bottom plate) |

**No near-zero fov key and no inside-geometry camera** on this level's scripted cut-scene: the fov keys are 0.504/0.281/0.375/0.3445 × script scale × 4/3 and the eye positions sit in the shaft well by design (it films the cab from the shaft). The port's camera arithmetic is already bit-pinned to the decompile by `fix_camera_exact_math.rs` (Script_GetInterp 0x1da158 keys, case 4's base-concat per the 0x1da054 disasm, Quat_QuatTransToMat 0x1e3fa8) — this probe re-verified the produced values live on 4a.

What CAN differ from the game and is worth the main agent's render check (§5): the port hard-codes the fov's aspect multiplier to 4/3 (`nis.rs:1013`) where the game multiplies by the live viewer's aspect — 4:3 only when the widescreen flag DAT_002a37c4 is 0, 16:9 (0x3FE38E39) otherwise, per-viewer-split halved/doubled [M Camera_CalcViewAngles 0x1baf70:22-42]. On a 16:9 render the port's cut-scene shots are ~25% tighter than the game's. Marked [S] unverified against a real 16:9 boot; not pinned in tests.

## 2. Auto-aim from the moving cab: the game has NO velocity terms anywhere

The census answers the driver's question directly — neither cab-relative nor world velocity exists in the game's chain:

- **Pick** `Check_AutoAim` 0x1a73f8: instantaneous geometry only — head → target bone −2, nearest in the angular cones (|Δyaw vs obj+0x54| < Autoaim_Angle_H·k, |Δpitch vs BL+0x8a8·π/2| < Autoaim_Angle_V·k, k = weapon+0x14·0.01·(1−d/25)·difficulty ×1.4 lock), LOS `Collide_LineOfSight` mask 0xa [M 0x1a73f8:110-166]; the winner's **screen-space** offsets (sx−0x10c/2)/(0x10c/4), (sy−0x110/2)/(0x110/4) → BLData+0x120/+0x124 + lock +0x114 [M 0x1a73f8:167-178]. Viewer dims 512×448 [M Camera_CreateCameras 0x1bb2b8:18-23], planes from tan(fov/2) with MAX_VIEW_CONE ratio ≈ −1.0002 [M 0x1baf70:36-50; L symtab 0x30d0fc = 2000.0].
- **Apply** `Player_GetAimingPoint` 0x1a4a98: the offsets re-project through the same planes into a local dir, RotX(−pitch·π/2)∘Bond-matrix, ×−256, + head [M 0x1a4a98:48-83] — the chain is self-consistent (re-projection inverts the pick projection), so at a fixed fov the screen-space path aims identically to angular offsets; it diverges only when the fov changes between pick and fire (zoom).
- **Bullet** `Player_WeaponInitBullet` 0x1a4bf0 → `Bullet_init` 0x12c5a0: dir = normalize(aim−head), speed = weapon+0x20 — **the shooter's (cab's) velocity is never inherited** [M 0x1a4bf0:152-153, 0x12c5a0:166-191].
- **Rider** `Player_FeetOnPoint` 0x194188 → `Control_InheritVelocity` 0x133e88: the platform delta moves the rider's POSITION only; the aim frame stays world (head world, yaw/pitch world) [M 0x133e88:64-143, 0x1a1d28:192-193,273-276].
- **Aim mode** (the 0x60 bit): no new scan; the assist offsets ease halfway toward the stick-inertia pair (+0x118/+0x11c) per tick and the lock survives [M 0x1a7c98:51-58]; outside aim mode all four offsets halve [M 0x1a7c98:112-115].

The port already matches all of this structurally: the pick cones/LOS/k in `drone.rs:14021-14078`, the aim-mode ease to the cursor at 0.5/tick with the lock kept (`scene.rs:3949-3957`), the decay, and `fire_player` with no inheritance (`drone.rs:12683-12783`). The known port deltas ([M]-cited, not lift-specific): the port applies angular offsets where the game stores screen-space ones (equal at fixed fov), the port scans drones only vs drones+`glb_players[0..3]` [M 0x1a7c98:83-102], the Q-Claw should scan `'Q'` grapple objects [M 0x1a7c98:64-80], and the port lacks GetAimingPoint's aim-ray geometry clamp [M 0x1a4bf0:158-164].

## 3. Why the real-play pick feels dead on the ride (the root cause chain)

`crates/nf-view/tests/fix_autoaim_elevator_live.rs::the_cab_ride_replays_and_the_moving_cab_sees_a_guard_the_pick_cone_consumes` rides the cab with real input (no pos writes, no re-seating, landing cut-scene watched): the cut-scene plays and hands back, the ride covers the shaft (y −8.08 → −155.62), the per-floor guards sit on the ride line (y −27/−67/−79/−99/−119, team 1 [D census]), and **the sensor LOS from the moving cab reaches a guard for 150 ticks through the 0x40-material well panes** (t2701+, cab y −71.59 — the pick's own LOS mask [M 0x1a73f8:164-165]). Two port gaps keep the pick dark in real play, NOT the aim math:

1. **The shaft doors never auto-open on the lift's approach** — the level's documented KNOWN GAP (docs/levels/0700004a.md §6, walkreal_0700004a_live header). In the game the doors open as the cab passes (the guards at the floors become visible/aimable); in the port they stay closed, so the pick's LOS (which correctly passes the well panes but not doors) never opens a window in real play.
2. **The scene harness never sets the render-side `drawn` bit** the pick gates on [M 0x1a73f8:65-109], so a headless pick cannot fire through the nf-view harness (the nf-game rig sets it — see the pin below).

## 4. Pins added (all green)

- `fix_autoaim_elevator_live` (nf-view): the ride replay pin — cut-scene ON→OFF hand-back, the full descent, ≥30 sensor-LOS windows from the moving cab through the panes.
- `ai_live::auto_aim_from_a_moving_shooter_is_the_same_instantaneous_geometry` (nf-game): the no-lead pin — the head carried one cab-tick down (150/4950 [D 0x06000571 keys]) re-picks the same guard with the ray re-snapped to bone 2 from the NEW head and the assist offset tracking the geometry, not a velocity history [M the chain in §2].
- `probe_0700004a_cutscenes_live` (nf-view): the §1 log (assertion-free).

Ran: `cargo test --offline --release -p nf-game --test ai_live auto_aim` (2 passed), `-p nf-view --test fix_autoaim_elevator_live` (1 passed), `-p nf-view --test probe_0700004a_cutscenes_live` (1 passed). Full suite via `tools/test/suite.py` — total line in §7.

## 5. Headless render spots for the main agent (the landing cut-scene, pack 0x060006cf)

The script camera drives the eye; the frames to render (from the probe's log):

| frame | eye (script camera) | expect |
|---|---|---|
| t1620 | (2.8885, −61.4215, −7.9598) yaw 0.0063 pitch 0.4069 fov 0.672 | shot A: the cab interior/Bond from below-beside while descending |
| t1940 | (4.3288, −53.2732, −6.8961) yaw −0.5223 pitch 0.1959 fov 0.375 (tele) | the landing hold: the platform/Bond at −54.15 through the telephoto |
| t2070 | (3.0531, −50.8883, −4.7352) yaw 2.7519 pitch −1.4442 fov 0.500 | looking up the shaft from above the cab |
| t2140 | (2.0938, −62.9202, −5.4492) yaw −3.0261 pitch −1.2681 fov 0.459 | the last shot descending |

What to compare: the shot visible vs a flat void (fov/frustum path), geometry drawn from the shaft-well eye (portal/cell resolution from a non-Bond eye), and the 16:9 tightness noted in §1.

## Live checks for the main agent

1. **The shaft doors' auto-open** (the §3 root cause): in PCSX2 on 0700004a, gdbwatch a −72-floor door pair's obj+0x90 (or its open channel) through t2650-2750 — expected: the leaves start opening as the cab approaches/plates fire, BEFORE any drone touches them. If the opener is a channel the plates write, the port needs the door↔plate wiring; if proximity, `Door_Update` 0x1363c8's approach arm.
2. **The pick's drawn bit on the ride**: fieldprobe Bond's BLData+0x114 (the lock) while auto-aiming a visible guard from the cab — expected: nonzero while the guard is on screen (the drawn gate passes in real play).
3. **The 16:9 fov**: boot with the widescreen option ON, play the landing cut-scene, screenshot — expected: the game's shots wider than the port's 4/3-hard-coded fov.

## 6. What's left

- The shaft-door auto-open port (the level's KNOWN GAP §6) — the actual real-play fix for "auto aim doesn't play too well with shooting from the elevator"; the pick/bullet chain needs nothing.
- The screen-space offset port (GetAimingPoint's exact re-projection + the +0x118/+0x11c stick pair's writer) and the drones+players scan — [M]-cited, not ride-specific.
- The parked spawner ride gap (the garrison never releases for a rider — the level doc §7's open live check) and this report's live checks.
- A render check of the §5 frames on a 16:9 canvas.

## 7. Suite

Fast runner `tools/test/suite.py` (801 binaries, build 943 s) started 17:29 UTC in the background on the branch tree; final-tree total line and the FAILED/TIMEOUT list: see the last line of `suite.out` in the sandbox at finish time — as of 19:05 UTC it had reported `FAILED probe_pylon_render::probe_render_pylon_only` and `CRASH mp_perf_16_live (exit -9)` (both pre-existing master tests, not files this task touches; the CRASH is a kill -9 under this sandbox's memory pressure from the parallel probe builds). My touched-crate runs are listed in §4.

## Usage

not visible to me (no token/credit/context-window/cost figures are shown anywhere in this sandbox environment; the agent transcript does not display usage meters).
