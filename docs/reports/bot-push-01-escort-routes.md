# bot-push-01-escort-routes — the 01 escort walks re-tuned for the faithful push

## What's done

Branch `s/bot-push-01-escort-routes` cut from origin/master b62a8907; origin/s/bot-push-per-bot-gates
(5392f79d) merged first — WORKLOG add/add conflict resolved by keeping both entries.

**The re-tunes (all real input; no teleports after the spawn):**

1. `follow()` steps Bond OUT of the lead's way-band (`crates/nf-view/tests/common/walker.rs`).
   The faithful push added the half the old walk never saw: the escort himself closes the gap —
   Bond holds at the 4.0 follow distance, the contact walks INTO him, PlayerInWay engages
   (route_blocked && dist <= 3.0 [M 0x15e4f0's PlayerInWay arm]) and the lead parks with Bond
   standing 2.9 away (the real-damage walk's node-15/37 stall at (7.0, 6.3, 9.5)). The follow now
   backs off (forward −0.7) when the drone closes inside 3.4, and inside 4.2 while he sits in
   PlayerInWay — the release [M 0x15ea28 case 3] reads only Bond's distance, so the step-back is
   the whole fix.
2. The main walk's PlayerInWay recovery no longer throws Bond 20 u east through the hall crowd
   (`back = cp + (cp−plate)·0.5`); it steps 4.5 behind him, opposite his route target — the same
   behind-the-route point the descent follow uses.
3. The plate sister (`the_plate_refuses_bond_alone`) shoots lane blockers: the hall crowd (five
   team-1 guards around the door-2 gap) plugged the old no-push trajectory's lane into 28 ×
   900-tick bot-orbit cycles. It now walks with the opt-in `body_block_response` (the same arm the
   02 real-health walk uses) and `prefer_head`. Its legs are lenient and self-heal (a wedged leg
   hands the next leg a fresh nav re-aim — the fight phases move the guards run to run), with a
   proactive resupply between legs (clip+spare < 10) and a hard assert that Bond actually REACHES
   the plate (d < 2.0) so the refuse-assert can never pass vacuously.
4. The hall fights chase the hunter: the named guards HUNT — guard 7 fled 14.8 u while the
   approach walked to his stale spot, and the fight opened with the captain unexposed; two clips
   later ch 0x99 had never risen. All three fight loops (the opening clear, both named-guard
   loops, the `clear_around` closure) now re-aim at the guard's live position until the fight
   opens inside 7.0 with LOS.
5. The follow kites melee huggers: a chasing guard inside 2.6 trades 0.75-damage punches
   ([M Player_WeaponInitBullet's 1.8 fists range] — the disarmed guards rush on fists) while the
   old follow stood at its 4.0 hold; Bond now backs (forward −1.0) while return_fire works.
6. The final plate approach walks AROUND the parked contact: he parks on the plate's south half
   (route node 36 is inside the volume [D the nav dump]) and the push holds Bond ~1.4 off his
   spine, so the straight SE→centre line stopped 0.9 short of the plate's east edge at
   (−31.47, 12.45). The walk now comes in at the plate's north half (z ≈ 16.8, the volume is
   6.5 deep), then steps to the centre.

**Results (release, --test-threads=1, slim ISO):**

- `walkreal_07000001_live`: **5 passed, 1 failed** — `following_mayhew_down`,
  `the_exit_fade_starts`, `the_hall_guards_wait`, `the_plate_refuses_bond_alone`,
  `walk_from_the_spawn_to_the_exit_with_the_contact` green;
  `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage` still dies (see What's left).
- `exit_aim_01_live`: 1/1. `flank_fall_repro`: 2/2 (the un-ignored courtyard flank + the relaxed
  lip press). `walkreal_07000003_live`: 3/4 — `walk_the_full_route_and_complete_every_objective`
  fails on the merged master tree (see What's left; not from this diff — 03 never calls
  `follow()` and the 01 edits are file-local).

## How Bond's movement changes when he brushes a guard (for the app feel-check)

- **The push runs every tick, geometry only, no ground preservation** [M Collide_Jointy
  0x1ec180:229-281]. Overlap discharges the tick it opens — a fraction of a unit per tick while
  the overlap persists — never the old airborne-tick dump (two guards, ~0.74 u in one tick, the
  pre-gates bug).
- **Brushing a standing guard lifts Bond for a few ticks**: measured 5 ticks (t13090-13095, apex
  y 6.49) against the 07000003 courtyard guard at (12.74, −19.23) with the closest-segment
  distance 0.65 (the pinned lip-press test). The pin now bounds ungrounded runs at 20 ticks and
  requires the landing.
- **A standing capsule holds Bond out at ~1.4 u spine-to-spine** (r 0.55 + 0.4 + margin):
  measured at the 01 plate, where the parked contact (x −32.9) held Bond at x −31.47.
- **Walking a crowded lane** (the sister's orbit logs) drifts Bond ~0.1 in y while the crowd
  shoves him across the doorway (7.37→7.51 x, 8.99→9.31 z across 900-tick windows) — the push
  slides him along the blocker's tangent rather than through it.

## What's left

1. **`walk_from_the_spawn_to_the_exit_with_the_contact_real_damage`** — Bond dies at tick 7805 at
   (−29.25, 6.43, 13.62) (the stairwell head), 91 hits for 100.0. The NF_RF_TRACE run pins the
   shape: he reaches the west corridor at 41.5 health, fights guards 3/14/15 through it, and
   enters the fight vs #17 at **2.2 hp** — the death is an attrition economy, not a route stall.
   Two unseen cross-map snipers take ~20 hp for free (`shooters [3]`/`[12] none visible` phases;
   #9 10.0 in 2 hits at d 20.8, #11 10.0 in 1 hit at d 35.6), and the disarmed rushers hug for
   0.75/punch every ~6 ticks. Next session: fight the snipers while they are still far (the
   named-loop chase pattern applies to the corridor), and/or loot armour on the route; the
   kite-in-follow alone cannot save 2.2 hp.
2. **`walkreal_07000003_live::walk_the_full_route_and_complete_every_objective`** — pre-existing
   on the merged master tree (b62a8907): the servant's captor fight ran dry at +1497
   ("servant 0's captor: out of ammo"), and the assert `servant 0 left WAIT_SWITCH/HOSTAGE, got
   0xa` fired at walkreal_07000003_live.rs:273. The gates branch reported 4/4 on its older base;
   the head-look merge's corrected head box moves the fight timelines (its own comment warns of
   exactly this). Not from this branch's diff: `follow()` is called only by the 01 file, and the
   03 test's path never reaches it. Next session: re-tune the captor fight's ammo (the 01
   chase/resupply pattern), same method.
3. **The full walkreal_* sweep and the full suite did not run** — the targeted runs above
   (07000001 ×3 files, 07000003, exit_aim_01, flank_fall) consumed the window; the full suite's
   build also needs the pre-pruned-target treatment on this 10 GB disk (the gates report's
   lesson: three starts, each hit `No space left on device` before the deadline). No new
   full-suite total exists yet; the driver should treat this branch's green set as the targeted
   files only.
4. **Live checks for the main agent** (none run — no PCSX2 here):
   - 07000001: Bond brushing the contact (skin 0x4F, kind 9) while he leads — expect a small
     per-tick push (no ground gate), and the contact parking in PlayerInWay (0x1e) while Bond
     stands inside 3.0, resuming as Bond steps past 3.0 ([M 0x15ea28 case 3]).
   - 07000001 west corridor: watch drone +0x4f8 on the disarmed rushers (weapon 0x0 after a
     drop) — expect bit 0x20 SET while alive (they push Bond and take his bullets like any
     guard).
   - The 01 plate: Bond's x when pressed toward the parked contact — expect the ~1.4 hold-out
     (0.55 + 0.4 + margin), matching the app's feel-check above.

## Usage

not visible to me
