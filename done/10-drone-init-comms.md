# Guard radio chatter: Drone_InitComms and channel 0x98

No game code in the port writes channel 0x98 today, so the 0c/0d power-station guard chatter may never play. Port Drone_InitComms from the decompile, keeping its dead-guard skip: NDrone2_SetAsDead 0x14a2f0:9 sets the 0x200 flag and NDrone2_PostDroneControl 0x14aff8 sets 0x400, so dead guards must not raise it (an earlier attempt, s/markers-drone-c, dropped that skip and is rejected). Pin the chatter firing through real input on 0c or 0d.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/drone-init-comms`, report in docs/reports/drone-init-comms.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox5 at 2026-11-09T10:51Z

---
## Report (done, 2026-11-09T12:15Z)

