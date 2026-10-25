## level-14-cutscene-door (2026-10-25, sandbox expiring ~20:52 UTC)
Branch `s/level-14-cutscene-door` off origin/master (aa94ce02). Task: 07000014 control-room
cut-scene — the door in the scene stays shut while Kiko walks through it (cut-scene camera
already fixed on `origin/s/cutscene-camera-base`, camera world matrix = keys alone
[M Script_CameraStart 0x1d9140]; scene viewed at Bond (40.87, 2.98, 40.85)). Find the door's
driver in that scene [D the script 0x06000833 streams + door placements nearby; M — a script
entity/anim stream on the door object, a Door_* channel the script writes (event 0xD), or
Door_Activate from the actor's path]. Port what is missing so the door opens as Kiko reaches
it; a test that the door's state/pose changes during the scene before she crosses its plane;
headless renders of the moment named in the report.
Plan: reuse sandbox work/ + ISO + nf-queue; branch pushed at start; fast suite runner started
early; census the 0x06000833 script streams (channels, events on the door object) and the
port's Door_* path; reproduce before fix; tests pin only [M]/[D]/[L]-evidenced behaviour;
report + finish steps.
