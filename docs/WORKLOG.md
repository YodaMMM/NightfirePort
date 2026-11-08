# WORKLOG

## level-02-bath-line (2026-11-08, UTC)

Branch `s/level-02-bath-line` cut from `origin/master` (b059df948). Task: in the bath scene
(about frame 4882) the servant's spoken line may stop before it ends. A fix for the same
symptom in the opening scene landed (starting a voice now clears a stale end flag, 74ee9b749).
Run the app path with NF_SILENT=1 NF_NULL_MIXER=1 NF_SOUND_LOG=1 (the real mixer at real
time, no audio device) through to the bath scene, read the line's start/stop and played
samples against its disc length, and fix any remaining cause. Headless tests run the sim
ahead of the mixer and starve streams; do not use them as evidence for stream length.
Reproduce on the real level first through real input, not a teleport. Compare with the
decompile, fix, and test red on the old code. No draw changes.

Entry started — decompile reading follows.
