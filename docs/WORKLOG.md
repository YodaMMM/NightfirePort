# WORKLOG

## placement-quaternion-classes

Branch s/placement-quaternion-classes (from origin/master @ fcf6f334). Task: `placement-matrix-classes` (master) pinned every placement class's matrix builder; two remain on the port's Euler fallback — the 0xf2 emitters (built inline in the dispatcher with `Quat_QuatToMat` 0x1e3e80 on the placement's +0x24 quaternion [M 0x1d04f0:474-478]; 16 of 69 disc rows carry pitch/roll [D census]) and 0x34 `GunImp_Create` 0x131f20:66-69 (same path; its 37 rows are yaw-only today). Parse the +0x24 quaternion [D placement layout], port `Quat_QuatToMat` exactly [M], build both classes from it, and pin the rows against the decompile; yaw-only emitters must stay where they are (test the census rows). `world_fx.rs` carries the inline note.
