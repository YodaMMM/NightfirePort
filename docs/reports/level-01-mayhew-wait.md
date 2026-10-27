# level-01-mayhew-wait — the 07000001 escort holds at its ally-wait node

## What was wrong

On Double Cross Part 1 (07000001) the port's escort (Mayhew, skin 0x0500004F) led straight
through the hall corridor and through door #1 while Bond was still at the corridor start
(playtest: Bond (4.63, 6.47, 6.80) at frame ~4255, Mayhew already past the far doorway). The
follow-distance bands ([M] `DSTATE_AllyLead` 0x15e698 case 3, path < −9.0 → AllyLeadWait) were
ported and pinned (`escort_follow_07000001_live`), so the missing piece was the **hold point**:
the game parks the escort at the route's ally-wait node until the ambush beyond the next door
is cleared, and the port had that dispatch resolved-but-disabled.

## The rule, and every link of its chain

- The kind-8 "Mayhew Route" (nav path 0, kind 0x8, 37 nodes [D the level census]) carries
  node 33 (2.91, 5.74, 7.57) with flag 0x20 — the ally-wait node, west of door #1's plane
  (door pair nodes 5/6 at z 10.17/11.78, the shoji at (7.92, 11.14) [D]).
- `NDrone2_ReachedDestNode` 0x153358:85-94 [M]: a drone reaching a 0x20 node while
  `switch_channels[0x99]` is off saves the current state in +0x5A2 and SetStates
  `DSTATE_AllyLeadMissionWait` (0x21). The real-game recording `tests/traces/01-doors.trace`
  holds the contact in 0x21 at (2.94, 7.22) for 1262 frames [L trace].
- 0x21's UPDATE (0x15efc8 case 3 [M]) returns to 0x1d AllyLead when the latched channel
  (+0x12C, no writer → 0x99) rises.
- The release chain, all on disc data of 07000001 [D]: the TOUCH plate at (6.79, 6.04, 8.91)
  (trigger #2, mask 0x08 player) writes ch 0x32 → the OUT mux (0xEE, out 0x32, list [0x33])
  mirrors it into ch 0x33 every tick [M Trigger_Update 0x1b1a78 case 5] → SP #276's start
  channel is its param-6 `out` = 0x33 [D placement; M SP_Update 0x1dbda8 state 0] → script
  0x06000086's stream-9 event 0xd writes ch 0x0e → the courtyard guards 6/7/11 at
  (5.42,14.44)/(9.05,17.01)/(5.93,17.86) enable → their death channels 0x11/0x12/0x13
  (`DroneFunc_SetDeathChannel` 0x13e660) → the AND mux pl 318 (0xEC, inputs [0x11,0x12,0x13],
  out 0x99) → 0x99 rises and the hold releases.

**The earlier note in `Drones::tick` ("ch 0x21's writer is an open live check") was a mis-read:**
SP #276's start channel is param 6 = 51 = 0x33, not 0x21, and its writer is the corridor plate
plus the OUT mux — no live check needed. The previous "enabling stalls the escort forever"
repro never armed the ambush because the harness never crossed the plate.

## The fix

- `crates/nf-game/src/drone.rs` (`Drones::tick`): the ally-wait outbox now parks the drone in
  `LEAD_MISSION_WAIT` while ch 0x99 is off (return state saved), replacing
  `let _ = dr.ally_wait_node.take()`. [M 0x153358:85-94, 0x15efc8 case 1/3.]
- Harness steering (not assertions) in `escort_follow_07000001_live` and `fix_npc_doors_live`:
  when the hold is armed and ch 0x33 is still off, Bond walks over the corridor plate — it is ON
  the escort route between the hold and door #1, so the walking player crosses it.
- `fix_npc_doors_live` budget/cap updates for the added ambush leg (measured: door #2 now opens
  at +160190, the escort settles past the doors at +271341).

## Verification (all through the real input path, `nf_view::load` + Walker ticks)

- New `crates/nf-view/tests/mayhew_hold_live.rs` — **green**: the escort parks 0x21 at
  (3.40, 6.34, 7.19) (dxz 0.62 from node 33 [D]), holds ≥ 300 ticks with Bond standing back at
  the corridor start, door #1 stays CLOSED the whole hold (he never works the door ahead of
  Bond); Bond walking the plate raises ch 0x33; killing the three ambush guards raises ch 0x99;
  the hold releases. Evidence tags are in the test header.
- `escort_follow_07000001_live` — **green** (the bands still pin with the hold in the state set,
  which already listed `LEAD_MISSION_WAIT`).
- `fix_npc_doors_live` — its door #1 assert said the escort self-opens door #1; the hold changes
  that flow, and the disc data says the player opens door #1 during the ambush (the guards whose
  deaths release the hold stand beyond it). Fixed the TEST per the rule (the premise contradicted
  the decompile-backed hold): the escort's first self-opened door is now pinned as door #2
  (measured OPEN at +160190, bond→contact 17.0). Final run at the time of writing: the escort
  led past both doors and settled at (−30.84, 6.34, 11.01) (+271341); the revised-assert run was
  still executing when this report was committed — its result is noted in the WORKLOG entry.
- Ran: `mayhew_hold_live`, `escort_follow_07000001_live`, `fix_npc_doors_live` (nf-view), plus
  nf-game unit tests transitively via the same cargo invocations. The full `tools/test/suite.py`
  run did NOT fit this session's sandbox window (background processes are killed between tool
  calls here; the build alone spans two 10-min foreground chunks) — the next message should run
  it on this branch merged with origin/master, with `walkreal_07000001_live` the main test to
  watch (it walks the whole level; its harness will need the same plate-crossing step if it
  stalls at the hold).

## What's left

1. Full suite on the merged tree; `walkreal_07000001_live` harness may need the plate step.
2. The escort re-park question: in the failing `fix_npc_doors` run the contact was back at the
   hold area once after leading west — worth one live check whether the game re-parks a returned
   escort (0x99 should stay on: the death channels persist [D channel-persistence]).
3. Live checks for the main agent (PCSX2, fresh boot of 07000001):
   - At the hold: `switch_channels[0x99]` reads 0 while the contact is in state 0x21 at
     (2.91, 7.57), and flips to 1 exactly when the third ambush guard dies.
   - The plate (6.79, 6.04, 8.91) writes ch 0x32 on player contact; ch 0x33 mirrors it the same
     frame (the OUT mux 0xEE).
   - SP #276 (placement 276) enters state 2 the frame ch 0x33 rises; script 06000086 stream 9
     writes ch 0x0e within its first seconds.

## Usage

`not visible to me` — this sandbox shows no token, cost or context-window figures.
