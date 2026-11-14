# real-health-00agent-01-hall-5

Branch `s/real-health-00agent-01-hall-5`, cut from current origin/master 9182c25b4 with
origin/s/real-health-00agent-01-hall-4 (4be0f61f5) merged in (its docs/WORKLOG.md and
docs/reports/* dropped from the merge result; merge commit cb1d8dc2c). The task: hall-4's
measurements were taken on the old base 2265b403e; on the merged tree the 00 Agent walk died
EARLIER than master alone (t3546 vs master's t6054) — re-measure every lever and re-tune.

## What's done

The merged stream moved the shotguns and master's aim-stop root changed the fight completely:

- **#6 (0x5000036, 0x1c) now parks AT the door mouth** (5.7,6.3,14.7) — 6.6 u from the hold
  line — and the deep captain **#7 (0x5000038, 0x1c) at (8.3,6.3,17.3)**. Both trade through
  the open aperture; nobody crosses (master's `zero_root_live 0x01` aim-stop root: the guards
  stop dead to shoot). Every hall-4 dance lever is dead weight on this stream — the old base's
  #6 was a 12-hp pistol guard parked deep; the merged #6 is a shotgunner at the mouth.
- The old entry died at t3546: the return-fire fight answered #6's first aperture burst and
  HELD Bond standing in the mouth ray — 16 pellet hits in exactly two windows (t3433-34 and
  t3545-46, nothing else for 2900 ticks) ate the whole 55.0 ledger [L the run0/run1
  NF_DMG_TRACE traces, /tmp/h5_run*.txt on this sandbox].

The re-tuned door phase (all levers d3-gated; the d1/d2 walks byte-identical — their tests
pass unchanged):

1. **The quiet crouched entry** — the deep line FIRST with the door CLOSED (no LOS, no fight),
   then the crouched press (a new opt-in `Walker::use_door_crouch`: the close-in walk and the E
   edges run crouched; defaults false everywhere else), then the crouched home. 0 hits through
   the whole entry [L the run2/3 traces].
2. **The lure+hold skip at d3** — nobody crosses on this stream (hall-3's own sustained-lure
   ledger: the noise draws nobody, the sighted gate never clears [M drone.rs enemy_alerts]);
   the 300-tick lure cost the only resource the door phase has: time.
3. **The fight-direct entry** — the press runs from the meet-plate spot (`use_door`'s own
   close-in), no positioning legs: the old entry's ~430 ticks (the press + the swing + the home
   leg's 185-tick swing-avoidance wander) left only 73 ticks before #7's volley [L the
   stamp-A ledger].
4. **The lane fight** — the run3 lane grid (hall-3's read-only diagnostic, now running on the
   merged stream) mapped the south wall's SECOND opening: full-height crouched flight lanes
   from the step-area stands (7.5/7, 7.6-9.5) to BOTH shotgunners — the same lane their
   pellets use [M drone.rs:16465 the round's world trace]. The sight never clears from the
   south (the run4 health trace: no drone took damage through the whole door phase —
   `exposed_aim` None everywhere), so the fight taps on aim-OR-LANE with the corridor-fight's
   own probe, the stand picked by the live probe at Bond's own floor, and the **dy-0.9
   head-biased lane aim** (the measured spread lands 0.42-0.64 BELOW the aim point — the
   dy-0.9 aim rides the head box; the dy-0.6 arm hit part 53 at 2.1/hit [L run15], the dy-1.5
   arm flew over [L run14]).

**Measured result: #6 — a NAMED guard, one of the three AND inputs (death channels
0x11/0x12/0x13) — DIED AT TICK 3164 to ONE tap (12.0 → -16.0, the 28.0 part-5 head record
[M NDrone2_HitDamage 0x145558 head ×8], bond 55.0 → 55.0).** One AND input fired for free,
through the real input path, from the level start with no teleports.

## The remaining wall (the next lever, in the ignore message)

The deep captain #7's volley reaction: his first full volley lands **~91 ticks after the first
tap noise** (7×7.2+4.6 = 55.0 in ONE frame — the whole ledger [L the run2/13/16/18 traces]),
and the pistol's 16-tick semi-auto refire gives ~5 tap attempts in that window (ONE landed
part-4 hit measured). The walk dies at t3255 at (7.06,9.59) with #6 dead — one named kill on
the board; master alone dies at t6054 at (7.40,10.55) with none. The walk does NOT yet reach
master's death spot; the kill race is measured to the tick. Concrete next levers:

- (a) arm the SMG DURING the entry's quiet waits (before the press, not inside the fight
  window — the SMG rate arm was TRIED inside the window and measured A LOSS: the 41-tick
  holster/draw ate the kill outright [L run17]);
- (b) the head-box rate at #7 (the pre-aim/bloom law above);
- (c) the fight order (the weapon-keyed sort exists; the race clock starts at the FIRST tap,
  so the volley owner needs to be the first tap's target).

## How it was verified

- `walkreal_07000001_live` (whole binary, `--test-threads=1`): **7 passed, 0 failed,
  1 ignored** (the ignore is the d3 00 Agent ledger, rewritten to the hall-5 merged-tree
  state) — the d1 real-damage and d2 Agent sisters pass unchanged (every change is
  difficulty-gated; their streams are byte-identical through the None-defaulted/false-defaulted
  opt-ins) [L 71.7 s]. Re-run green after the debug stamps were removed.
- `interactables_07000001_live`: **10 passed, 0 failed** [L 30.5 s].
- 19 instrumented runs ([L] /tmp/h5_run0-18.txt on this sandbox, all through the real input
  path): the run0/1 DMG traces, the run2 entry ledger, the run3 lane grid, the run4 health
  trace, the run11 floor-probe wedge, the run13/14/15/16 dy-scan ledger, the run17 SMG loss,
  the run18 final shape.

## What's left

- The #7 volley-reaction race (the levers above), then the escort's west route, the slab
  ambush and the plate AND-touch — hall-4's post-AND route is untouched by this task.
- The full 4-crate suite did not run on the final tree (the sandbox memory/disk limits noted
  in the earlier reports); the two whole level batteries above are the coverage.

## Live checks for the main agent (PCSX2, fresh boot)

1. The second opening: with the hall door open, confirm a crouched player at the meet-room
   step (≈7.0, y-step, 8.5-9.5) can hit #7 at (8.3, 17.3) while #7's pellets also reach him —
   i.e. the lane is genuinely bidirectional through the south structure's second opening, and
   note whether the real game's spread lands low like the port's.
2. #7's volley reaction timing: from the first player shot fired into the hall, measure the
   ticks until #7's first full 8-pellet volley — the port measures ~91; the real game's
   reaction time gates whether the kill race is winnable at all on disc behavior.

## Usage

Token/credit/context figures: `not visible to me` — this environment exposes no token, cost
or context-window readout to the worker. What was visible: the task message's clock (sandbox
expiry ~04:25 UTC; this report written ~04:05 UTC), `git` hashes, test outputs and process
exit codes. No input/output token counters, no credit balance, no dollar cost.

## ntfy

Sent at the finish step (the record appended below only if a fallback was needed).
