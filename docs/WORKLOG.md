## level-4a-cutscene-autoaim (2026-10-25, sandbox expiring ~19:27 UTC)
Branch `s/level-4a-cutscene-autoaim` off origin/master (62f70bcc). Task: 0700004a Night Shift
lift ride, two real-play reports: (1) the level's intro/scripted cut-scene "is busted" — log
which scripts run, the camera per tick (position, fov — near-zero fov key draws a flat void;
camera inside geometry draws nothing), the actors, the hand-back; compare with the game
[M Script_CameraStart 0x1d9140, Script_UpdateCamera 0x1db518, Script_Interp case 4 0x1d9a08,
the actor entity streams] and fix what differs; headless renders of the cut-scene frames named
in the report. (2) "Auto aim doesn't play too well with shooting from the elevator" — Bond
riding the moving cab, auto-aim tracks badly; find the game's auto-aim target pick and lead
[M Player_* aim assist, cab-relative or world velocity] and fix the port; a real-input test
riding the cab and shooting a guard.
Plan: reuse sandbox work/ + ISO + nf-queue; branch pushed at start; fast suite runner started
early; census Script_CameraStart / Script_UpdateCamera / Script_Interp case 4 and the port's
camera path, then the auto-aim census (target pick + lead, moving shooter); reproduce before
fix; tests pin only [M]/[D]/[L]-evidenced behaviour; report + finish steps.

- (17:28-19:10) Branch pushed at start; fast suite up (run_suite.sh launcher: alsa prefix env;
  the nohup needs setsid + cargo on PATH here). [M] cut-scene census: fresh-load probe
  (probe_0700004a_cutscenes_live) — SP #42 0x06000571 t0 (no camera, the playable ride), the
  landing NIS 0x060006cf t1611-2186 (camera ON, 4 shots fov 0.672/0.375/0.500/0.459 = key scales
  0.504/0.281/0.375/0.3445, hand-back t2186 = the 575-tick [L lift-4] hold, disable off t2187),
  SP 0x06000052 t3088, exit t3540 — no near-zero fov, no inside-geometry eye; the camera math is
  already bit-pinned (fix_camera_exact_math). Open delta [S]: the port's fov aspect is hard-coded
  4/3 [nis.rs:1013] vs the game's live viewer aspect [M 0x1baf70:22-42] (16:9 tighter shots).
  [M] auto-aim census: the game's chain has NO velocity anywhere — Check_AutoAim 0x1a73f8
  (instantaneous cones + LOS mask 0xa, screen-space offsets 0x120/0x124), GetAimingPoint 0x1a4a98
  (re-projection through the 512x448 viewer planes, self-consistent at fixed fov),
  WeaponInitBullet 0x1a4bf0/Bullet_init 0x12c5a0 (no shooter/cab inheritance), FeetOnPoint
  0x194188 → InheritVelocity 0x133e88 (position only, aim world). Port matches structurally.
  The real-play "tracks badly on the elevator" root cause: the shaft doors never auto-open
  (the level's KNOWN GAP §6) + the harness's unset drawn bit — the ride pin shows 150 sensor-LOS
  ticks from the moving cab through the 0x40 panes (t2701+, y -71.59) that the game's pick would
  consume. Pins: fix_autoaim_elevator_live (ride+hand-back+LOS windows) and ai_live::
  auto_aim_from_a_moving_shooter_is_the_same_instantaneous_geometry (no-lead). Report + renders
  + live checks: docs/reports/level-4a-cutscene-autoaim.md.
