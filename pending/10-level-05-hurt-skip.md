# 07000005: guards and the level's hurt volumes

The hurt-volume rule skips drones on 07000005 and voice-0x13 drones [M 0x13fff0:310-311]. Verify the skip's exact condition in the disassembly (level code compare, voice byte offset, delay slots), find every 05 hurt volume and which guards can reach them, and pin through real input that a guard walking into one is unhurt while Bond is hurt.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-05-hurt-skip`, report in docs/reports/level-05-hurt-skip.md. No draw changes: name what is wrong if it is a draw issue.
