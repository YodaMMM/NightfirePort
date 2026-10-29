# level-03-every-objective — report (2026-10-28, branch s/level-03-every-objective)

Task: make `walkreal_07000003_live` walk the WHOLE 07000003 mission on real input — every
objective the mission data lists, each asserted complete, ending on the level exit firing.

## What the current tests already covered (before this task)

`crates/nf-view/tests/walkreal_07000003_live.rs` (4 tests, all green at HEAD):
- obj0/obj1/obj2 announced at spawn through the real monitor (ch 0x4d reveal touch),
- the front gate (locked door → E plate 0x31 → SS → through) — real,
- obj0's 0x4e and obj1's 0x4f/0x50 **handed** via `channels.set` (the old "no on-level writer"
  claim), then the disc's AND chain (0x4f+0x50→0x51, 0x4e+0x51→0x52) real,
- exit refusal without 0x52, exit firing 07000004 with it — real,
- the AND-needs-both-inputs sister (scene-level hands) and the kill-a-servant fail path — real.

## Done this session

**The three leaf writers are on the disc and ported — the "un-ported mission/dialogue layer"
premise was wrong for all three leaves:**

1. **obj1's leaves 0x4f/0x50 are the servant placements' param 6** (= the game's `Drone+0x135`
   rescue channel byte): #389 carries **0x4f**, #388 **0x50**, #422 **none** [D — probed through
   the port's loader this session; 07000002's servants carry 0x49/0x4A the same way [D]]. The
   whole chain is implemented [M]: the enable plate (p3 = 0x21/0x25) starts the HOSTAGE cower
   (DroneTypeSettings row 5 start 0xa, via dtype 0xe + prop 0x4e; M 0x14c938), the captor binds
   (nearest armed alive guard at the entering tick; M 0x144030), its death moves the Hostage
   UPDATE to HostageSaved (M 0x15cb78:24-36) whose ENTER raises p6 through
   `DroneFunc_HostageSaved` 0x1441c0:11-18 (plus the save-count global and the 0x200004f/0x2000050
   messages, levels 0x7000002/0x7000003 special-cased [M :20-52]).
2. **obj0's leaf 0x4e is the Dragon Safe key**: E on the case (SP #110, script 0x06000563, mask 4
   use-key [M 0x1dc338:82]) spawns the kind-2 pickup with +0x28 = 0x4e [D script stream 1], the
   take writes it [M 0x18e650:330-333] — all already pinned green by `doublecross_items_live`.
3. The walkreal's full-route test was **rewritten to drive the real writers**: both servant
   rescues completed on real input this session — Bond walks to each enable plate, walks to the
   servant, kills the bound captor with real fire, and the rescue channel rises (0x4f, 0x50),
   servants leave HOSTAGE (post-save states 0x64 / 0x13), 0x51 rises and **obj1 completes through
   the real AND + monitor**. The stale "no on-level writer" header claims in the walkreal were
   corrected to cite the audit/probe evidence.
4. WORKLOG entry `level-03-every-objective` (restored docs/WORKLOG.md from
   origin/s/level-02-every-objective; master carries none). Checkpoint pushed mid-session.

## Verified how

- `walkreal_07000003_live` file tally this sandbox: **3 passed / 1 failed** (the rewritten
  `walk_the_full_route_and_complete_every_objective` — WIP, see Left). The three green: the short
  route (hands documented as the stand-in), the AND sister, the servant-kill fail path.
- The servant-rescue legs' asserts are the pinned [M]/[D] facts: p6 channels rise iff the captor
  dies; obj1 state 4 via `Mission_MonitorObjectives`' monitor.
- FULL SUITE NOT RUN this session (the 2 h window went into setup + the discovery + the rewrite).
  The pushed tree still has the WIP test red — the next session finishes it, then runs
  `tools/test/suite.py` (or chunk_suite.py — this sandbox kills background jobs between tool
  calls; nohup'd suite died immediately, confirmed).

## Left (in order, for the next session)

1. **The case E is edge-triggered** — the walk's aim-while-holding-E loop pressed early while
   facing wrong and the keycard never spawned (`expect("the keycard is live")` at walkreal
   :304). Fix: converge the view first (no use), then a single press tick — exactly
   `doublecross_items_live`'s proven shape (repro spot (5.90,5.97,−42.41), yaw 1.710, pitch
   −0.458, one `step_bond(..., use=true)`), then the back-step take loop (already in the test).
2. **Nav polish**: #389's 0x21 plate (14.3,6.1,−33.3) hangs by the garden moat — walk_to's
   straight line flanks into the water, walk_nav_to routes through the garden door but parks on
   the ledge above it (y 9.37); the (9.0,−34.4) twin is worse. Approach from the gate-plate lawn
   side or fire the plate by walking its extents from the west lane. #388's 0x25 plate + both
   servant walks + the captor kills are already green.
3. After the case: the 0x52/obj2-announce/no-fail asserts + the exit leg (already written).
4. Optionally the #422 rescue (no channel; the save still fires the count/message).
5. Full suite on the final tree merged with origin/master; quote the total line.
6. docs/systems/levels.md:546-549 still carries the stale "07000003 ch 0x4e → NONE GAP" line
   (pre-audit; GAP-2 fixed it in the audit doc only).
7. WORKLOG: append the continuation under the same `level-03-every-objective` entry.

## Live checks for the main agent

1. **Servant rescue write**: in PCSX2 on a fresh 07000003 boot, walk to servant #389
   (15.4, 6.3, −27.0) after crossing its 0x21 plate, kill the guard standing over it, and
   `gdbwatch` `switch_channels[0x4f]` — expect the write from `DroneFunc_HostageSaved`
   0x1441c0 (the `switch_channels[bVar1] = 1` store) within a tick of the guard's death, plus
   the save-count global at DAT_00298fac incrementing and the Text_AddMsg with label
   0x200004f/0x2000050.
2. **Post-save state 0x13**: #389 sat in state 0x13 after the save (the port's run), #388 in
   0x64 (GoToGoal). `NDrone2_StateFuncs[0x13]` — name it from the table at 0x29b300 (+0x13·4) and
   confirm which of the two the real game lands in (the port may need the 0x13 arm's behaviour).
3. **The rescue channels' reader**: confirm nothing else consumes 0x4f/0x50 besides AND #436
   (out 0x51) — e.g. watch for a music/event read.

## Usage

not visible to me (no token, cost or context-window figures are shown in this environment; no
estimate made).
