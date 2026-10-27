# WORKLOG

One entry per task, appended at the end. Never rewrite or drop another task's entry.

## level-0d-gun-aim-audio

Task: Chain Reaction Part 2 (0700000d) deck-mounted laser gun. Two play defects on top of
s/gunbeam-0d: (1) the beam is off-axis from where the gun points, depending on aim angle —
the beam start/direction must derive from the barrel transform's muzzle and fire direction
[M GunImp_Update 0x1320d8:394-407 PositionBeam(muzzle, ray hit)]; (2) the fire audio stutters
("jittery at first then actually plays") — the TurretGuns entry's +0x18 sound (1307 for the
laser [D]) must be played on the game's cadence, found from the Sound_Play3D / looping-handle
calls in GunImp_Update [M]. Tests: beam origin = drawn muzzle, direction = barrel forward at
several aim angles [M]; fire sound starts once and holds (or repeats) on the game's cadence
while firing.

Status: started 2026-10-27 ~03:50 UTC. Setup: sandbox reset lost the previous clone;
re-cloned, branch `s/level-0d-gun-aim-audio` created from origin/master @ b864d3d6.

Status: done 2026-10-27 ~04:47 UTC. The beam's origin/direction now derive from the drawn
`barrel_matrix` (the game reads both from the one gun-cel matrix [M 0x1320d8:196-211, :325;
the disassembly pinned Concat(src,dst)=dst·src, Mat_CopyRot = rows 0-2 only, Mat_GetDir =
row 2 un-negated]); the old trig dropped the base yaw and added the offset x/y unrotated.
The fire sound holds ONE loop (handle ext+0x98 [M :247-334]: play once, hold across the
cadence, stop on release/overheat/dismount [M 0x132c18:62-63]; 0x61 spin-up/down arms and
the per-shot one-shot arm ported) — the old per-tick play3d stacked a looping voice per
shot (the stutter). Heat/cooldown drains + hot recovery added [M :100-112]. Tests:
`0700000d_gunimp_beam_axis_live.rs` (6 aim angles, origin/dir/bullet/far vs the drawn
matrix) and `0700000d_gunimp_fire_sound_live.rs` (one voice, holds, release/overheat/
dismount stops); the 8 existing gunimp tests stay green; report
`docs/reports/level-0d-gun-aim-audio.md`.
