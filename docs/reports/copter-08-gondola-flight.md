# copter-08-gondola-flight — the 07000008 gondola helicopter flies its attack, not a hover

Task: on 07000008 the helicopter that attacks the gondola flies in, then holds one spot in the air,
and the mission completes. The port's copter hit walk (Copter_Update 0x17dbb8:277-320) and the gondola
completion on real shots are on master; the flight itself is the open part. Work in progress — updated
as steps land.

## Plan

1. Trace the port from the ride's start: the copter's position, route node, Copter_Update's case and
   target each tick until the kill; find where it stops moving.
2. Port the flight from the decompile: `Copter_Move` 0x17d628 (path follow), `Copter_Update` 0x17dbb8
   (case machine: approach, attack runs, strafes, hit reactions), the stream/route data [M + D]; cite
   each case's exit.
3. Tests from the ride (real input): the copter's position keeps changing between attack passes
   (moving-window distance check, the decompile's speeds), it dies on real shots, the mission completes
   only after the kill. Sister: an unshot copter never completes the mission.
4. Name the PCSX2 read that would confirm the flight (fresh boot, the copter's +0x30 over time).

## Status

- (in progress)
