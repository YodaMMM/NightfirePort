# WORKLOG

## sound-sweep-leftovers

Branch `s/sound-sweep-leftovers` from `origin/master` (c70343b3a). Task: fix the rows the
sound-gates sweep marked WRONG and left unfixed — `Sound_StopAllWithId` 0x12/0x3D/0xF0 on
substate change (0x190630:83-85, 0x1910f8:41-43), 0x5EF Oddjob's hat spawn (0x12c5a0:286-290),
and the report's line-201 row (the Player_ChangeSubState loop stops / 0x1DB refused-jump
leftover). Each with a test driving the event on a real level, proved red on the old code.
