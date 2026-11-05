# Guard mode byte +0xc4 and guard rounds through glass

Two player-visible divergences found by a sweep for fields that are read but never written.

1. `Drone::attack_mode` (`+0xc4`) is never set, so `self.mode = self.attack_mode` at the abseil landing (AbseilSlide 0x169730:36, StepOff 0x1738e0:21) and in the mode-7 arm near drone.rs "set_2nd_behaviour" drops the drone to mode 0. The field comment says there is no writer, but `NDrone2_GetDroneTypeAttackTypeFriend` 0x14cbd8:22-23 writes `+0xc4 = GetDTYPENEW(drone, +0xc7, 0)`, and it runs from `NDrone2_DoModeSettingsNEW` 0x14c600, the chain the port's `mode` derivation follows. Port the write at init, fix the comment, and prove with a live walk that a landed abseiler on 07000011 fights with the right mode.

2. Guard rounds stop on glass: `glass_pass: false` at the guard spawn sites in drone.rs is a documented shortcut. The game clears the stop for every record without bit 0x800 [M Bullet_update 0x12e7b0 disasm 0x12f030-0x12f058]. Arm it from the record as the player side does, then re-pin the real-damage walks that relied on the old behaviour (the 08 station route first). No draws removed or reordered.

Check each with the decompile, cite [M]/[L], and keep tests honest: real input, no teleports.

---
claimed-by: sandbox7 at 2026-11-05T18:11Z
