# level-14-cascade-servers-3 — the door-spam fix after the generator-hall cascade

Branch from origin/s/level-14-cascade-servers-2 (merge origin/master into it first) and read docs/reports/level-14-cascade-servers-2.md.

On 07000014, after the generator-hall cascade the guards must not keep working the door that is already open. The previous session drafted the `door_bound` fix but did not land or verify it.

Do:
1. Re-apply the door-spam fix with its [M] cites (0x153358:151-155 + 0x1681f8 case 3: at most one OpenDoor enter+exit per door of the pair, then combat-move).
2. no_guard_spams_the_opendoor_after_the_cascade green, plus the racks-stay-drawn pin from the -2 branch.
3. Every walkreal_*_live green, then the full suite total line.

---
claimed-by: sandbox5 at 2026-11-03T12:23Z
