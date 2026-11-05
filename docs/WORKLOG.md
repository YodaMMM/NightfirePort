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

Continuation 2026-11-05 13:2x UTC (second session, sandbox expires ~15:21 UTC): the work landed in
c5e98528b — both Script_Play sites push Request::TextFlush (the load arm's camera tail before the
mode arms' Script_Update so the script's own first line survives [M 0x1dbc08], and SP_Update case
1's 0x1dc174 restart [M]); the nis.rs "flush rides the view side" comment corrected (nothing
view-side does it); the type-4/5 deletion scope pinned [M 0x1b1604-0x1b1610]. This session
re-verified every decompile claim against the ELF disassembly (jal Text_FlushAllSubtitles at
0x1db618 outside the death-NIS gate beqz/beq chain 0x1db5c0-0x1db608; jal Music_Event 0x1de328 at
0x1db610 inside it), merged origin/master (107 commits), restarted the full suite (first attempt
died on a stale sandbox alsa.pc pointing at /usr — fixed the prefix libdir, not a repo file), and
is writing the report.
