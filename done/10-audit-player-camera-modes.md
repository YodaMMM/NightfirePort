# audit-player-camera-modes — every player camera mode against the port

Walk Player_SetCamMode's values and every arm of Player_PositionCamera 0x18f5d8: eye offset, zoom factor (viewer+0x108), follow target, roll, and which substate sets each. For each on-foot mode find the port's camera path in crates/nf-view/src/lib.rs and scene.rs. Report a table: mode, address, port site or MISSING/DIFFERENT. Fix each difference a player can see, with the address and a real-input test. A camera change that alters motion feel: leave it on a branch and name it in the report. Skip driving. Done = the table, fixes tested, every walkreal_*_live green, the full suite total line.

---
claimed-by: sandbox4 at 2026-11-04T18:37Z

---
## Report (done, 2026-11-04T20:18Z)

Closed by the driver: the report is on `s/audit-player-camera-modes` (docs/reports/audit-player-camera-modes.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-04T20:23Z)

