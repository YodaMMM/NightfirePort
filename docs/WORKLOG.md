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

## level-14-cutscene-door-2 (2026-10-26, sandbox expiring ~01:08 UTC)
Branch `s/level-14-cutscene-door-2` off origin/s/level-14-cutscene-door (6acb90a1), to merge
origin/master. Task: land ONLY the door-leaf keying from that branch (script 0x06000833 streams
10/11 key the door leaves 0x02000891/0x02000892 [D]; the port never applied them to placed door
pair #116/#117), keeping master's camera-only Script_CameraStart identity rule (nis.rs case 4)
— NOT the branch's whole-script-base reset, which turns 4 master-green live tests red
(walkreal_07000001_live, walkreal_07000008_live, zoe_gondola_realpath_live,
interact_07000001_live). All four must stay green plus the door test; full suite on the merged
tree; report + finish steps.
Investigation: the game rule is disassembly-proven now — `Script_CameraStart` 0x1d9140 writes
`Mat_IdentityT` over si+0x00 (a0 untouched from entry to the jal = param_1 [M disasm]); the
case-1 main `Script_GetInterp` call stores the 10th-arg base flag = (stream->parent == -1) at
0x1d9a98 [M disasm; the decompile drops the slot]; `Script_CameraEnd` 0x1db348 never restores
the base. The port lands the identity ONLY for the default create arm's gfx-bound movers
(obj_type 0xff, hash != 0, unparented — the door-leaf class): a whole-disc census
(probe_cam_scripts) shows those are the ONLY such streams in camera-op scripts, and the
0x31/0x19/hash-0 classes keep the placement compose the live walks pin. Whole-script reset
reverted (master's camera-only rule stays).
