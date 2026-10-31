# level-0d-jib-walk-off — report

Branch `s/level-0d-jib-walk-off` (from origin/master 02e4bc18, merged with origin/master
7d0091a0). Task: finish the level-0d crane deck → yard chain on real input — (1) the deck-edge
walk-off onto the walkway strip, (2) the dock walk + E #460 in-route, (3) the beam boarding via
the north steps and the roof/catwalk onto SP #115's westward pass, the ride west and the drop
into the yard — replacing the remaining teleport in
`walkreal_0700000d_live::walk_arrival_to_the_c5_door`.

## What's done

**Legs 1 and 2 are REAL INPUT now; the old single teleport is replaced by two NARROWED,
geometry-pinned stand-ins.** The chain in the test reads: … the swing → **the walk-off (real)**
→ **the catwalk walk west (real)** → stand-in 1 (the catwalk→bay exit) → **the dock E #460
(real)** → stand-in 2 (the beam boarding + west ride) → the yard legs 2-5 (unchanged, real).

1. **The walk-off is solved and deterministic.** The walkway strip is a STATIC map object at
   12.13 that forms a DIAGONAL band across the crane's rest-axis frame: r 5.5–10 with the
   band's lateral centre sliding from off +1.0 (r 5.5) to off −1.5 (r 10) [probe20_static_map
   rays], merging into the east-west catwalk x 157..196 at z −265.0..−266.25. The rim-gap
   window's miss (~0.13 m) came from aiming at a constant off — the band moves with r. At the
   swing's END pose the deck's rim on the strip bearing is OPEN (the rest-pose rim stalls the
   run — probe17 v4's committed finding — but the end-pose rim does not), so the walk-off runs
   at end pose: pre-turn in place (the post-swing ground carry leaves Bond facing away; walking
   while turning wanders into the mast), then forward held with re-aim at the strip point
   r 6.0/off +0.75 (the band at r 6.0 spans off +0.5..+1.0). The run's carry (~2.5 m through
   the 0.91 m fall at the run speed, gravity 0.0045/tick² [probe17 v3's traced fall; the
   0.9 m/20-tick fall in probe20's traces]) lands Bond ON the strip at r 5.99/off +0.51
   (pos y 13.22 = surface 12.13 + the ~1.1 pos offset) — reproduced across every run this
   window (probe20_walk_off runs 2/3, probe20_leg2_dock runs, and the walkreal itself).
2. **The catwalk is walked west, real input**: (195.49, 13.22, −265.71) → (161.1, 13.17,
   −265.70) through three walk_to waypoints. Its geometry is fully scanned: south face wall at
   z −266.26 for the body band (y 12.6/13.25) along x 156..196, west end wall ~153.5–157, the
   x 162–163 floor slit is ~2 cm wide (the capsule bridges it — Bond walks across at 12.1x).
3. **The dock E #460 is real input**: from the E-proven bay approach the test walks the use
   (use_ticks 240) and asserts ch 0x2b [D #460 p0; M Trigger_Activate 0x1b2318:23-33], which
   arms SP #115 `ScriptPlayer_ShippingCrane` (mode 8, mask 1, out 0x2b, 3000 ticks [D]; the
   held state-6 arm releases on the edge [M 0x1dbda8 state-6 `s.delay = 0`]) — the same chain
   the interactables test pins.
4. **No port gap was found in nf-game — every physics suspect examined came out faithful**, so
   nothing was fixed in src: the jump is vertical-only ([M Player_HandleJump 0x1916b0 zeroes
   the velocity then adds the fixed WldGravity-derived impulse; the caller Player_SSWalk
   0x192310:69 passes WldGravity @0x2d8890 (D ELF symtab)] — re-confirmed this window), the
   script-entity collision posing works (the deck rotates and carries [probe18], the beam
   poses [probe20 beam watch]), and the catwalk/strip collision is the disc's own static
   geometry [probe20 rays].

## What is NOT done (the two stand-ins, with the pinned geometry)

1. **The catwalk→bay exit.** The catwalk (12.13) is SEALED at walk height: the south face wall
   z −266.26 runs its full length, the west end walls at ~153.5–157, the floor slit is 2 cm.
   The dock bay (floor 9.0–9.5, x 168..194, z −268..−292) sits 3.1 m below; the steps island
   (12.7, x 188..189.5, z −268.75..−269 — the staircase 12.67→11.92→11.13→10.35→9.56 lands
   there) and the 14.7 pad (x 186..192.5, z −269..−273) are across the bottomless slot
   z −266.5..−267.5 [probe20 slot/wall scans]. The building's interior shows 3.48/7.98 surfaces
   at x 186 under the bay and a 10.4 step at x 193 [probe20 interior scan] — the game's own
   exit is likely inside the building (or via the roof, which carries the y ≈ 19.7 catwalk
   squad [D census]); the interior walk is the next window's first job.
2. **The beam boarding + west ride.** The beam/trolley (script 0x060009EF entities
   0x0200_0a80/0x0200_0a81; origin y 11.33, z −287.11; shapes ±4.17/±2.98/±17.01 and
   ±2.44/+2.95/±16.80 [D]; top 13.47–14.32) shuttles x-only 137.54 → 177.22 (t ≈ 1260) →
   137.54 (t ≈ 2670) over 3000 ticks — y never changes [probe20 beam watch] — so the bay floor
   (9.5) is ~4 m under the top and the boarding is not a bay step. The trolley's top (14.28)
   matches the 14.7 pad, but the shuttle's east limit (x 177.22; the beam then spans
   x 173.05..181.39) never reaches under the pad (x 186..192.5). The 16.9 mezzanine
   (x 178..183, z −270..−283) and the roof (18.58) DO overhang the east-extreme position — the
   player's boarding climb reaches the roof/mezzanine first (the access is the open question,
   same as the bay exit — likely one interior route serves both). The stand-in covers only the
   ride + the drop into the yard.

## How it was verified

- `walkreal_0700000d_live` — **1 passed / 0 failed (27.3 s on the final merged tree; 36.6 s
  pre-merge)**, `stuck 0; falls 1` (the fall is the walk-off itself). The walk-off, the
  catwalk walk and the E #460 are real `Sim::step_bond` input; the two stand-ins are
  `bond.pos` writes with the pinned geometry in their comments.
- `interactables_0700000d_live` — **11 passed / 0 failed (30.6 s on the final merged tree;
  41.8 s pre-merge)** — the crane-side mechanics the route relies on (E #450/#460, the SS, the
  SP arming) all re-verified on the merged tree.
- `probe_crane20` (committed, print-only, `--ignored --nocapture`): `probe20_static_map`,
  `probe20_fine_maps`, `probe20_walkway_scan`, `probe20_beam_watch`, `probe20_roof_scan`,
  `probe20_leg2_dock`, `probe20_interior_scan`, `probe20_west_scan`, `probe20_walk_off` — the
  scans every number above cites.
- The full suite (`tools/test/suite.py`) ran twice: the first run (started 05:27 UTC, the
  pre-edit tree) was killed at the merge — its partial read 9 FAILED/CRASH lines, all
  environmental (the 3 disc_golden + identity_disc on the slim ISO, and exit -9
  TIMEOUT-kills: mp_combat_weapons_live, frame_cost_live, fix_0c_pylon_live); the final-tree
  run was started at 06:43 UTC — **the window ends before it prints a total**; its partial
  line and any FAILED/TIMEOUT lines are quoted below when it advanced. No src changes are on
  this branch (tests + docs only), so the touched-binary totals above are the regression risk.

## What's left (the next window's list)

1. Find the building interior's route from the bay (9.5) up to the roof (18.58) / the 16.9
   mezzanine — the interior scan's 3.48/7.98/10.4 surfaces are the ladder/stair candidates;
   walk it on real input and delete stand-in 1.
2. Board the beam from the roof/mezzanine over its east-extreme position (x 173..181.4, top
   13.47–14.32), ride west to x 137.54, drop into the yard at (~137.5, ~14.3, −28x) — the
   existing yard leg 2 starts at (136.5, 4.5, −290) and feeds straight in; delete stand-in 2.
3. If the interior route needs a port fix (a ladder/step class), cite it [M]; the walk-off
   needed none.

## Live checks for the main agent

1. **The catwalk's exit** — fresh boot 0700000d, walk Bond along the crane dock's catwalk
   (12.13, z ≈ −265.7, x 160..195) west and watch how the real game descends to the dock bay
   (9.5). Address of interest: the catwalk's south face (z −266.26) and the building's interior
   under the bay (the 3.48/7.98 surfaces at x 186). Expected: a doorway/stair the port's ray
   scans read as wall — if the real game walks through, the port's static collision read is
   missing a face.
2. **The beam boarding** — fresh boot, E #460, wait for the beam's east extreme (t ≈ 1260),
   then watch where the player can stand to step onto the beam top (14.32 at the edges,
   13.47 mid). Expected: the roof (18.58) or the 16.9 mezzanine (x 178..183) — confirm which
   surface the game's route uses and how it is reached from the bay.

## Usage

not visible to me — this session's environment shows no token, credit or context-window
figures (no input/output token counts, no context-used meter, no cost display was available
to copy).
