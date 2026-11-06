# WORKLOG

## weapon-change-refused

Branch cut from origin/master 7370525a4a0310a0e8a3a1250585291202b4d51a. Task: parse the
`level_tag+0x62` byte that gates the refused weapon change (sound 0x1DB,
Player_WeaponChange 0x1a6f80:128 per docs/systems/sound-coverage.md, currently MISSING),
find which levels set it, port the refusal (weapon stays, 0x1DB plays), test on a real
level that sets it, proved red on the old code. No guessing at the byte's meaning: cite
its reader and the disc value.
Progress: setup reused from an earlier session (repo, work/, ISO, nf-queue, cargo present);
branch created fresh from origin/master 7370525a4a0310a0e8a3a1250585291202b4d51a. Started
21:15 UTC; deadline ~23:11 UTC, no new work 22:51.
