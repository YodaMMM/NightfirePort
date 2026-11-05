# WORKLOG

## subtitle-flush-camera-scripts
Started 2026-11-05 06:09 UTC. Branch s/subtitle-flush-camera-scripts (cut from origin/master
a89818235). Task: Script_Play 0x1db588 calls Text_FlushAllSubtitles 0x1b15c8 on every camera
script start (+0xAA8 & 0x40), the flush after the gated music event is NOT gated — the port
flushes only on the skip arm (drone.rs subtitle_track.flush ~16705) and the nis.rs:1688 comment
claims the flush rides the view side (verify; correct either way). Add the flush at the
Script_Play site for every camera script, matching 0x1b15c8's deletions (type-4/5) and leaves;
the flush-vs-own-first-line order must follow the game. Pin on a real level with a subtitle live
as a camera script starts + a non-camera script leaves it alone. Run the subtitle/cut-scene
binaries (grep subtitle_track), the nis tests, nf-game; suite via tools/test/suite.py. Sandbox
expires ~08:07 UTC.
