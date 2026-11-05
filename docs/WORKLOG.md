# WORKLOG

## guard-mode-c4-glass (branch s/guard-mode-c4-glass)

Started 2026-11-05 ~19:06 UTC. Task: two player-visible divergences.
1. Drone::attack_mode (+0xc4) never set -> abseil landing (AbseilSlide 0x169730:36,
   StepOff 0x1738e0:21) and the mode-7 arm near "set_2nd_behaviour" drop the drone
   to mode 0. Game: NDrone2_GetDroneTypeAttackTypeFriend 0x14cbd8:22-23 writes
   +0xc4 = GetDTYPENEW(drone, +0xc7, 0), runs from NDrone2_DoModeSettingsNEW 0x14c600.
   Port the write at init, fix the comment, prove with a live walk (landed abseiler
   on 07000011 fights with the right mode).
2. glass_pass: false at guard spawn sites in drone.rs is a shortcut. Game clears the
   stop for every record without bit 0x800 [M Bullet_update 0x12e7b0 disasm
   0x12f030-0x12f058]. Arm it from the record as the player side does, then re-pin
   real-damage walks that relied on the old behaviour (08 station route first).
   No draws removed or reordered.

Progress log:
- Sandbox was reset; full setup redone (clone, work-slim, ISO, rust, cargo fetch).
- Branch s/guard-mode-c4-glass created from origin/master (2f0c88e2c) and pushed.
