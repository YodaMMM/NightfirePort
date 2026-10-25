# level-0a-rain-indoors-5 — the 41 glass pane: the test was wrong, not the game

## What the task was

`interactables_07000041_live::the_glass_pan_shatters_under_the_real_fire` — the one red on the
merged tree (rain-4 + master), green on master: the rain-3 report's second stream-shift red, then
believed to be "a pinned fire window riding the shared generator's position". The task: find which
draw decides the glass break on that path (the fire spread, a damage roll, the glass health),
decide whether the test pins a game rule or one lucky stream value, and make it assert the game's
rule — never weaker than [M].

## The answer: no draw decides the break — the test's own geometry was the red

The break is **rule-decided end to end**; the shared generator is irrelevant to it:

- The pane's health is its placement param 1 — pane #48 carries **0** [D PL#48; M 0x12c230 disasm
  0x12c2a8-0x12c2e0 `cvt.s.w` — the port's `health: (param(1) as i32) as f32`].
- `Break_Update` subtracts the contact's damage and calls `Break_Kill` exactly on the
  `health < 0.0` crossing [M 0x12bfd8:44-53]. The damage is the round's FIXED HITTEST word —
  `weapon_data+0x0C` = **3.5** for the P2K [L ee_snapshot.bin weapon_data @0x2bf150 + 2·0x10C =
  0x2bf368, +0x0c; the record's +0x08 gate is 0.0 so no override applies] — written per flight
  tick by `Bullet_update` [M 0x12e7b0 disasm 0x12f05c-0x12f0b8 `lwc1 f0, 0xC(weapon_def)` →
  `swc1 f0, 0xE0`]. **No Rand call anywhere on that path** [M 0x12bfd8 has none; 0x12e7b0's only
  Rand is the whoosh arm's sound pitch].
- The fire path's own shared-stream draws move nothing here: `Bullet_init` draws the three spread
  values [M 0x12c5a0:167-170 `Rand_FRand_MVar2(2s)`, `Rand_FRand(TAU)`, `Rand_FRand(PI)`] and
  applies a radial offset scaled from the record's +0x24 spread field — the P2K's field is
  **0.0** [L ee_snapshot.bin 0x2bf368 +0x24], so every round flies the aim exactly. (The draws are
  still consumed — the stream shifts — but the P2K cannot miss a dead-centre aim.)
- The game spawns the round from the player's head bone and re-aims it at the aim point [M
  Player_WeaponInitBullet 0x1a4bf0:226-245 `AnimGetBoneWorldTrans(BL+0x7e8,0,0,·)` +
  `Vec_Normalise(aim − start)`]; the port fires from `Bond::head` the same way [M
  crates/nf-view/src/scene.rs run()'s `camera` arg = `self.bond.head()`].
- `Bond::look` reads the view pitch as the game's **±1 value scaled by π/2** [M RotMatrixX
  0x1e46a0:16-24 — the port's `ps2float::mul(self.pitch, HALF_PI)`].

The old test was wrong three ways, and the pass on master was the third wrong thing:

1. **It wrote RADIANS into the ±1 pitch field.** `sim.bond.pitch = atan(dy/dist)` put −0.3515 in;
   the sim reads `pitch·π/2`, so the shot actually flew at **−0.5519 rad** — the probe's spawned
   bullet `dir = [0.0, −0.5245, −0.8514]` is `sin(−0.3515·π/2) = −0.5245` exactly.
2. **It aimed at y 0.9 — above the pane.** Pane #48's box is y 0..0.499, x 3.176..3.667, z
   −68.903..−68.371 [D the placement's bounds]; a ray crossing the pane's plane at y 0.898 misses
   the box top by 0.4 regardless of stream, spread or eye height.
3. **It hit anyway on rain-free trees because of Bond's settle height.** With the mis-aimed steeper
   pitch, the shot from the head lands inside the box only when the head sits ≈2.0-2.5 high —
   Bond mid-settle at the stand_at height (1.4 + 0.701 = 2.1 → the ray passes y 0.102 at the pane,
   inside). Where the game-true rain is present, the extra per-tick env draws [M `Env_Update`
   draws `Rand_Rand(20000)` + the wind/respawn draws from the shared generator every tick, and
   `Drones::tick` runs them first — env.rs module header; M 0x17bc88/0x18c580] shift the stream,
   the script timing around the settle shifts with it, and by the shot Bond has landed (pos y
   1.1015, head 1.8025): the same ray passes y −0.196 at the pane, below the box, and the floor
   eats the round 3.4 units out (the probe: the one bullet spawned, died at t=1, the pane's health
   never moved). That is the whole "stream-shift" mechanism for this red: not a damage roll, not
   the spread — the generator moved a **geometry accident**.

## The fix (test-only; the assert is stronger than before)

`crates/nf-view/tests/interactables_07000041_live.rs::the_glass_pan_shatters_under_the_real_fire`
now pins the rule on the real fire path:

- sister first: the pane stands while unshot, and its authored health is 0 [D; M 0x12c230's parse];
- the P2K's contact damage is 3.5 [L ee_snapshot 0x2bf368 +0x0c];
- Bond stands at the same spot, then aims **from `Bond::head`** (the fire's own spawn origin [M
  0x1a4bf0:226-245]) at the pane's **box centre** (3.42, 0.25, −68.64), writing the atan pitch
  through `HALF_PI` [M RotMatrixX 0x1e46a0] — the ray passes through the box for ANY settle height,
  so the stream can no longer move the outcome;
- hold fire: the first round spawns on the press tick (+0x44 = 0 [M 0x1a5460:688-690]) and the
  health-0 pane dies on the first landed hit [M 0x12bfd8:44-53]. The 40-tick window is
  choreography, not a pin;
- after the kill: `health == −3.5` — the kill is the first landed round's subtract [M 0x12bfd8:44-53].

Nothing was weakened: the old assert (`shot`) is kept and two rule asserts (health 0, the −3.5
accounting) were added around it. The test binary is green: **14 passed, 0 failed, 2 ignored**
(direct run, the merged tree; re-run on the final tree after the batch-1025l merge — see the suite
appendix).

## What ran

- `cargo test --offline --release -p nf-view --test interactables_07000041_live -- --test-threads=1`
  (the whole 16-test binary): 14 passed / 0 failed / 2 ignored, 24.9 s — on the rain-4+master tree
  before the batch-1025l merge, and the fixed test alone passed twice more (1.76 s, 1.67 s).
- The full suite: `tools/test/suite.py` daemon — see the appendix for what landed in the window.

## What's left

- The suite's **total line did not land in the 2-hour window** (the daemon restarted on the final
  tree at 19:28 UTC after the batch-1025l merge; the run phase cannot cover ~940 binaries in the
  remaining ~50 minutes on this 2-CPU sandbox). The appendix records the partial coverage and its
  failure classes; the next session should re-read `suite.out` (and `/home/z/suite.log`) before
  pushing further.
- If the main agent wants belt-and-braces on "master was green by accident": build the pre-fix test
  on a master worktree (no rain fix) and observe the pass with Bond mid-settle — [S]-grade
  corroboration; the rule fix above does not depend on it.

## Live checks for the main agent

Nothing visual is claimed — the fix is test choreography + rule asserts. One optional check, only
if the accident story is wanted on live hardware:

1. PCSX2, 07000041, fresh boot: stand Bond at (3.42, 1.4, −64.0), aim the P2K crosshair at the
   low glass panel ahead (the pane #48 placement), fire once — EXPECT the pane to shatter on the
   first hit (health 0 [D]; the game's own contact damage 3.5 [L]). gdbwatch on the pane's
   `tag+0x04` (health): EXPECT 0.0 → −3.5 on the hit tick, then obj+0xFE bit 0 set [M 0x12b970:60].

## Usage

not visible to me (this session's environment shows no token, credit, context-window or cost
figures; nothing to copy).
