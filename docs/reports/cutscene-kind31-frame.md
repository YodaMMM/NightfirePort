# cutscene-kind31-frame — the general rule for unparented kind-0x31 script streams (in progress)

## WORKLOG entry

cutscene-kind31-frame (2026-10-27): branch s/cutscene-kind31-frame from origin/master. Master's
level-14-capture-placement fixed Countdown's capture-scene hand-back by excluding script
0x0600062F's unparented kind-0x31 streams from the SP base compose (they key in world frame once
Script_CameraStart 0x1d9140 has reset the script base to identity), scoped to that one script on
purpose (07000001's intro 0x06000087 s5/s11 still compose). Plan: find the general rule the game
applies [M Script_Interp 0x1d9a08 case 1 :91-138, Script_CameraStart's base reset,
Script_EventHandler 0x1d94c0 case 6, Script_Free 0x1da880:44-48], replace the per-script scope
with it, census every SP script's kind-0x31 streams [D], check each scene's hand-back against the
level geometry; the 01 intro and 14 capture tests stay green.

## What's done

Setup; this section grows as steps land.

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures to the session.
