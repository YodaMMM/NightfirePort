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

Work log (continued):
- (16:40) The draw-order census [M]: `Rand_Rand` 0x1e3738 (two 16-bit LCG steps, Y then X, value
  `(SEED_Y&0xffff)·n>>16`; the port's `Rng::rand` already matched). On the kill frame:
  `Env_Update` 0x17bc88 draws `Rand_Rand(20000)` every frame with a world (rain #279 on
  07000001); `Drone_InitComms` 0x139520 / `Doors_Calc` 0x135fe0 / `Trigger_Calc` 0x1b1900 draw
  nothing; `Bullet_update` 0x12e7b0 draws `FRand(0.5)` per flying bullet per tick (the walk is
  newest-first, bullets first); `Break_Update` 0x12bfd8 is class 0x20's update ([L
  ee_start_05.bin@0x26f890+0x20·12 = 0x0012bfd8]) and consumes the PREVIOUS frame's contact
  (contacts come from `Collide_Update` 0x1ea398, the last step of
  `control_movement_object_handler` 0x1336f8 — so the kill frame is the flight frame + 1);
  `Break_Kill` 0x12b970:59-66 draws the roll after the records gate, before both sound arms and
  the pieces; `Drone_BrokenObject` 0x138588 draws nothing (0 Rand calls); `Debris_Create_Loop`
  0x1ed548 → `Debris_Create` 0x1aaa00 draws 8 per record (FRand(0.1), Rand_Rand(half),
  FRandHalf(spread)×3, FRandHalf_Vec(spin)×3); `ShatterPolyRecurse` 0x12b270 draws in the
  glass arm; `Collide_Update` draws `Rand_Rand(0x3f)` only on surface-byte bit 0x80 contacts
  (the vase's 0x10 material carries neither 0x40 nor 0x80 — no draw on the kill contact); the
  escort machine states 0x15e698/0x15eda8/0x15ec08/0x15efc8/0x15f120/0x15f2e8/0x15f938 draw
  nothing (the AI draws near the kill are the other guards' combat/head-tracking/idle rolls).
- (16:55) The roll ported at the game's in-kill position (`kill()`: roll before the pieces'
  draws; kind 2 no draw). nf-game lib 522/0; break_debris_live's twin-vase draw pin 8n → 1+8n
  [M 0x12b970:66] — the old 8×count pin contradicted the decompile (the roll precedes the
  debris loop), fixed with the addresses.
- (17:26) The canaries on the roll tree: escort_follow_07000001_live 1/1 (level_exit 07000002
  at tick 25939, 7 stall-recovery rounds, 10.0 s wall), walkreal_07000001_live 3/3 (21.5 s),
  breakables_live 2/2, glass_breach_live 1/1, interact_07000001_live 10/10, break_debris_live
  2/2. Task 959's 18-46 s stalls do not reproduce on the roll alone — that branch also added
  "Bond in the level-01 class-2 contact pass" + the intact-pane set_dynamic injection (its own
  revert note says the injection stalled walkreal), so the stall blame lands on those, not the
  roll draw.
