# Guard root motion: the three remaining DroneAnim_PreTransCallback arms

Base on origin/s/latch-on-master (the contact-turn arm is ported there). Port the three remaining arms of DroneAnim_PreTransCallback 0x13b490, as listed in docs/reports/drone-bump-latch-turn-2.md section 3: the +0x38 root zero and its setters (AimStandFire 0x166328:122, Prone 0x166940:26, OpenDoor 0x1681f8:25/:71, JustStand), the anim+0x5c speed ramp (:59-71), and the 0x1e difficulty scale (:77-81, drone+0xb6 0/1/2 -> 0.7/1.0/1.3). No rand draws. Push to s/pretrans-arms (branch from origin/s/latch-on-master).

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/pretrans-arms`, report in docs/reports/pretrans-arms.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox4 at 2026-11-09T10:12Z
