# level-0c-secret-door-reach-3 — report

Task: continue reach-2. The [L 2026-10-31] fresh-boot runs closed the two planned entrances to the
secret-door plates #558/#559 — the corridor's north end stops at z −150.23 on a barrel row in
every lane, and the ST-1 walkway's south end holds at z −153.65 (a railing, no drop). Find the
route the game really uses, make the port's barrels block like the game's, walk the real route on
real input, open the door through the plates, and delete the `[plate2 + 0.6]` placement in
`walkreal_0700000c_live`. Run walkreal_0700000c_live and interactables_0700000c_live.

## What's done

1. **The "barrels" are the level's own static collision, not placements [D].** The corridor
   placement dump (crates/nf-formats/examples/dump_0700000c_corridor_rows.rs) lists every
   0700000c placement in the x 40..70 / z −195..−135 box: the only dynamic kinds there are the
   kind-0xE6 cover nodes #313–#332 (y 8.71, the AI level) and the drones. The level's 0xE1
   Destroy barrels (Destroy_Create 0x12c318, health 20, blast 5.0 [M 0x12c318; D p2=4 p3=0x14
   p5=0x1f4]) stand by the secret door and in the warehouse — (61.9–63.2, 0.38, −183) — NOT at
   the corridor's mouth. Nothing dynamic blocks at (53–56, −150); the row the [L] run hit is
   static geometry. The port already carries it: the lane probe
   (walkreal_probe0c_barrelrow_live.rs) measured the port's south walk stopping at
   z −150.24 / −150.23 / −150.74 in the [L] lanes x 53.0 / 54.3 / 55.4 — the game's −150.23.
2. **The barrel-row stop is PINNED (the "make the port block like the game" deliverable).** New
   test crates/nf-view/tests/walkreal_0700000c_barrelrow_live.rs: for lanes x 51.0 / 53.0 / 54.3 /
   55.4 the south walk (real input, steering at the corridor mouth exactly as the live run) must
   stop with z in (−151.6, −149.5) — the [L 2026-10-31] wall — and must NOT reach z −170.
   walkreal_0700000c_barrelrow_live: 1 passed, 0 failed.
   The strict-south lane's east slide past the row's east end (the port's walker at x 54.3
   pushing straight south glides east to (61.35, 1.57, −170.02) around x ≈ 55.7 at z −150..−152)
   is printed, NOT asserted — the game's east end was never measured [S; live check 2].
3. **The walkway lip seal reproduced.** walkreal_probe0c_lip_live.rs: the slow walk hangs,
   the full-speed run-through hangs at exactly the [L] spot (13.24, 15.76, −153.65), and the
   pre-edge grounded jumps (z −153.2 / −152.6) land ON the railing's top (y 16.70, grounded) and
   stick — the reach-2 deck chain is dead in the port as in the game [L].
4. **THE REAL ROUTE, walked end-to-end: the mezzanine's annex vent.** The vent SS #180
   `ventmesh` + the blocking poly #184 `VentmeshBlockingPloy` hang in the corridor's CEILING at
   (53.8, 5.5, −163.59) with the Q-Laser latch #182 `VentLatch` at (53.80, 5.90, −165.36) [D
   #182 p5 = 0x54; Lock_Create 0x1ae1f8] — all above the mezzanine deck (5.44..6.03, x 52..56,
   z −176..−152 [D reach-1 scans]) — i.e. INSIDE the corridor, SOUTH of the barrel row. The
   route (all real input): the deck → its internal 0.56 step-down at z ≈ −161 (the walk holds —
   the game's own lip-hold [L]; a grounded jump crosses) → the held-Q-Laser burn of #182
   (Lock_Update 0x1ae3d8:82-101's held-fire path; +144 ticks) → ch 0x54 → SSs #180/#184 sweep →
   the drop through the open vent lands in the corridor at (54.23, 2.02, −163.54) (the floor
   0.90 [D secretreach]; the fall is past the row) → the corridor walk south → the plates:
   ch 2 under Bond → the E-arm → the AND mux #560 (out 8, inputs 2, 3 [D]) → ch 8 → Door_secret1
   #544 sweeps spent open → the yard → the walkreal's existing real legs to the 0d hand-off.
   `walkreal_0700000c_live` leg 4 now WALKS this: the `[plate2 + 0.6]` placement is DELETED.
5. **The one stand-in left: the mezzanine REACH.** The port's statics flood-isolate the deck
   (reach-1 comp 3; the east climb islands; the lip seal above), so leg 4 opens with ONE
   documented placement onto the deck (54.0, 5.8, −158.0) — cited inline — with the ground-route
   sisters (the deck floor, the latch's existence, ch 0x54 off) holding before the real legs run.
   How the GAME reaches the mezzanine is live check 1 (the grapple points gp0 (61.13, 12.28,
   −143.48) / gp1 (77.45, 12.12, −143.95) hang over the warehouse [D #308/#346/#500/#508/#511]).
6. **One incidental fix:** the final 0d gate #556 Load_Level (centre (96.92, 12.62, −209.71) [D],
   target 0x0700000d, key ch 0x1c) was missed by the walk's old r(1.0) approach after the new
   legs' tick shift — the gate plane walk now targets the placement's own centre (96.8, −209.6,
   r 0.4). The E press loads 0700000d on this tree.

## Verification (the runs I made, this tree)

- `walkreal_0700000c_live`: **2 passed, 0 failed** (20.65 s) — the full mission: the hinge burn,
  the vent drop, the hall, the photograph, THE VENT ROUTE (the latch burn +144 ticks, the drop
  past the barrel row), the plates, the door, the yard, the ladder, the wires, the band, the gate
  → the 0d hand-off.
- `interactables_0700000c_live`: **5 passed, 0 failed** (11.24 s).
- `walkreal_0700000c_barrelrow_live` (new): **1 passed, 0 failed** (9.90 s).
- The probes (diagnostic, --nocapture): the lane probe (the port's per-lane stops), the lip probe
  (the seal), the vent-route probe (the chain end-to-end), the corridor placement dump ([D] rows).
- The fast runner: started at 13:58 UTC on this final tree; the sandbox disk had to be pruned
  first (the previous session's tools died at 100% full — 4.2 G of stale test binaries in
  target/release/deps removed), so the runner is rebuilding the 984 test binaries from cold and
  will not reach a total line inside this window; the next session reads `tail suite.out`.
  My targeted runs above cover the binaries this task touches.

## What's left

1. The mezzanine REACH (the stand-in) — live check 1 decides whether the game grapples (gp0/gp1)
   or climbs something the parse cannot see; then the stand-in dies and the walk is placement-free.
2. The suite's total line on the final tree (the runner is alive; read suite.out).

## Live checks for the main agent

1. **The mezzanine access.** Fresh boot: teleport-free — can the game's Bond get onto the
   mezzanine deck (5.44..6.03, x 52..56, z −176..−152)? Try the Q-Claw from the warehouse ground
   at (61, 0.5, −143) into grapple point gp0 (61.13, 12.28, −143.48) [D #308/#346/#500/#508/#511
   — the class 0x51 targets]; record the ride's landing, then whether the 8.71 floor (x 53..62)
   connects DOWN to the mezzanine (a 3.27 m walk-off) and whether the mezzanine's deck at
   z −152..−157 is walkable to the latch spot. Also try the pallets #497/#498 (0x3b) and any
   crate stack against the deck's north face.
2. **The barrel row's east end.** Fresh boot: put Bond at (54.3, 1.0, −144) and push STRAIGHT
   south (no steering). Does the body slide EAST past the row (the port slides to x 61.35 at
   z −150..−152) and reach the warehouse ground south of the apron slab — or does the game's row
   seal to the apron? This decides whether the port's east gap is the game's own.
3. **The vent chain on hardware.** From the mezzanine at (53.8, ~5.7, −164.5): burn the VentLatch
   #182 (53.80, 5.90, −165.36) with the Q-Laser (ch 0x54 rises), watch SSs #180/#184 sweep, drop
   through the hole — the port lands at (54.2, ~1.0, −163.5). Confirm the game's landing spot and
   that the corridor walk south reaches the plates (ch 2/ch 3) and the door #544 sweeps.
4. **The mezzanine step-down.** At the deck's z ≈ −161.2 internal step (6.0 → 5.44): does the
   game's walk hold like the walkway lip, and does a grounded jump cross (the port crosses at
   z < −160.4)?

## Usage

- The session's own input/output token totals, context used and cost: not visible to me.
- The research subagent's Task-tool result printed its usage (read from its return message): the
  barrel-class research — `Tokens -> prompt:44744 completion:2274 total:47018`. The first
  (larger) research subagent timed out before reporting. No other credit/context figures are
  shown in this environment.
