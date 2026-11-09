# WORKLOG

## pain-generic-gate

Task: the game's health gate on every player damage path. Player_HandlePain 0x1902c8
returns on health <= 0 before writing anything (per the driver's brief); the port's
pain_generic path lacks that gate. Read every player damage entry (bullets, blasts,
melee, falls, gas, hurt volumes, scripted kills) and give each the game's gates in the
game's order; pin a case where the old code changed state on a dead or zero-health
player. Reproduce on the real level first through real input, never a teleport; test
red on the old code; walkreal_* every level the change touches, one level per tool
call. No draw changes.
Branch from origin/master at 290dc4b03. Report: docs/reports/pain-generic-gate.md.
