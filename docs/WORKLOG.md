# Work journal

## 984-break-sound-roll (branch s/984-break-sound-roll, started 2026-10-23 16:15 UTC)

Task: `Break_Kill` 0x12b970:65-76 — kind 2 plays 0x2E with no draw; every other kind rolls
`Rand_Rand(100) < 0x29` → sound 0x5F2, else 0x5F3. The port (`crates/nf-game/src/breakable.rs`
`kill`/`effect`) always plays 0x5F3 and consumes no draw, because inserting the draw shifted the
shared RNG stream and broke escort_follow_07000001's timing (task 959 measured 7.0 s vs 18-46 s
stalls).

Plan:
1. Read the decompile `0012b970_Break_Kill` in full + disasm; read `Rand_Rand`, the debris
   creates 0x1ed548 / 0x1aaa00, the bullet pass, and the drone AI draw sites; cite every RNG
   consumer between the bullet hit and the kill in frame order [M].
2. Place the roll at the matching position in the port's stream; if escort_follow_07000001
   stalls, find why its timing depends on the stream and fix that with evidence.
3. Tests: kind 0 kill → 0x5F2/0x5F3 by the seeded roll, one draw consumed; kind 2 → 0x2E, no
   draw. Suite via tools/test/suite.py before push.

Work log:
- (16:15) branch created from origin/master efc6ed52, WORKLOG started.
