# level-07-guest-collision-3 — the frozen mission walkers walk; the doorway clears itself

## Verdict

The frozen-walker root cause was **not** the leading candidate (`request_anim`'s same-state early
return — disproven: the walk danim loops with `held false`, `anim.state == walk` throughout the
freeze). The real departure is in the **undrawn slide**: the game moves an undrawn walking drone
by a raw position add with **no collision test**, while the port routed the same slide through
`move_by`'s probe — whose 0.4-overshoot rays graze the 07000007 west doorway's throat and
rejected every 0.02 step forever. One [M]-cited change in `drone.rs` unfreezes every drone:
`walkreal_07000007_live::walk_the_party_start_to_finish` is green, the walker clears the
doorway on his own, and the guards' patrol surfaces stay green.

## The [M] chain

- `NDrone2_DoAnimation` 0x149f88:11-18 — for an undrawn non-guest (`obj+0xFC == 0`, class
  `+0xD8 != 0xC`, settings speed non-zero) with no blend-out script (`+0x538 == 0`), the gate
  returns 0 exactly when the playing danim's info-flag-4 is set: `+0x570 & 4`, and `+0x570` is
  the info flags word stored from `Drone_AnimInfo[danim]` at `SetDAnimInternal` 0x13aee8:50.
- `Drone_Control` 0x139268:75-96 — on that 0, the slide arm adds
  `sinf(obj+0x54) · Drone+0x58c` to `obj+0x30` and `sinf(obj+0x54 + π/2) · Drone+0x58c` to
  `obj+0x38` **directly** — no `NDrone2_MoveTest`, no ray, no push. `Drone+0x58c` is
  `Drone_AnimInfo+4` stored when the danim was set [M 0x13aee8:51] (0.02 for walk 0x188).
  (The game also clears `+0x4F8 & 0x100000` there and clamps a negative speed against the
  +0x320 strafe bits while a door is bound [M :77-84]; neither has a port reader — noted.)
- The port's slide arm (same trigger, [L]-validated in the comment) called `move_by` →
  `probe_move`: two rays at feet/knee with a 0.4 overshoot past the step. Inside the doorway
  throat both rays graze the jamb geometry and the step is rejected — the walk anim kept
  looping (`held false`) while the position never changed. The freeze trace: drone[22] frozen
  at (79.42,−50.32), `anim st 0x37 dn 0x188 time 0 held false`, **994 consecutive
  `probe_move` rejects** with `feet false knee false`.

## What changed

1. `crates/nf-game/src/drone.rs` — the undrawn slide is now the game's raw add (`pos` and
   `special_spot` shifted by `sinf/cosf(yaw) · v`), with the citation; `move_by`'s probe no
   longer stands between an off-screen walker and his route. This is every drone: mission
   walkers, patrolling guards, anyone behind the camera.
2. `crates/nf-view/tests/common/walker_b2.rs` — the stuck ladder gained the player's back-off
   (stuck 3: a 60-tick straight reverse before the re-approach), and an open/swinging door no
   longer resets the wedge counter via `use_door`'s open early-return (the ladder can climb).
3. `crates/nf-view/tests/walkreal_07000007_live.rs` — the new
   `the_mission_walker_completes_his_doorway_leg`: Bond wakes the chan-23 group through the real
   plates, stands at the hall mouth facing west, and drone[22] must cross the doorway and reach
   path 3's node 1 (74.39,−44.97) [D the level's nav dump] — the exact leg that used to freeze.

## How it was verified

- `walkreal_07000007_live`: **3/3 green** — `walk_the_party_start_to_finish` (the party walk
  passes; the walker is no longer parked in the west doorway), the wake/zoning test, and the new
  doorway-leg test (walker22 travelled 5.01 units to the far node at t256).
- Guards'-patrol sisters with the slide touching every drone: `rook_door_06_live` 1/1,
  `rook_door_06_walk_live` 1/1, `fix_guest_solid_live` 3/3 — all green.
- `nf-game` builds clean; `nf-view` test targets compile clean.
- Ran: `cargo test --offline --release -j2 -p nf-view --test walkreal_07000007_live`,
  `--test fix_guest_solid_live`, `--test rook_door_06_live`, `--test rook_door_06_walk_live`,
  `--test interactables_0700000a_live`, plus the compile of the full workspace.

## What's left

1. **`interactables_0700000a_live::armour_pickup_taken_by_walking_over_it` is still red**, and
   the stall MOVED with the fix: the 20 script civilians now walk their routes, and g8 opens the
   node-7 corridor door d4 on his own pass (state 0x81 at t570 [L probe]); the door's swung-open
   panel (`st 0x0004, pos01 1.00` at (−19.5,4.60)) forms the pocket Bond wedges in at
   (−20.23,3.93) — 1.74 short of node 7, input fully eaten, until the level's fail chain
   restarts (exit at +2978). `use_door`'s open early-return refuses the press, the ladder's
   sidestep/jump can't clear a panel, and the E-toggle attempts I tried (aiming at the placement
   pos and at the live `obj+0x90` matrix translation) never moved the door state — the press
   path needs the exact `Player_Activate` cast shape (head+look pitch, the panel's sphere) as a
   [M] read before it will land. Next message: port the E-toggle at the live panel pose (or walk
   the crossing before g8's door pass), then the walk goes green.
2. The full-suite total line: the sandbox killed every background process between tool calls
   (the fast runner cannot survive here), and the 2-hour window closed during the foreground
   run — **no total line this session**. The per-crate runs above are the recorded totals; the
   final tree IS merged with origin/master (already up to date at merge time).
3. `request_anim`'s same-state early return stays as shipped (it matches the game's
   `ZeroAnimCall` arm [M 0x13ab58:97-103], backed by the [L] patrol trace).

## Live checks for the main agent

1. 07000007 fresh boot, gdbwatch on drone[22] obj+0x30 while Bond stands at (76.6,−50.7) facing
   west: the walker should cross the doorway throat and walk the hall north-west at ~1.2 u/s
   (0.02/tick) with no gait — the raw slide. If the real game's undrawn walk shows GAIT
   animation instead, the slide arm's trigger read needs a second look.
2. 0700000a fresh boot: does the real E on the OPEN corridor door d4 (panel swung, pos01 1.0)
   toggle it closed? Watch `Door_Activate` 0x136xxx's hit test against the swung leaf — if the
   real game refuses the toggle too, the player's play is a walk-around, and the armour walk
   needs the crossing timed before g8's door pass instead.

## Usage

not visible to me
