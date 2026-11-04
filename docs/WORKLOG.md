
## aim-ease-wire-accuracy-pins
Started 2026-11-04 14:36 UTC. Branch s/aim-ease-wire-accuracy-pins cut from origin/master
182408f8. Task: real-input pins for two landed behaviours — (1) aim assist eases the offsets
toward manual at 0.5/tick with no Check_AutoAim scan while the aim button is held [M
Player_AutoAim 0x1a7c98:50-59]; (2) the move quad zeroes on climb ticks only, so a wire rider
keeps the last walk lanes and guards read him as MOVING [M Player_Update 0x193600:129-134 →
DroneWeap_DoOpponentTargetting 0x179a48:34-41 → DroneWeap_DoBulletAccuracy 0x179740:71-73].
No behaviour change expected; a pin that shows a divergence fixes it with the address.
- (14:37) branch created, WORKLOG started, initial checkpoint pushed.
