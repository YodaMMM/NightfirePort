# pane-ride-on — intact glass really stops Bond on every level

2026-11-07, branch `s/pane-ride-on` (from origin/s/pane-collision-pairing 79dad06ea, merged
origin/master 10956badb). The pairing was proven and the ride wired behind `PANE_RIDE` (drone.rs,
off). This session settled the one blocker the pairing report left: the three 07000001
real-damage walks dying to guard melee under capsule-solid panes.

## The tick-by-tick bisection: which pane first changes which guard

Harness (throwaway, reverted): an env-gate on the `PANE_RIDE` switch plus a per-tick dump of
every live guard's position from the walker's tick. The same real-input Operative walk
(`walk_from_the_spawn_to_the_exit_with_the_contact_real_damage`) ran with the switch off and on;
the dumps diff tick by tick.

- The timelines are bit-identical to tick 682. **Tick 683: guard 13's x shifts by 0.01
  (−0.87 → −0.86) at (−0.87, 6.34, −0.47)** — the first divergence anywhere in the walk.
- The pane there: **k=27, placement 136, object (53,113)** — a thin (8-tri) kind-2 glass sheet
  at z = −1.167 spanning x −1.909..−0.086, y 5.340..7.947 [D the pane_pairing_census rows].
  The guard stands inside its x/y span, 0.70 in front of its plane, and his next move would
  brush it.
- Chaos from that 0.01 compounds through the fight/melee timings: at tick 10075 Bond died at
  (−28.4, 6.5, 16.4) to **guard #5's fists — 76.5 in 35 melee hits** (master: #5 deals 24.0 in
  8 hits and the walk passes with 37.2 left; 41 hits for the full 100 in the ride-on run).

## The game's own contact set: the guards really do brush the sheets

Read from the decomp (the task's fork point — per-cel scoping vs walk-side change):

- Movement queries are ROOM-scoped, and the room set is generous: the player's HITTEST start
  room is `obj+0x20` (his current room) [M Player_Collision 0x194e58], the guard's the same
  (`DCVars+0x450 = obj+0x20`, kind 0x800, mask 0 [M NDrone2_Collision 0x140e90:71-102,
  0x140ec4, 0x140edc]) — `Collide_StraddleCels` 0x1eac98 floods from that room through every
  portal within `queryR·1.5` of the sweep capsule, up to 126 rooms [M 0x1eb320:125,
  Intersect_Portal 0x1ea7b8:118-172]; per room `Collide_Pick` 0x1eb320:313-329 walks the
  room's object chain into `Collide_PickObj` 0x1eae80 (admission: the object's glist resolved,
  `glist+8 != 0` [M 0x1eae80:88-90]; `obj+0xF0 & 0x20 == 0` [M :72-82]) — plus a global
  straddle list at 0x270590 whose members join EVERY query in EVERY room [M 0x1eba44-0x1eba48].
- A placed 0x20 pane links into the room whose AABB contains its world-sphere center:
  `build_LinkToRoom`'s second argument — the `0` in `Break_Create` — is the straddle-test
  FLAG, not a room index (delay-slot `daddu a1,zero,zero` [M 0x12c29c]); the room is found by
  `build_FindCel` 0x1c8cb0 (AABB containment, up/down-ray disambiguation). `control_link_object_to_cel`
  0x1346b8 inserts it into that room's `cel+8` chain.
- The cascade pane's pairing: the guard's spot and the pane's centre share room candidates
  (map objects 1 and 188 contain both [D the probe]); the pane's centre also sits 0.15 inside
  room obj 106's z = −1.02 edge while the guard is 0.55 outside it — but the flood crosses any
  portal within `queryR·1.5` = 0.45..0.83 of the capsule, and the guard walks this wall
  0.5-1.0 away for his whole patrol. Under every reading the game's guard movement query
  admits this pane and slides on it, exactly as the port's ride does.

**Verdict: per-cel capsule scoping would not remove this pane — the port's global capsule set
is a fair stand-in for the game's room flood, and the walk is what changes** (the task's second
branch). A true per-room scoping would need the FindCel ray disambiguation, the straddle tests
and the portal-crossing test ported whole, to arrive at the same collision.

## The walk-side change that landed

`w.dmg.finish_fights = true` for the 01 real-damage walks (inside the 01 test file, gated to
`!tops_up()` — the existing 07000016 opt-in knob): a failed stand-and-fire window gets the
determined pass (`shoot_bounded`, ≤1800 ticks) instead of the 240-tick cooldown — the cooldown
was exactly the window a weapon-0x0 melee guard needed to finish Bond. With the ride ON the
Operative real-damage walk now **passes** (end health 3.0, deterministic — the sim has no
cross-run variance; 14520 ticks vs master's 9120).

With the ride OFF (the landed switch state) the three 01 real-damage walks still pass — the
flag only bites a failed fight, and re-running all three on the landed tree confirms no
regression (1/0 each).

## What still blocks the flip (honest state)

The Agent sister under the ride still dies: 48 hits for 100 — #5's melee 33.6 (baseline 24.0),
then the **west-door SMG pack #3 (26.5 in 12), #12 (23.1 in 17) and #17 (14.7 in 9)** shred
Bond at d 1.0-4.6 with no return-fire fight ever fired at them: the shifted patrol timeline
kept the pack out of every fight window (the route pre-clear met #5/#14/#15 instead), and the
final legs' gates did not fire before the death at tick 5774. The 00-Agent sister was not
reached in the window. So `PANE_RIDE` stays OFF and the two `dead_pane_live` ignores stay,
with the reason updated to this precise state (the d1-green + the exact d2/d3 blocker). The
next session: give the pack a fight window (pre-clear / leg-gate, level-gated to 01), then
flip the switch, remove both ignores and run the full battery.

## Runs (this session, NF_ISO set, --test-threads=1)

- 01 real-damage Operative: ride OFF passed (37.2 left); ride ON died (the bisection, the
  #5 shred); ride ON + finish_fights **passed** (3.0 left).
- 01 real-damage Agent: ride ON died (the SMG pack); landed tree (ride OFF) 1/0.
- 01 real-damage 00 Agent: landed tree (ride OFF) 1/0.
- The three on the landed tree with the ride OFF: 1/0 each (the flag regression check).
- dead_pane_live: 0 passed / 2 ignored (the updated reasons). glass_pass_every_level_live:
  unchanged (3 passed / 3 ignored — the ignores predate the pairing).
- The other walkreals and the ai binaries: the background suite covers them on this final
  tree — see the suite line below before the push. [SUITE]

## Live checks for the main agent

1. (carried from pane-collision-pairing #1) 07000001 museum hall, fresh boot: does Bond's
   capsule collide with the 0x20 kind-0/1 floor props (e.g. placement 22's object at
   (−10.7..−10.0, 6.1..8.2, 11.5..12.5))? The ride keeps them out of the set; a live answer
   arbitrates their [S] status.
2. (carried from #2, now sharpened) 07000001: do the hall guards deflect on the kind-2 sheets
   — specifically the sheet at z −1.167, x −1.909..−0.086 (placement 136): a guard patrolling
   at z ≈ −0.5 there. `gdbwatch.py` on the guard's +0x90 matrix while he crosses the sheet
   plane. This session's bisection + the decomp room-flood say yes; a live confirmation pins
   it.
3. (carried from #3) 07000009's 16 hp-6 panes (placements 108-123) and the drive glass —
   unchanged.

## What's left

1. The d2/d3 walk-side fix (the west-door SMG pack's fight window), then `PANE_RIDE` on,
   both ignores removed, the full battery (dead_pane, glass_pass, every walkreal, the ai
   binaries) and the suite with the ride on.
2. The kind-0/1 props' movement solidity ([S] unverified) — the live check above.
3. The bullet's glass-PIERCE deviation (inherited from done/30-dead-pane-collision-2.md,
   unchanged).

## Usage

The room-scoping research subagent reported, read from its return message: "Tokens ->
prompt:127598 completion:7313 total:134913". My own session's input/output tokens, context
used and cost: not visible to me.
