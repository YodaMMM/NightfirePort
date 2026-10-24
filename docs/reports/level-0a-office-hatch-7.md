# level-0a-office-hatch-7 — the spawn settle without its five master-tree regressions

Branch `s/level-0a-office-hatch-7`, started from `origin/s/level-0a-office-hatch-6` (not merged),
merged `origin/master` early (915b4923). All five named failures reproduced on the merged tree
(each passes with `arm_spawn_settle()` disabled, confirming attribution), then four of the five
were fixed this round.

## What was done

### 1. The five regressions — root-caused with a per-scenario probe

`crates/nf-view/examples/probe_hatch7_regress.rs` (committed) traces each scenario per tick; run
with the arm line flipped to separate the held-tick effects from the rest shift.

**The mechanism class (4 of 5): the broad whole-tick skip.** hatch-6's settle held tick did
`return` out of `physics_tick` after the gravity add — skipping the move/look reads, the crouch
read, and the whole collision tail including the capsule PUSH. The game's observable (the
ee_start root hold) pins only the gravity accumulation, the un-integrated root, and the held-off
grounded zeroing — the broad skip was a port choice, and it swallowed things the game's player
does do in those ticks:

- **`headless_app_harness::the_crouch_option_picks_the_stand_condition`** — the test's first
  crouch press lands on tick 1, inside the hold; the early return is before
  `Player_HandleJump`'s crouch read [M Player_SSCrouch 0x1924c0's port arm], so the press was
  swallowed and the toggle leg failed.
- **`fix_08_collision2_live::the_01_seam_scoped_pass_matches_the_flat_one`** — the direct
  `nf_view::load` starts ENABLED (`waited 0 disabled false`, probe), so the test's teleport
  ([-2.62, 6.468, -1.12]) lands inside the hold window; the held ticks resolved no wall push and
  the read happened with the capsule pressed into the seam wall (flat [-0.0258, ·, -0.0043] vs
  scoped [-0.0260, ·, -0.00043]) — a position the game's collision never freezes at. With the
  push running on the held ticks the wall resolves the bond out to x -2.6459527 (probe, both
  trees) and the read matches flat≈scoped≈0.
- **`interact_07000009_live::standing_short_leaves_the_armour`** — the probe: the bond placed
  short of armour #270 stands static through the held ticks (the overlap with the local step
  geometry unresolved), then the RELEASE tick's push ejected it +0.365/+0.254/+0.254 toward the
  pickup — inside the take radius by tick 11 (armour 50.0, gone true). The game's player never
  accumulates wall overlap.
- **`fire_modes3_live::torpedo_is_homing_slow_and_cameraless`** — NOT the hold: the old aim
  (`aim_down(10, -0.5)`) clamps pitch to -1.0 and fires straight INTO the courtyard floor under
  the spawn; with the full weapon catalog the round dies on the floor between read ticks 2 and 3
  (probe: alive 0.0362 at t2, list empty at t3). The game's own 05 rest 24.9633007 [L ee_start_05]
  sits 0.0141 below the port's floor sampling and 0.0032 below the port's settled spawn root —
  the straight-down shot dies within the read window at EVERY one of these roots, so the
  alive-after-3-ticks expectation was geometry-fragile, not game-backed. TEST FIXED (evidence
  cited in the test): `aim_down(&mut scene, 2, -0.25)` = pitch -0.5; every pinned assertion (the
  homing/quarter-speed/no-camera bits [L +0x6C], the ramp [M Bullet_DoTrails 0x12d890:134-138],
  the quarter start [M Bullet_init 0x12c5a0:188-190]) is unchanged.

**The fix (commit 1ed1da93, [M]-flavoured narrowing):** the held tick keeps the unconditional
gravity add [M 0x194df0:107], skips the vel→pos integration (the root holds [L ee_start_0700000a]),
and holds off the grounded tail's velocity zeroing (the accumulation IS the observable) — but now
RUNS the move/look/crouch/jump reads and the capsule push resolution (`pos += push`
[M Player_CollisionHandler 0x1949a8:136-140]). The three grounded-tail zeroing sites (head-bump,
land, rest) are gated on the held flag. The gate itself stays `// [S] unverified` — not widened,
not narrowed beyond what the observable pins.

**Verified:** the [L] rest pin `the_spawn_height_matches_the_game_snapshot` still passes
bit-exact (walkreal_0700000a_live 4/0/3), the 05 truck ride stays green (walkreal_07000005_live
6/0/2, fix_truck_live 2/0), and the four tests above pass. The bathhouse is the fifth and stays
red (below).

### 2. `bathhouse_shout_mixer_live::the_bunker_trio_shout_reaches_the_mixer_with_the_ambients_live` — still red, mechanism narrowed

Fails bit-identically before and after the narrowing (`shout id 115 reached the mixer silent
(l=0.40999818, r=0) at tick 1`) — so the divergence is NOT the held-tick behaviour; it is the
boot rest itself: the settle rest 1.0891 [L ee_start_0700000a] sits ~0.008 below master's
port rest (~1.0973, grounded at tick 1 with the velocity zeroed), and the mixer's stream pan
reads the listener (the camera follows the bond's head) — the lower boot camera shifts the
source-to-listener geometry across a marginal right-channel clamp. Note the GAME boots at the
LOWER rest (1.0891 is the game's own [L] value), so if the game's own pan at this route is also
r=0 the test pins master's spawn artefact; if the game's pan is r>0 at 1.0891 the port's panner
y-handling diverges. Deciding needs the live read below — I did not touch the test.

### 3. Not reached: the hall glide / corner / main walk

The window went to the five regressions (the task's item 1); item 2's triangle dumps and
per-tick contact logs did not start. The sisters' notes from hatch-6 stand unchanged (all three
still ignored on this branch, notes updated by nothing this round).

## What was run

Foreground chunks (`--test-threads=1`, NF_SILENT=1, release) on the final tree (merged with
origin/master): the five named tests (4 pass / bathhouse fails as described), walkreal_0700000a
4/0/3, walkreal_07000005 6/0/2, fix_truck 2/0, keyboard_0700000a 1/0, hatch_crouch 8/0,
drop_stand 5/0, level_carry 4/0, fix_world_gravity 1/0 — **31 passed, 0 failed (excluding the
bathhouse), 5 ignored**. The full fast-runner sweep did NOT fit the window after the
reproduction/probe work — the remaining families are untested on the narrowed hold this round.

Crates touched: nf-game (player.rs), nf-view (tests/fire_modes3_live.rs, the probe example).

## Live checks for the main agent

1. **The bathhouse pan** (settles the fifth test): fresh boot 07000002, tunnel route onto the
   bunker plate, watch the shout's stream Play mixer gains (the port's `stream_play` (l, r),
   tick-1 shout id 115) — read the game's (l, r) for the same shout and the camera/bond y at
   that tick. Expected if the game's pan has r>0: the port's panner y-handling diverges (then
   the fix is the panner, not the settle). If the game's r=0 too: the test's both-channels pin
   is master-spawn-backed and should be re-pinned to the game's own values.
2. **The settle gate** (hatch-6's live check 2 stands): fresh boot 0700000a, watch
   `Player_Update` 0x193600 / `Player_Collision` 0x194df0 for the first 8 frames after the
   placement — which parts run during the held ticks (does the handler's push phase run? does
   the crouch read respond?). This upgrades the narrowed model's `// [S] unverified` gate.
3. The doorway base climb (hatch-6's live check 4 stands) — the glide/corner shape.

## What's left

- The bathhouse pan decision (live check 1) and its fix on whichever side the read lands.
- The full fast-runner sweep on the narrowed hold (every level's first 4 ticks changed shape).
- The 0a hall glide, the corner, and `walk_office_floor_qworm_keyboard_hatch` (task item 2) —
  untouched; the sisters stay ignored with their hatch-6 notes.

## Usage

not visible to me
