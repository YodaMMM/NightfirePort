# WORKLOG

## death-nis-music-restart

Branch cut from origin/master 2f0c88e2c. Task: SP_Update case 1 (0x1dbda8) reaches Script_Play too
[0x1dc174], so a death NIS started there runs the same camera tail as the load arm's —
Music_Event(6, 1), then the flush — but nis.rs leaves that arm flush-only with a gap comment.
Plan: share one camera-tail helper (kind-0x41 Event + TextFlush) between both Script_Play sites,
remove the comment, list the placements that can start a death NIS through this arm [D], pin one
through the real path if any exists (else keep a unit pin), run the nis tests, the music binaries
and nf-game.
