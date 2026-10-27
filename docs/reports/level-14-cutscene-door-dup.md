# level-14-cutscene-door-dup — the control-room cut-scene door's static copy (in progress)

## WORKLOG entry

level-14-cutscene-door-dup (2026-10-27): branch s/level-14-cutscene-door-dup from origin/master
(ec917093). Symptom: on Countdown Part 1 (07000014), the control-room cut-scene door (Bond at
(40.87, 2.98, 40.85)) moves with the script streams (level-14-cutscene-door-2 keyed leaf) but a
static copy of it stays in place behind it, so the doorway still looks shut. Plan: identify which
instance duplicates the scene door [D placements + the scene's Entity/stream objects], find the
game's rule that hides one while the script moves the other [M Script_HideObj 0x1db808 / the SP
hide cascade / the door's own draw gate / the script entity replacing the placement], and fix so
exactly one door draws and it moves with the scene.

## What's done

Setup; this section grows as steps land.

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures to the session.
