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
