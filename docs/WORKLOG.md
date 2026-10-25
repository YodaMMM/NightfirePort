### level-4a-rappel-vanish (started 20:17 UTC, 2026-10-25)

Task: 0700004a lift ride — "rappel guards disappear after shot": a guard on a
rope vanishes instead of dying/falling when hit. (1) Find the 4a abseil guards
[D], drive the real ride with real input, shoot one, log state/anim/visibility
per tick. (2) Compare with the game [M — NDrone2_DSTATE_AbseilSlide 0x169730 /
AbseilHang 0x169940 damage arms, the death on a rope (the fall, the corpse),
abseil_death_live on 11]. Fix the port so a shot rope guard does what the
game's does; real-input test; headless renders named in the report. Branch from
master (24084ef2). Sandbox deadline ~22:14 UTC (stop new work 21:54).

---
level-4a-rappel-vanish (2026-10-25 20:40): root-caused + fixed, tests green, checkpoint push.
  The 4a rope-guard vanish: AbseilDeath's 4a UPDATE arm deleted the corpse on the per-tick cull's
  DRAWN flag — the guard disappeared the kill tick (repro: real-input ride, abseiler #282 wakes on
  ch 0x3d [D], ropes, a killing round → state 0x92 + present=false the same tick while drawn/grace 2).
  The game [M 0x169b28 case 3 on 0x700004a]: obj+0xFC != 0 (drawn recently) → the corpse persists;
  once the drawn-grace is out → Enable(0) + obj+0xFE |= 1 (soft delete). And the corpse FALLS:
  AbseilDeath's ENTER sets +0x4F8 = (& ~0x20) | 4 [M 0x169b28:18-21] → DoCollision 0x149ec8:12-14
  returns 0 (no collision) while DoGravity 0x141e58:12-15 returns 1 on bit 4 before the 0x20/0x200
  folds — Drone_CollisionHandler 0x140968:152-192's DoGravity arm integrates the corpse
  collision-free (velocity += WldGravity·r, pos += velocity·r, no damping — the ×0.98 tail is the
  rope arm's — no settle). The port froze the corpse (bounded=false skipped the whole handler) AND
  deleted it when drawn (inverted gate, wrong field). Fix: keeps_gravity field (bit 4), a
  collision-free gravity arm in collision_handler for !bounded && keeps_gravity, and the UPDATE
  gate on drawn_grace == 0 (the DEAD arm's own pattern). Tests: nf-view
  rappel_vanish_0700004a_live (real-input ride → kill → the fall pinned exactly + the 3rd-undrawn-
  pass delete + the disc census), nf-game abseil_death_live 5/5 + nf-view abseil_elevator_live 3/3
  still green (the 11 path untouched).
