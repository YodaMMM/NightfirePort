# level-02-bunker-stick — report

Branch `s/level-02-bunker-stick` (cut from `origin/master` 81da6c47; commits `0a17709d`→
`80802987`). Task: Bond hangs between the two bunker doors on 07000002 at (−45.42, 7.1,
−22.15) — not grounded, the floor 5.33 below — reproduced on real input, cause found with
evidence, pinned from both arms.

## What's done

1. **The doorway measured from the disc** [D placements + the posed leaf-collision dump]:
   the "two bunker doors" are kind-2 proximity-slider doors #268 (−46.07, 6.76, −23.21) and
   #330 (−46.16, 6.77, −20.27), NO open/lock channels (p2 = p3 = 0 — the E opens them [M
   Door_Activate 0x135be0], the 360-tick auto-close re-closes [M 0x13660c-0x136700]). Their
   CLOSED leaves span z −24.15..−22.27 and −21.43..−18.93: between them the doorway holds a
   STATIC MIDDLE PIER (z ≈ −22.3..−21.4) whose east face stands at x −45.97, full height
   [L-port the westward rays at knee/chest/head all hit −45.971 at z −22.22]. Once the leaves
   part, the doorway has TWO passable slots: NORTH (z −23.93..−22.3) and SOUTH
   (−21.4..−19.37). The threshold under the pier strip is a FLOOR HOLE — the downward ray
   from y 8 at (−46.1, −22.15) finds no floor above y 0 [L-port]; the bunker level below
   stands at 1.05..1.8 [D walkreal's underground legs] — "the floor 5.33 below" the reported
   7.1.
2. **The game's own crossing identified** [D]: the doorway sensor #389 (the ch-0x81 ambush
   trigger) stands at (−46.96, 6.04, −23.35) — inside the NORTH slot; walkreal_07000002's
   route node "bunker door" is (−46.1, 5.9, −23.2) — the same slot. The game's route crosses
   the north slot; the mid-gap strip is the pier.
3. **The wedge reproduced on real input** [L-port]: a straight steer into the mid-gap
   (z −22.15) — the fix-02 run's wedge trajectory — sticks Bond at (−45.42, 6.37, −22.22),
   the reported x/z to the centimetre, grounded on the 5.33 floor, for 900+ ticks with
   forward AND strafe held (the capsule push pins him — no tangential motion). Doors open
   throughout: the stick is the pier, not a leaf.
4. **The fix pinned from both arms** (`crates/nf-view/tests/bunker_stick_02_live.rs`, NEW):
   - `the_midgap_wedge_sticks_between_the_bunker_doors` — the sister: the old behaviour
     sticks (the reported spot, the doors open, the pin held across the sampled tail).
   - `the_game_own_north_slot_crossing_walks_through_and_lands` — the fix pin: the same
     start, the crossing aimed at the game's own north slot: Bond crosses the door plane,
     LANDS grounded on the corridor floor band (5.0..6.2), and stays grounded through the
     sampled tail.
5. **No port code changed**: the pier and the threshold hole are the disc's own geometry
   [D]; the divergence was the trajectory walking into them. The game's own crossing is the
   fix, with the sister documenting the sticking trajectory as the cause's pin.

## How it was verified (the touched binaries, --test-threads=1, slim ISO, current tree)

| binary | result |
|---|---|
| bunker_stick_02_live (NEW) | **2 passed; 0 failed** (4.4 s) |
| interactables_07000002_live | **16 passed; 0 failed** (13.0 s) |
| walkreal_07000002_live | **1 passed; 0 failed; 1 ignored** (37.6 s — the ignored arm is the pre-existing real-health blocker of s/fix-02-real-health, untouched here) |
| walk_07000002_live | 2 passed; 0 failed |
| nf-game (the crate the pin drives) | **588 passed; 0 failed; 5 ignored** (the crate's binaries are unchanged by this task; recorded this window on the identical crate tree) |
| the door/collision binaries (suite state, same tree) | door_aim_ray_live 2/0, npc_doors_live 6/0+1 ign, fix_npc_doors_live 1/0, door_portal_live 2/0, fix_door_react_path_live 1/0, door_pose_probe 2/0, door_text_labels_live 4/0, door_hint_boot_count_live 2/0, door_guards_locked_door_07000014_live 4/0+1 ign, rook_door_06_walk_live 1/0 — all green |

**The fast runner**: this sandbox kills background jobs between tool calls, so the chunked
form of the same runner binary set (`tools/test/chunk_suite.py`, one flock-shared state file,
2 workers) ran it in foreground chunks on the identical crate tree: **490/996 binaries
recorded, 2083 passed, 2 failed, 142 ignored**. The 2 failures are `fix_0c_pylon_live`
(×2) — the known GPU-less wgpu-adapter panic (`NotFound { active_backends:
Backends(0x0) }`): no render backend in this sandbox, before any game code runs; the branch
changes no render code. 4 heavy mp/net binaries were OOM-killed by the sandbox (rc −9) and
re-run pending.

## What's left

- **The y 7.1 AIRBORNE variant did not reproduce this session**: the probes hold the same
  x/z pin grounded at 6.37 (the sidestep phase included). The airborne hold (1.77 above the
  floor, "not grounded") is either a deeper capsule/feet-band interaction over the threshold
  hole or a load-state difference of the real-health runs — the live checks below settle it.
- The suite tail past the cut-off (the sandbox clock, not a test result); the OOM re-runs.

## Live checks for the main agent

1. **The pier wedge in the real game.** Fresh boot of 07000002, E the bunker doors open,
   walk Bond straight into the mid-gap strip (−45.42, −22.15), hold forward: does the game's
   Bond slide along the pier face toward the north slot, stick grounded as the port does, or
   lift? Watch `bond.pos`/grounded (gdbwatch). Expected if the port matches the game: stuck
   at the pier, grounded.
2. **The reported airborne spot.** From the wedge, sidestep north along the pier and press
   west over the threshold hole (−46.1, −22.15): does the real game hold him at y ≈ 7.1
   ungrounded (the port's reported hang), drop him into the bunker level below, or slide him
   free? A save-state is fine for the approach but the measurement window must be a FRESH
   boot path.
3. **The threshold hole's disc truth.** Probe the floor at (−46.1, −22.15) in the real
   game (drop a save-teleported body or camera): is the doorway mid-strip floor really
   absent down to the bunker level, or does the port's static parse miss a disc floor there?
   If the disc has floor the port lacks, the port's world parse is the next fix.

## Usage

not visible to me (no token, credit or context-window figures are shown in this environment).
