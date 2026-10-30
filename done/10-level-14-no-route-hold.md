## Your task: `level-14-no-route-hold`, guards with no route hold still like the game

Master now has the A* door-link gate (locked doors block routes [M AINetwork_DoAStarPath 0x1571b0:262-271]) and the 0.5 mission-route arrival [M NDrone2_InitAIPath 0x153720:113]; `door_guards_locked_door_07000014_live` pins that 07000014's released guards no longer work the locked west door. One divergence remains there: in the game the guard holds COMBAT_NO_ROUTE 0x5f the whole time (17 live samples at 30-frame spacing, CUR 0x5f / PREV 0x5a, opponent pointer set, grace 0 [L 2026-10-30 PCSX2, Bond held in the generator hall after ch 0x87+0x88]); the port's guard (#25) cycles 0x5f -> STEP_AIM_LEFT 0x73 / STEP_AIM_RIGHT 0x74 / COMBAT_NO_SIGHT 0x5c every ~90 ticks. The game's NoRoute UPDATE [M NDrone2_DSTATE_CombatNoRoute 0x164c08 case 3] keeps the state when NDrone2_ChooseCombatMove returns 0, 0x67 or 0x6f, and only changes for other picks. Find why the port's ChooseCombatMove picks the step-aims here (no los, no seen) and what the game's returns [M NDrone2_ChooseCombatMove; cite lines], fix it, and extend the 07000014 test to pin: the guards at the door hold 0x5f. Run every *guard*/*drone*/*combat* binary and the walkreal binaries; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-no-route-hold`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-30T20:12Z

---
## Report (done, 2026-10-30T21:24Z)


## Report body (attached by the worker, 2026-10-30T21:24Z)

# level-14-no-route-hold — why the port's door guard cycles where the game holds

Branch `s/level-14-no-route-hold` (from origin/master 61c7e3ea). Task: in the game the 07000014
door guard holds COMBAT_NO_ROUTE 0x5f the whole time (17 live samples, 30-frame spacing, CUR
0x5f / PREV 0x5a, opponent set, grace 0 [L 2026-10-30 PCSX2 fresh boot]); the port's #25 cycles
0x5f → STEP_AIM_LEFT/RIGHT 0x73/0x74 → COMBAT_NO_SIGHT 0x5c. Find why the port's
ChooseCombatMove picks the step-aims, fix it, extend the 07000014 test to pin the hold.

## The root cause (found, documented, not fully landable in this window)

The game's NoRoute UPDATE (case 3 [M 0x164c08:36-62]) keeps the state exactly on ChooseCombatMove
picks 0 / 0x67 / 0x6f and SetStates anything else — the port's handler mirrors that keep-set, so
the divergence is in the PICK. The game never picks 0x73/0x74 at the door because **every Can*
probe of `NDrone2_ChooseCombatMove` gates on the +0x31C side bits** (`CanStepLeft` 0x14f640:50
tests `& 8` before anything else; the Right probe `& 0x10`; the same head on the dodge/strafe
pairs) — and those bits are stamped EVERY tick by **`DroneMove_NoBunching` 0x150ad0** (called
from `ControlSTANDARD` 0x1490f0:202-226, which also zeroes +0x31C/+0x314/+0x310 and stores the
scan's return byte into drone+0x17 — the `cVar1` of ChooseCombatMove 0x150c18:58): the two
guards stand ~2.4 apart at the door [L: (-51.6,41.2) / (-51.2,38.8)], each reads the other as
the +0x314 neighbor (the pairing radius is 3.0 [M Steer 0x14eba8:83 `+0x310 = 0x40400000`]),
the 8/0x10 pair lands on BOTH, every dodge/strafe/step probe refuses, and every pick cascades
to 0x67 (Rand(4) ≠ 0) or 0 — 0x5f holds.

The port side has TWO gaps, both evidence-backed in the test file:

1. **The port's NoBunching mirror is inert** (`NB_SCAN_LIVE = false` in drone.rs — a documented
   hold). The port's own 01 blocker note at the const is STALE: with the scan flipped on,
   `walkreal_07000001_live` passes 5/0 (68.9 s, this session) — the escort re-pin it waited for
   has happened. The flip is NOT landed here because the full-suite verdict could not be
   produced in this window (see below); it is a one-const flip.
2. **Even with the scan flipped on, the pairing never bites**, because only ONE released guard
   reaches the door in the port: #25 cycles at (-50.1, 41.7) while #18/#19/#20 stay at 0x38 by
   (-51.3, 32.4) / (-47.9, 33.5) / (-40.2, 25.2) — the game's two guards both walk to the door
   and stop 2.4 apart [L]. The release-to-door divergence (which guards answer ch 0x8C) must
   land first so the scan has a neighbor to pair.

Eliminated along the way, with evidence: the A*-boundary gate in the step probes (the port's
`nav.move_test` passes both lateral steps at the guard's spot — 1784 boundary walls, bounds on,
(1, None) [D probe in the test's trace]); the `DefaultInit` placement-param-26 movement lock
(all near-door guards have p[0x1a] = 0 [D the committed param26 probe]); the class-init
stationary (DroneType[0x00] init None); the vision gate mapping (+0x228 & 4 ↔ `seen` is right;
the live grace 0 confirms not-seen). A nav.move_test conjunct at the choice's pair probes was
tried ([M 0x14f640:92] the game's probe IS a MoveTest) and **PULLED**: it regressed
`walkreal_07000014_live`'s corridor door-wait leg (6/0 → 5/1, reproduced; master's drone.rs
6/0); the pulled note cites the whole chain at the choice site.

## What landed

- `crates/nf-game/src/drone.rs`: the evidence note at the pair probe (the root-cause chain + why
  the conjunct was pulled); the `prop_value` test accessor (`behaviour_util_getProperty`
  0x1c7d98 through the tables without a `Data`).
- `crates/nf-view/tests/door_guards_locked_door_07000014_live.rs`: the released-guards test now
  records the states sampled at the door (evidence print) and the released-guard end-state dump
  (the one-guard divergence); **the hold pinned as `the_door_guards_hold_combat_no_route_like_
  the_game`, `#[ignore]` with the two named blockers** — the game's assert (0x5f at every
  30-frame sample over ≥17 samples) is written game-true and must not be weakened.
- `crates/nf-view/tests/walkreal_probe014_param26_live.rs`: the param-26 probe (diagnostic).

## How it was verified

- `door_guards_locked_door_07000014_live`: **2 passed, 0 failed, 1 ignored** (5.3-5.6 s,
  release, NF_ISO set).
- `walkreal_07000014_live`: **6 passed, 0 failed** (43-45 s) — the pulled conjunct restores
  master's green (with the conjunct: 5 passed 1 failed, twice).
- `walkreal_07000001_live` with NB_SCAN_LIVE flipped on (a throwaway build, reverted): **5
  passed, 0 failed** (68.9 s) — the const's 01 blocker note is stale.
- Full suite: `tools/test/suite.py` was started on this tree repeatedly; this sandbox's 2-core
  build could not complete build+run inside the window, so no total line exists yet — the next
  session should run it on this tree (the touched binaries are quoted above).

## What's left

1. Flip `NB_SCAN_LIVE = true` (the 01 note at the const is stale — prove with the full suite).
2. Root-cause the release-to-door divergence: which of #18/#19/#20 the game sends to the door
   and why the port's three stay at 0x38 — then the pairing bites and the hold pin un-ignores.
3. The full-suite total line on this tree.

## Live checks for the main agent

1. **The side bits during the hold.** gdbwatch drone+0x31C for BOTH door guards during the hold
   window: expect bits 8|0x10 set on each (each guard reads the other as the neighbor),
   rewritten every tick (ControlSTANDARD 0x1490f0:202 zeroes them, then NoBunching re-stamps).
2. **The neighbor pointer + the return byte.** drone+0x314 (the paired neighbor) non-zero and
   drone+0x17 (NoBunching's return) == 1 at the door — these two confirm the pairing chain end
   to end.
3. **The released set.** Which guards walk to the door in the game (the live note says two stop
   at (-51.6,41.2)/(-51.2,38.8)) — their placement indices (the port's #18/#19/#20 stay home) —
   so the release chain can be traced against the disc script.

## Usage

not visible to me (this environment shows no token, credit, context-window or cost figures for
the session; nothing to copy).
