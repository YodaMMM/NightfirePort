# 07000002: the servant's line in the bath scene

In the bath scene (about frame 4882) the servant's spoken line may stop before it ends. A fix for the same symptom in the opening scene landed (starting a voice now clears a stale end flag). Run the app path with NF_SILENT=1 NF_NULL_MIXER=1 NF_SOUND_LOG=1 (the real mixer at real time, no audio device) through to the bath scene, read the line's start/stop and played samples against its disc length, and fix any remaining cause. Headless tests run the sim ahead of the mixer and starve streams; do not use them as evidence for stream length.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-02-bath-line`, report in docs/reports/level-02-bath-line.md. No draw changes: name what is wrong if it is a draw issue.
