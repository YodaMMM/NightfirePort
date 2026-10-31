# level-0a-bridge-reach — report

Branch `s/level-0a-bridge-reach` (from origin/master bf6b5a2a, batch-1031g). Task: 0a reach the
bridge on real input — delete the two REACH teleports in `walkreal_0700000a_live` and walk the
game's route to each spot.

## What's done

Both named teleports are gone, replaced with walks through the player's own movement
(`Wp`/`walk_to` real input only):

1. **The mouth bridge's west end (−11.35, 1.12, −18.6)** — was `bond_stands_at_the_mouth_bridge_edge`'s
   REACH teleport (old line ~884/888). Now: the test walks `walk_to_security_keyboard()` (the main
   walk's proven prologue: the arrival plate, the cab sweep, the Q-Worm PC loop + the REAL install,
   the walk-back, the REAL keyboard use, the muxes) and then the main walk's 23 ring legs verbatim
   to the keypad's wall stand (−11.9, −17.5), and walks south-east onto the bridge strip
   (`walk_to(−11.35, 1.12, −18.6).r(0.3)`), then the mid span. The strip's west edge x −11.43 and
   the wall's west face x −10.70 [D] are never crossed; the closed door #724's leaf is never
   touched — the bridge's west half is pocket-side floor.

2. **The hall line (−8.0, 1.12, 16.29)** — was `the_pick_ref_accumulator_tracks_the_game_ref`'s
   probe-placement write (old line ~1005). Now: the SS cab sweep waited out, out of the lift, the
   lobby south, the through-line THROUGH the doorway into the west hall (−10.0, 0.4, 16.29; the
   glide leg, green on master — the jamb slide around the base pieces at z ≈ 16.278 [D port world
   tris] is part of the through-line), then back east along the hall to the spot
   (`walk_to(−8.0, 1.12, 16.29).r(0.3)`), then the yaw faced west with real turn input. The [L]
   constancy window and the wall-press window are unchanged and green.

## Route evidence found on the way (both pinned in the test comments)

- **The straight lobby approach to the hall line wedges where the game does not.** The diagonal
  (lobby south → the spot) runs z ≈ 16.16 at the jamb column's x — south of the base course — and
  the raw forward hold pins on the column's south-east corner (root −7.75, z slid 16.21 → 16.02,
  the pick_ref accumulator +0.1/tick = a wall contact every tick [L this round's failing run]). The
  game's route enters the hall THROUGH the doorway (the through-line); the spot is reached by
  walking back east along the hall line. This is a route fact, not a port bug: the through-line
  crosses clean on master (230 ticks, the documented slide).
- **The desk lane's civilian g8 patrols the lane itself** (loop (−14.8, 8.3) ↔ (−18.7, 5.4), rng 16,
  fov 1.57, react 0 [L the NF_ALARM_TRACE roster dump]). A fresh-state lane crossing meets him
  head-on (d 0.5, seen → challenge 0x3e → RunToAlarm → the press → fail 0x4000032 [L two trace
  runs this round]) and he cannot be cleared (team 3, protected). The main walk's deterministic
  timeline (the PC detour delays the crossing) keeps him out of sight the whole way — so the
  bridge test shares the main walk's bit-identical timeline to the keypad's wall and diverges only
  after it. A real player's answer on a fresh timeline would be timing the patrol; that is an
  open design choice for a future round, not pinned by any [M]/[D] evidence I have.
- The old "wedging mouth legs" note in the sister test's REACH comment was stale: the wedge it
  referred to was the pillar crossing, which the main walk re-routed south of the pillar; the ring
  is green end-to-end.

## Verification

- `walkreal_0700000a_live`: **7 passed / 0 failed / 2 ignored** (the 2 ignores are the pre-existing
  glide/corner ignores, untouched; master's totals for this binary were 7/0/2 too).
- `interactables_0700000a_live`: **7 passed / 0 failed / 0 ignored**.
- `walk_0700000a_live`: 3 passed / 0 failed (first run SIGKILLed — 3 GB box, memory pressure during
  the parallel build; green on the retry with `--test-threads=1`).
- `keyboard_0700000a_live`: 1 passed / 0 failed.
- The main walk `walk_office_floor_qworm_keyboard_hatch` re-run green (the shared timeline's source).
- Full suite (fast runner `tools/test/suite.py`, foreground chunks — the sandbox kills background
  jobs between tool calls, so the recommended `nohup` start was not viable here): **218 of 986 test
  binaries ran, 741 passed / 0 failed / 62 ignored, no FAILED/TIMEOUT/CRASH** before the sandbox
  window closed. Crates built for the suite: nf-game, nf-formats, nf-audio, nf-view (release,
  offline). The binaries my change touches are all in nf-view and all ran green.

## What's left

- The remaining ~768 suite binaries (the heavy walk/probe ones) did not fit the sandbox window;
  the next message should run the fast runner to completion on the final tree.
- The wall-press window in `the_pick_ref_accumulator_tracks_the_game_ref` still probe-places Bond
  at (−40.0, 1.12, 16.83) (the [L] window's own baseline; the third write, not one of the two
  named spots). The walked reach for it (hold forward west along the hall from the hall line) is
  the natural next step; the hall west of x ≈ −17 has no walked evidence yet.
- The two ignored tests (the west-hall glide settle; the corner crossing under NF_WALK_CLIP) stay
  ignored with their notes — the +0.0256 climb-settle mechanism remains unported.

## Live checks for the main agent

None required this round — the task's route findings are all [D]/[L]-pinned in the test comments.
Optional confirmation (would upgrade the g8 finding): a fresh-boot walk from the 0a spawn toward
the desk lane, watching whether the real game's civilian challenges a player who walks the lane
head-on (address: the g8 drone record at spawn (−14.8, 1.00, 8.3); expected: challenge at close
range, then RunToAlarm only if the player stays in sight).

## Usage

not visible to me
