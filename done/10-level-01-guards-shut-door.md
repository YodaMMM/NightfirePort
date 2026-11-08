# 07000001: alerted guards and a shut door

In the hall before the south door, alerted guards that heard Bond's shots stay 3-7 u back and side-step, and never open the shut door to reach him. Read the drone's investigate/chase states and its door use (NDrone2 door open calls, LinkCreep_Handler 0x15a2c8, the hearing alert) and find whether the game's guards open a shut door to reach a heard shot. Reproduce through real input from the hall's south side, compare, fix if the port differs.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-01-guards-shut-door`, report in docs/reports/level-01-guards-shut-door.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox7 at 2026-11-08T14:50Z

---
## Report (done, 2026-11-08T16:39Z)


---
## Report from the worker (done, 2026-11-08T16:39Z)

# level-01-guards-shut-door — 07000001: alerted guards and a shut door

Task: in the hall before the south door, alerted guards that heard Bond's shots stay 3-7 u
back and side-step, and never open the shut door to reach him. Read the drone's
investigate/chase states and its door use (NDrone2 door open calls, LinkCreep_Handler
0x15a2c8, the hearing alert) and find whether the game's guards open a shut door to reach a
heard shot. Reproduce through real input from the hall's south side, compare, fix if the port
differs.

**Verdict: the port matches the game — no fix.** The game's guards DO open shut doors to
reach a heard shot whenever the route their path record carries crosses that door's flagged
node pair, and the port implements every piece of that machinery. But on 07000001 the south
door's flagged pair lives only on the kind-8 Mayhew mission route, which the game's A* never
searches — so the hall guards can never acquire a route through it, and the game's own combat
law parks them exactly where the port parks them: 3-8 u back, cycling side-steps and
stand-and-fire. The pinned repro test is green on the current code; the original expectation
(a guard opens the door) was the wrong one and is documented below.

## What the game does (the decompile chain)

- The hearing chain: `DroneFunc_HandleSoundAlerts` 0x148a30 sums `Sound_Alertness` per frame
  and sends msg 0x14; `DroneVision_AlertSound` 0x177800:62-78 sends alertness > 0.7 straight
  to Attack 0x56, otherwise share > 0.2 or alertness > 0.66 to HeardNoise 0xA2; Suspect 0xA4
  walks the sound via prop 0x21 → AlertToPosition 0x62 [M 0x16b0e0:20-79], and any further
  loud shot (share > 0.1) sets `+0x4F8 |= 0x4000000` and Attack [M 0x16b0e0:120-126].
- The door use is unconditional ON A ROUTE: every followed route runs
  `NDrone2_ReachedDestNode` 0x153358 (from `LinkCreep_Handler` 0x15a2c8:58-65); its
  door-node case binds the door into Drone+0xBA8 and enters DSTATE_OpenDoor 0x81 with no
  seen/heard gate [M 0x153358:144-174]; `NDrone2_OpenDoor` 0x14aa38:8-13 calls
  `Door_Activate(door, drone, 0)`; while a door is bound, `LinkCreep_Handler` 0x15a2c8:66-68
  SKIPS the blocked test; `NDrone2_MoveToGoalPosition` 0x1517d8:61-63 returns 10 (door
  pending) whenever the drone state is 0x81 and AlertToPosition 0x165270:72-75 waits;
  `NDrone2_ControlSTANDARD` 0x1490f0:233-245 re-opens every 60 ticks in any state.
- BUT the route itself is confined to ONE path record: `AINetwork_DoAStarPath` 0x1571b0
  searches `route+0xF4`'s own node/link arrays (+0x2C count, +0x34 nodes, +0x3C links, +0x40
  adjacency) — there is no cross-path search. The path records eligible for the node searches
  are those whose in-memory +0x08 word reads `& 0xD == 0`, and that word IS the disc record's
  kind: `AIPath_Parse` 0x1d2598:18-30 stores disc +0x80 (the kind) into in-memory path+0x08,
  and the filter reads path+0x08 in `AINetwork_NodesForPosition` 0x1586a8,
  `AINetwork_NavPathForPosition` 0x158488 and `AINetwork_NodeSearch` 0x15aa88. The drone's
  home path is picked once at `NDrone2_PostLoad_Init` 0x14d890:18 (`NavPathForPosition` →
  Drone+0x954).
- So a heard shot produces a route only inside the guard's own kind-eligible path record.
  Combat standoff bands when no route crosses: `DroneFunc_CombatState` 0x148338:45-49
  approaches to `min(engage_range, 6.0)` (the expression folds to 6.0 when engage >= 6.0),
  arrival 2.0, out-of-range → 0x5A, unseen → 0x5C/0x5F, and 0x5F's update rolls
  `NDrone2_ChooseCombatMove` 0x150c18 — step 1.8 [M 0x14f640:63], dodge 3.6, roll 5.2.

## Why the 01 hall guards never open the south door (the level's nav data)

The port nav census [L, `guards_shut_door_07000001_live`] and the disc block [D]:

- Port path 0 — kind 0x8, 37 nodes, the whole level, and ALL THREE door pairs: the south
  shoji n5 (7.1, 10.2) / n6 (7.0, 11.8) [door #1, (7.92, 5.34, 11.14) [D placement]], the
  west door n10/n11, the exit door n15/n18. This is the Mayhew mission route (npc_doors_live
  "the kind-8 Mayhew route, path 0"). Kind 0x8 & 0xD = 8 → **excluded from every node search
  in the game** [M 0x1586a8/0x158488/0x15aa88].
- Port path 1 — kind 0x2 (NavPath), 28 nodes, hall + west room; its ONLY door pair is the
  west door #2 (n21 (−14.68, 14.06) / n22 (−13.50, 14.09) [L]). The four ch-0xE hall guards
  route here [L the in-test route probe: path Some(1)].
- Port path 2 — kind 0x2, the courtyard; its pair is the courtyard door #0 (n14/n16).
- Port path 3 — kind 0x4 (patrol) → excluded by bit 2.

So no kind-eligible path record carries the south door's pair: a hall guard can never A* to
the meet-room side of the shut shoji — in the game either. The observed cycle in the port
(guards 5/6/7/11: AIM_STAND_FIRE 0x68 → COMBAT_NO_SIGHT 0x5C → COMBAT_NO_ROUTE 0x5F →
STEP_AIM_L/R 0x73/0x74, holding z 15.1-19.6 = 3.3-8.5 u north of the plane) is the game's
own combat law producing the reported "3-7 u back, side-stepping". The pre-opened aperture
works because `Route::to_position`'s direct `move_test` needs no A*: the existing
walkreal_07000001 lure ("the guards come through the aperture ONE AT A TIME") and the
npc_doors drives already pin the pair-crossing chain on this level
(`the_01_escort_driven_through_the_shoji_opens_it_and_crosses`, green).

## The reproduction (real input, no teleport after load)

`crates/nf-view/tests/guards_shut_door_07000001_live.rs` —
`south_shut_door_the_hall_guards_hold_the_standoff_like_the_game`: load → the intro (ch 0xB)
→ the meet-room walk onto the meet plate (ch 0x32 → SP #276 → ch 0xE, the four hall guards
enable) → the hold line (7.2, 6.4, 8.8), 2.3 u south of the shut shoji (the door auto-opens
at ~1.1 u [L the stage-walk trace], so every leg stays z <= 10.0 and the assert pins the door
still CLOSED at the hold) → 2400 ticks (40 s) of pistol fire aimed into the hall, tap every
16 ticks, reload on dry. Result: `door_opened_at None, crossed []`, the guards cycling the
combat states above — and the test PINS this (door stays shut, nobody crosses) as the game's
law, with the kind-8 exclusion as the cite.

History of the session's runs: the first assertion (a guard must open the door) was RED on
the current code — that red is what drove the decompile/nav investigation above; the corrected
assertion (the standoff law) is the one the game's own data supports. A second staged scenario
(west door #2, whose pair IS on the hall path) could not reach the shut-door premise: the
walk-by leaves guard #5 (skin 0x5000039) parked 1.0 u from the open shoji's live sphere
(r 1.56 at (−14.1, 6.6, 14.0)) in AIM_STAND_FIRE, and a body inside sphere+radius blocks the
360-tick auto-close [M 0x13660c-0x136700] — the door never shut, so the scenario measured the
camper, not the door chain (census in the session logs; the chain itself stays pinned by the
npc_doors drives).

## What's left

- The full 4-crate suite did not fit the session (setup consumed the first hour: sandbox
  restart, /tmp wipes, a disk-full at 100% cleared by dropping stale test executables). No
  `crates/*/src` file was touched, so per rule 10 the touched-code sweep is empty; the level's
  own batteries were run whole (below).
- A live confirmation of the camper reading (below).

## Live checks for the main agent

1. South standoff on real hardware: load 07000001, walk Bond south of the shut south door,
   fire ~30 s into the hall. Watch hall guard Drone+0x10C (state, reads): expect the
   0x58-family cycle (0x5C/0x5F/0x73/0x74/0x68) and NO 0x62/0x81; watch Drone+0xBA8 (bound
   door, writes): expect 0 throughout; watch the shoji object's obj+0xF4: expect 2 (closed).
   If the real game enters 0x81 or opens the door, my kind-8-exclusion reading is wrong and
   the nav data needs a re-measure.
2. Home path pick: at the guards' enable (ch 0xE), read Drone+0x954 for guards 5/6/7/11
   (writes at PostLoad 0x14d890:18): expect the kind-2 hall NavPath record, never the kind-8
   record.
3. The west camper: with door #2 open and a guard parked in the aperture (AIM_STAND_FIRE ~1 u
   from the leaf), watch the door's obj+0x72 (auto-close timer): expect it to reach 360,
   re-arm at 360 without closing while the guard stays within ~2 u of obj+0x80, and the door
   to close within 360 ticks after he leaves.

## Test binaries run (whole)

| binary | result |
| --- | --- |
| nf-view `guards_shut_door_07000001_live` (new) | 1 passed, 0 failed |
| nf-view `walkreal_07000001_live` | 8 passed, 0 failed |
| nf-view `npc_doors_live` | 6 passed, 1 ignored (the pre-existing pinned escort ignore) |
| nf-view `fix_npc_doors_live` | 1 passed, 0 failed |
| nf-game `ai_live` | 37 passed, 0 failed |
| nf-game `doors_01_live` | 2 passed, 0 failed |

## Usage

The worker session's own token / context / cost figures are not visible to me. The two
research subagents this session spawned reported, at their return: port-side drone/door map —
prompt 215171, completion 6908, total 222079 tokens; decompile-side guard/door map — prompt
161762, completion 8112, total 169874 tokens (read from the subagents' result footers).
