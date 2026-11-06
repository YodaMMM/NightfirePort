# 1b: the float wedge is the walk's, and the six other aim reds

Branch from `origin/s/aim-console-phase`, merge current `origin/master`, push to `s/1b-zero-g-floor`. Read docs/reports/aim-console-phase.md: the snap is innocent on console 8.

CORRECTION: an earlier version of this task called this a game-side bug. It is not. Measured with no input:
- PCSX2: from (44.7, -8.2, 1.68) Bond settles at (44.63, -8.64, 1.73). Port: (44.58, -8.63, 1.73). Same push.
- PCSX2 and port: (46, -8.3, 0), (46, -8.7, 0) and (44.7, -8.66, 1.68) all float and hold. Port substate stays 8, not grounded. `Player_ZeroG` 0x192ab0 has no ChangeSubState, so zero-g never ends by itself.
- So nothing lands Bond in walk mode there. The test comment "walk-mode pins it, thrusters dead" is wrong: fix the comment. Collision pushes him into a pocket under the bank.

Do not change zero-g or collision code for this.

1. Find why the walk's thrust cannot leave that pocket: a thrust into the bank face, or `float_to`'s lift input. Fix the float route on 1b; the report's two-phase goal idea is the next lever. A player would simply fly out, so check that the input used is the real lift input.
2. Re-run the 1b walks until consoles 7 and 8 latch.
3. Run the `[ring]`/`[gate]` probe on 08, 0c, 14 x3 and 16, one at a time, and say which are snap faults and which are route or raw-aim tuning. 14's three are a nav stick at "the corridor south to the leg-2 spot": look at what stands there.

Keep the [L] 08 pins and the 0x30000000 fix. This branch changes where rounds land, so say what it does to the walks; it may need to join `s/stream-batch`.

---
claimed-by: sandbox5 at 2026-11-06T11:58Z
