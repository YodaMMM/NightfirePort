# level-4a-ride-cast — where the 0700004a ride cut-scene's characters stand (in progress)

## WORKLOG entry

level-4a-ride-cast (2026-10-27): branch s/level-4a-ride-cast from origin/master (ec917093, which
carries d6af2214's cast-revert: the SP-base compose threw 07000014's world-root control-room cast
~50 units away and was reverted; the 0700004a ride pack's car-local roots stay open). Symptom: the
0700004a lift ride's landing cut-scene (pack 0x060006cf, SP #289 base (0, -53.53, -3.38)) plays its
cast with CAR-LOCAL roots (sequence 0x040005fd root (2.35, 0.85, -0.64) [D]), so the port draws them
~53 units above the car; the real game draws them in the car (frame 700 the red-beret guard holding
the satchel up in the hatch beside the officer, frame 900 Bond under the car [L 2026-10-26]).
Plan: find the mechanism that places this pack's actors in the car (an anim parented to an entity
stream, the AnimObject's own transform, a root channel flag, the pack's type-0xC loadable record)
[M + D, PCSX2 read of the guard's anim object obj+0x30/obj+0x90 if needed], port exactly that, keep
every other cut-scene's cast where it is (cutscene_models_live, the 07000014 scene).

## What's done

Setup; this section grows as steps land.

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures to the session.
