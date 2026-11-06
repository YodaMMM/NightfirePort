# guard-batch-repin

Branch from `origin/s/guard-batch` 1c0249b79 (the per-goal arrival radii + guard rounds through
glass, both decompile-reviewed; suite there 4399 passed / 8 failed). Pushed as
`s/guard-batch-repin` with the WORKLOG entry; no [L] pin touched; NO code change landed this
session — the two non-walk reds were investigated and one fully diagnosed (below), the walks are
untouched and listed at the end.

## The astronaut red — fully diagnosed, no game-code fault found

`enemy_accuracy_live astronauts_hit_like_the_chain` fails at "diff 2 d 5: 1 shots in 10 s
(interval 180)" (assert wants 3..=4).

- **The driver's hypothesis is excluded**: the astronaut's `+0xc4 = 0x1d` change does NOT alter
  his firing. Verified by bisect with a real rebuild (1m27): reverting BOTH new lines
  (`self.attack_mode = self.dtype_of(d, self.dtype, 0)` and the kind-0x18
  `attack_mode = 0x1D`) leaves the test red identically. The change's own citations hold:
  [M NDrone2_GetDroneTypeAttackTypeFriend 0x14cbd8:22-23 writes `+0xc4 = GetDTYPENEW(+0xc7,0)`;
  DefaultInit 0x14b300:725 writes `+0xc4 = 0x1d` in the kind-0x18 arm — read in the decomp].
- **The actual cause is the branch's fire-path glass pass** (drone.rs `fire_weapon`'s
  `glass_pass: weapon_data+0x6c & 0x800 == 0`): reverting ONLY that one site turns the whole
  test green (verified bisect, 30 s run). The glass law itself matches the decomp — I verified
  [M Collide_Update 0x1ea398:222-229: when the query's HITTEST+0x9a carries bit 0x20, a contact
  whose surface byte is EXACTLY 0x10 is force-passed (bVar3 = 1) — plus the standing glist-0x40 /
  surface-0x40 / 0x80-roll law]. Two further candidate refinements were tried and are NOT the
  fix: the raw-byte equality variant (`mats[k] == 0x10` instead of `& 0x3f == 0x10`, per
  0x1ea398:225's `!= '\x10'`) and clearing the harness's stray rounds after the settle.
- **Mechanism**: weapon 0x6A is the 0x3b fuse family [M Bullet_init 0x12c5a0:199-211 —
  kind 0x6A → 14400 ticks], so the glass-passed rounds no longer die on the 0700001b station
  glass and fly (very long) through the harness's 4000-tick settle AND into the measured window;
  their DoTrails flyby draws [M Bullet_DoTrails 0x12d890 — Rand_Rand(5) within 3 u of the viewer]
  consume the SHARED RNG stream that the sight-acquisition rolls ride. Measured effect: at
  diff 2 d 5 the k=0 astronaut never acquires (dr.los false for the full 2400 ticks, unseen
  climbing to 2327; Bond is healthy at the window: health 100, substate 8) and k=1 acquires late
  and fires once. The pre-branch stream (glass-stopped rounds dying early) is what the test's
  3..=4 expectation was calibrated on.
- **Conclusion**: no game-code fault proven — the port's post-branch law is the better-matching
  one; the red is the harness's stream-calibrated expectation. The fix belongs in the harness
  (make `measure()`'s acquisition window stream-independent — e.g. a seeded per-case Rng reset or
  a longer acquisition allowance), NOT in the glass law. Left to the next session with the
  bisect evidence above.

## The kiko red — reproduced, not diagnosed

`kiko_button_real_walk_live the_07000016_real_walk_reaches_the_kiko_death_button_and_the_cinema_clears_the_gate`
fails at kiko_button_real_walk_live.rs:319 — `the pit squad member #14 dies to Bond's fire`
(the walker's `shoot` budget expires on the nearest alive death-channel 23..=26 member). Not
diagnosed: no bisect run yet (the astronaut consumed the budget). Candidate: the branch's
guard movement (goal_arrival) moving the pit member out of the shoot window, or the same
stream-shift class as the astronaut.

## What's left

1. Fix the astronaut harness calibration (stream-independent measure window) — the analysis and
   bisect evidence above.
2. Diagnose + fix the kiko pit-squad shoot red.
3. The six walks, test-side, with each binary run whole and quoted: walkreal 07000016
   real_damage, 01 00agent, 03 real_damage, 13 real_damage, 08 00agent (the shoot-out must kill
   every end pane), 06 real_damage. No [L] pin loosened.

## Live checks for the main agent

1. 0700001b diff 2, an astronaut 5 u from a standing Bond: gdbwatch on the sight-acquisition
   roll's RNG draws (DroneFunc_FirstSightState 0x1477c8's Rand call) and on the astronaut's
   +0x4f8/+0x500 during the first 40 s — does the real game's acquisition also stall past 40 s
   when Bond stands still (the port's post-glass-law stream says it can), or does he acquire
   within ~20 s? That settles whether the harness or the port's acquisition is off.
2. 0700001b: a plasma round fired at a glass pane with weapon 0x6A — confirm in the game the
   round passes the pane (the [M] 0x1ea398 law) and flies on; record HITTEST+0x9a at spawn
   (0x12f048) for the 0x6A record.

## Usage

No token, credit, cost or context-window figures are visible to me in this environment (no usage
meter is shown in the session UI or the tool outputs). not visible to me.
