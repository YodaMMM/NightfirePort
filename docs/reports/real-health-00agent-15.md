# real-health-00agent-15 — report (2026-11-07)

Branch `s/real-health-00agent-15`, cut from origin/master 22d5836b7. The 15 00 Agent sister
(`walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage_00agent`) stays
**IGNORED with a measured ledger** — the round's deliverable per the task's "or ignored with a
measured reason" — and the round's levers are coded in the test body (d3-gated blocks; the
d1/d2 paths byte-identical, the diff is one test file and no lib code).

## What's done

1. **The #526 grenade lever is implemented** (the task's named lever), d3-gated, in
   `crates/nf-view/tests/walkreal_07000015_live.rs`: after the door[32] guard (placement 722)
   dies, Bond walks over his drops — the AR 0x1a AND a kind-1 pile of its grenade launcher 0x1b,
   a full shot budget [M Pickup_Handler 0x18e650's AR drop arm; DroneWeap_DropWeapon 0x17a2a0] —
   opens door[32] (`use_door`), draws the launcher, and lobs at #526 from the threshold stand
   (26.5, 41.4, 71.2), **15.2 u off the mine**: past the grenade's radius 8 AND past the blast
   sweep's (int) truncation floor at (int)d ≥ 15 [M Explode_Propagate 0x17c518,
   0x17c61c-0x17c628] — both blasts read zero there, vs the rim shot's 69.3 raw at the d3 pain
   row 1.0 [M bullet.rs damage_factor 0x14..=0x16]. The law that makes it work is pinned this
   round: the round's fuse is **TIME, 210 ticks** [M Bullet_init 0x12c5a0's default; grenade.rs's
   fuse arm — "count ticks, not distance"], and **a bounce NEVER explodes** [M Bullet_CollisionHandler
   0x12ce80] — so the lob bounces down the lane and rests mid-lane by the rim, where the pop's
   case-0x4B entry chains #526 (the guard's own round popped 7.7 u from the sphere and chained it
   [L the d1 ledger]). The rim walk + rim shot skip when the mine is already gone — a read-only
   gate; the d1/d2 runs keep their exact legs. **Never reached in a run: every path dies at the
   crossing or the planks first** (below). The assert pair (thrown from > 8 u; the chain costs
   ≤ 6.0) is in place for the next round's green run.
2. **The plate sneak** (the burst lever, found this round): the d3 roster's #15 burst (63.0 in 9
   hits [L the 12-13-14-15 ledger, reproduced exactly on this master at tick 867]) rides the
   touch plate at (80.8, 41.8, 0.1) (out 0xf6). The plate's touch box, read from the port's own
   trigger bounds [L]: **x 80.31..81.31, z −1.18..1.35** — the gantry's west wall (x ≈ 81.7)
   carries its door aperture around z −1.0, whose south half lies below the plate's −1.18 edge.
   The d3 run now hugs the south rail: (83.0, −1.4) → through the aperture's south half
   (80.0, −1.6) → onto the lane. The measured effect: the close fight killed #15 for **21.0 in 3
   hits** (vs 63.0) — the approach beats the burst.
3. **The dying-grenade trap is measured and named**: at the 3.6 u point-blank fight the
   grenadier's own grenade (weapon 0x0's blast arm) popped at Bond's feet for **79.0, 75 ticks
   AFTER the kill landed** (dead at 992, total 100.0). A head-shot finisher (`shoot_point` on his
   head, one head-hunted round drops a 20-hp guard [M NDrone2_HitDamage 0x145558 ×
   DroneDamage_Head]) was added at the aperture's west side (the line to his advanced spot
   (76.7, 41.4, −4.6) is 4.5 u); in its one run the aperture leg's stop drifted into the plate
   box (dead at 1046 at (80.66, −0.96), inside the box, before the finisher's kill) — the
   finisher stays in the code with the leg-stop note.

## The measured ledger (why the sister stays ignored)

Every path tried this round exceeds 100 health **before #526**:

| cost | value | law |
|---|---|---|
| the crossing fight vs #15 | 63.0 in 9 hits (the 8-10 u lane fight, no grenade) or 21.0 + the 79.0 dying grenade (the 3.6 u sneak fight) | [L both runs, this round] |
| #462's own-shot blast | 10.7 | 80·(1−13/15) at the ≤ 14 u shot law, the d3 row 1.0 [M 0x17c518's truncation] |
| the plank pair #7/#8 | 26.3 (19.3 #8 + 7.0 #7 at frames 3605-3610; death at 3610 at (35.11, 41.34, 25.20)) | [L the 12-13-14-15 ledger, reproduced] |

The d3 stand at door[38]-north (34.6, 16.0) **can never fire**: both guard lines are walled by
door[39] from there (the lines to (28.63, 35.51) and (42.71, 35.49) cross the door[39] wall
plane at x 32.6 / 37.2, outside its aperture) — geometry, not aim.

## The next levers (in the ignore message, in the code)

1. The #526 lob (implemented, above) — removes 69.3 → ~0.
2. The sneak + finisher — the burst to ~7-21 **if** the finisher's kill lands before the throw;
   the leg stop needs one more nudge south (the stop circle must clear the plate box: target
   (79.0, 41.4, −2.6) instead of (80.0, −1.6)).
3. The plank stand moved to door[39]'s north side (34.6, 41.4, 28.0) — the lines are clear at
   9.6/12.9 u and the fights' own push closes to 8 u; run it the moment door[39] opens.
4. The #462 shot at (int)d ≥ 14 → 5.33 (halves the fixed 10.7).
5. A live check would settle the throw gate: does the 0x50000b2 grenadier AI throw at > 5 u?

## How it was verified

- The d3 sister ran five times this round (release, real input, no top-ups): the 12-13-14-15
  ledger reproduced exactly (63.0 at 867, death at 3610 to the planks), the sneak run (21.0 +
  the 79.0 grenade, dead at 992), the finisher run (dead at 1046 in the plate box). Each run's
  full last-hits ledger is quoted in the test's ignore message and the WORKLOG.
- The 0xf6 plate box is read from the port's own `Trigger.lo/hi` bounds [L the probe, 2026-11-07].
- **NOT run: the other walkreal binaries, the nf-game/nf-formats/nf-audio suites** — the window
  did not fit them. The diff is ONE test file (`crates/nf-view/tests/walkreal_07000015_live.rs`),
  no lib code, no shared-helper change; every behavioural addition sits behind
  `if difficulty >= 3` gates the d1/d2/d1-real calls never execute, and the one shared-path edit
  (the `mine526_gone` skip) is a read-only check that is false on every d1/d2 run observed.

## What's left

- One focused round: the finisher's leg-stop nudge, the plank stand at door[39] north, the #462
  shot at (int)d ≥ 14, then the #526 lob's green run — the budget closes at ~85-90 by the ledger
  arithmetic if the crossing fight lands ≤ 21 with no grenade.
- Every `walkreal_*_live` + the full suite total line (rule 5) on the round that lands green.

## Live checks for the main agent

1. **07000015, 00 Agent, the grenadier throw gate**: at the crossing, stand 6-8 u off the
   0x50000b2 guard and let the fight run — a gdbwatch on his throw action would pin the distance
   gate (measured: throws at 3.6 u, does not throw at 8-10 u). If the gate is ≥ 5 u, the sneak +
   finisher path is safe at 6 u and the budget closes.
2. **07000015, the #462 shot distance**: the nav lane's line to (46.8, 41.556, 0.6) clears at
   14.0 u — a `--bond 60.8,41.4,0.6` render looking west would show whether 14.5-15.0 u is
   reachable on the lane (the (int) law makes 15.0+ free and 14.x half-price).

## Usage

not visible to me — no token, credit, cost or context-window figures are exposed anywhere in
this sandbox (no status bar, no session summary, no API dashboard access).
