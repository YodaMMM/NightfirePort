# Guard root motion: the remaining DroneAnim_PreTransCallback arms, redone

s/pretrans-arms was rejected (audits note: the +0x38 zero-root latch was made sticky). Redo from origin/master: NDrone2_PreDroneControl 0x148db0:23 clears +0x38 at the start of every guard frame (add that clear at the port's PreDroneControl site before the UPDATE send); port both setters (AimStandFire 0x166328:122 and DroneWeap_AimTarget_IsOpponent 0x178ae0:52, +0x56a in 0x1e..0x1f and CanMoveToRelativePos) plus the others listed in docs/reports/drone-bump-latch-turn-2.md; then the anim+0x5c speed ramp (0x13b490:59-67) and its hip write, then the 0x1e difficulty scale (:77-81). Run every walkreal_* binary one per call and repair walks as a player would.

Branch from origin/master, push to `s/pretrans-arms-2`, report in docs/reports/pretrans-arms-2.md. No draw changes.

---
claimed-by: main-session agent at 2026-11-09T17:28Z

---
## Closed (2026-11-10T09:06Z)
Superseded by pretrans-arms-3/-4 (done) and 20-pretrans-arms-5 (pending).
