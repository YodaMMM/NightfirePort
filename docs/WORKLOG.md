
## aim-ease-wire-accuracy-pins
Started 2026-11-04 14:36 UTC. Branch s/aim-ease-wire-accuracy-pins cut from origin/master
182408f8. Task: real-input pins for two landed behaviours — (1) aim assist eases the offsets
toward manual at 0.5/tick with no Check_AutoAim scan while the aim button is held [M
Player_AutoAim 0x1a7c98:50-59]; (2) the move quad zeroes on climb ticks only, so a wire rider
keeps the last walk lanes and guards read him as MOVING [M Player_Update 0x193600:129-134 →
DroneWeap_DoOpponentTargetting 0x179a48:34-41 → DroneWeap_DoBulletAccuracy 0x179740:71-73].
No behaviour change expected; a pin that shows a divergence fixes it with the address.
- (14:37) branch created, WORKLOG started, initial checkpoint pushed.
- (19:41) fresh sandbox, resumed from origin/s/aim-ease-wire-accuracy-pins (the wire-rider fix
  03e6db08 was already in). Setup redone (rustup, py7zr, work-slim, the ISO). Wire part found
  complete (wire_rider_readback_0700000d_live). Aim part: read the decompile — Player_AutoAim
  0x1a7c98 runs ONCE per tick (the only jal sits at Player_Weapon 0x1a2b70-279 ← Player_Update
  0x193600:412), aim bit set → ONE 0.5/tick ease of the assist offsets toward the manual offsets
  (Player_Aiming 0x1a1d28 updates those EARLIER the same tick, 0x193600:282), no scan; bit clear →
  the scan + all four offsets halve. The port eased TWICE per aim tick (the level-0c copy in
  step_bond BEFORE the cursor update + the dir-pick copy after it): removed the early copy, the
  dir-pick ease (the game's position) is the one per-tick ease. Pin test
  aim_assist_ease_0700000d_live.rs: hip-fire lock on a 0d guard (P2K 0x02, no raise script,
  zero spread [L ee_start_05 weapon_data 0x02]), the exact 0.5^k ease law per tick, the converged
  aimed round == the view dir, the sister (button up → the scan re-snaps).
