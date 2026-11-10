# 07000001: a guard pushed off the upper west floor falls through

Continue blocked/10-guard-fell-through-floor.md (read its report and live-check list). With solid guard bodies a 01 guard was pushed off the upper west floor and fell through (y about -1e6). An earlier attempt (branch s/guard-floor-keep, if present) was stopped mid-way while proving its test red on the old code. Find why a pushed guard loses the floor (the body push moving him past the AI bounds or a cel edge the port lacks), compare with the game's drone bounds push and floor handling (AINetwork bounds, Drone_FeetOnPoint), and fix the port with a test red on the old code through real input. Run every walkreal_* binary one per call if guard movement changes. The two PCSX2 checks in the blocked report are the main agent's; name what they would settle.

Branch from origin/master, push to `s/guard-floor-keep-2`, report in docs/reports/guard-floor-keep-2.md. No draw changes.

---
claimed-by: sandbox4 at 2026-11-10T07:31Z

---
## Measured facts (added 2026-11-10T07:45Z)

- [L PCSX2, 07000001, god held, Bond placed, ch 0xF raised]: before ch 0xF the guards in x -40..-15 sit in state 1 and never move. After ch 0xF the upper-floor pair (from (-27.2,10.2,2.3) and (-29.1,6.3,0.0) climbing to (-25.0,10.3,2.5)) holds cover (states 5e/5f/69/95). With Bond standing at z 4.2, 8.0, 12.0 and 16.2 for ~1800 frames each, no upper-floor guard came north of z 6.1, and none lost the floor (min y 9.7). The game's guards never reach the z 16.7 north edge in this setup, so look first at why the port's guard goes there (chase vs the game's cover states).
- [M NDrone2_Collision 0x140e90:169-174] the bounds push runs last in the collision pass, only while drone+0x4F8 bit 31 is set, radius 0.4, mask drone+0xBA4. [M AINetwork_GetBoundsPushVectorForSphere 0x157bd0] it walks only the guard's current cel's boundary list (cel+0x34). The port's push_out walks every wall on the level, which is stricter, so a push applied after push_out (or a skipped push_out) is the likelier way out.

---
## Report (done, 2026-11-10T09:03Z)

