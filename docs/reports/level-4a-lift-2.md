# level-4a-lift-2 — 0700004a: the lift hand-back into the lobby (round 2)

## What was asked

`walkreal_0700004a_live::ride_the_lift_from_the_spawn_and_walk_into_the_lobby` is still
`#[ignore]`: the post-enable hand-back was the blocker. Settle the hand-back position the game
uses [M Player_Enable 0x195498, the event-6 snapshot] — if it needs a live read of Bond's
position at re-enable, name it exactly. Real input only; un-ignore when it passes, with a sister
that can fail.

## Done: the hand-back POSITION is settled — every link read in the disasm/decompile this round

The composed value the previous round shipped is **correct**: the game re-enables Bond at
**(1.799, −107.662, −6.830)** — SP #289's base composed onto the unparented marker's key. The
chain, with the new links in bold:

1. **The base is the SP placement**: SP_LoadScript passes the SP object's +0x30/+0x50 (pos/rot)
   into Script_Load [M 0x1dbc08:37-40], which builds SCRIPTINFO+0x00 =
   `View_RotTransMatrix(rot, pos)` [M 0x1d8248:148] = SP #289's placement (0, −53.534, −3.377),
   rot 0 [D placement].
2. **The polarity proof** (the round's core): Script_Interp's case-1 `Script_GetInterp` call
   passes its 9th argument (the stack char — the ABI is f12,a0,a1,a2,a3,t0,t1,t2,t3,0(sp)) =
   `(stream parent == -1)`, computed `lb $v1,0x25($s1); nor; sltiu; sw ($sp)` [M disasm
   0x1d9a7c-0x1d9a98], **and Script_GetInterp's tail runs `Concat(SCRIPTINFO+0x00, out)` when
   that flag is non-zero** [M 0x1da158 tail]. So the UNPARENTED marker stream (s25, parent None
   [D 0x060006cf]) composes the base INSIDE GetInterp; the parented path composes it once at the
   chain end [M 0x1d9a08:114-135]. Both paths compose the base exactly once — the early
   fix-final-blockers2 "world-space keys" reading was wrong; the landing-matrix round's compose
   stands. (I initially re-flipped the polarity from the case-1 caller C alone — the callee's
   tail settles it; recorded in WORKLOG.)
3. **The snapshot source**: the event-6 handler reads the PREVIOUS stream's object — the
   handler's stream base is +0x80 while Script_Run walks streams at +0xa8, so
   `info + 0xab3*0x28 + 0x80` = stream[i−1]; the kind gate (1/2) and stream->0x10 give the
   marker's object [M 0x1d94c0 case 6 + 0x1d8548]; with no such object the default is Bond's own
   matrix (glb_players+0x90). The marker's obj+0x90 → SCRIPTINFO+0x40.
4. **The apply**: Script_Free sees +0xaa8 bit 0x100 and calls
   `Player_Enable(glb_players, SCRIPTINFO+0x40, +0xab6)` [M 0x1da880:46-49]; Player_Enable
   writes the matrix translation VERBATIM to obj+0x30 (copied to +0x40), yaw/pitch from
   Mat_GetDir, zeroes the velocity vectors, sets substate 2, and its param-3 sphere scan handles
   only wire/creepwall cels — **NO ground snap inside the enable** [M 0x195498:54-89].

So no live read of the position is needed — the decompile chain settles it. The port already
does exactly this.

## NEW: the composed spot floats in OPEN AIR — the real blocker, sharpened

The disc's collision has **no floor in the hand-back column**: floor_below from y −104 and from
−107 down 400 units is None at x 1.8 / z −6.83; the bottom −156 slab ends at z −1..−1.5; no
walls within 30 units horizontally (the walkway edge is 5.5 north) [L probes
walkreal_probe_l2b/l2d — the port's world is built from the disc's own object cels via
`build_scoped` over `level.instances(true)` with nothing skipped]. The guards' −104 walkway (x
−4.34, z −0.58 — drones #250/#251's floor [D census]) exists and was probed Some(−103.99) — the
floors are real, 6 units north of the hand-back. The previous round's "real floors the port's
rays miss at x 1.80/z −6.83" was an inference from the guards' census; the probes show the rays
miss because the disc has nothing there, not because the port dropped geometry.

Consequence: the port's Bond falls forever after the enable (probe l2: airborne from t2188, y
−989 at t3501, hp 100, floor8 None throughout). The walkreal's ride → landing → lobby route
cannot pass until the SHIPPED game's behaviour after the enable is known.

## Live checks for the main agent (fresh boot — save-state resumes don't count)

1. gdbwatch the player object's position `glb_players → obj+0x30` (x/y/z — the matrix
   translation Player_Enable writes [M 0x195498:60-63]) through t2190..2800 (the −52 landing
   scene's end + the fall). Expect the write to (1.80, −107.66, −6.83) at the enable tick, then
   the fall trajectory and WHAT ENDS IT: a landing on the resumed cab (the cab passes y −107.66
   at ≈t3950 if Bond still hung there — but he out-falls it), the −108 walkway (6 units north —
   he has zero velocity [M Vec_Zero at 0x195498:81-86], so he can't drift onto it), a void
   recovery, or death. Also watch `BLData+0x94a` (the substate Player_Enable sets to 2) and the
   health byte for the fall's outcome.
2. A Collide_RayIntersect straight down from (1.80, −107.66, −6.83) in the real game: does the
   game's room-scoped collision find a floor the port's flat world reports None on? Expected per
   the port's census: None — and then the catch must be outside the static cels (a script, a
   moving solid, a game-side reset).

## Tests

- **NEW sister, GREEN (no re-seats anywhere)**:
  `walkreal_0700004a_live::the_handback_lands_bond_at_the_composed_marker_and_the_column_is_open_air`
  — boards with real movement input, rides with none, the landing scene plays un-skipped, and
  the first post-enable tick places Bond at the composed matrix (±0.05/±0.5); then pins the
  column's open air (floor_below −507 None) against the guards' floor (Some) as the control. It
  fails if anyone reverts the composition (Bond would land at the raw key −54.13) or if collision
  appears in the column.
- `fix_landing_matrix_live` — still GREEN (1 passed), unchanged.
- The walkreal itself stays `#[ignore]` with the sharpened blocker + the exact live checks in
  the ignore message (the honest state: the position is settled; the post-enable world needs the
  fresh-boot measurement before the scrape/fall-exempt logic can be reworked).
- Probes committed as the evidence trail: walkreal_probe_l2 (the ride + the fall + the disc's
  mid-landing placements), l2b (the shaft's static census: floors only on the z 0..2 walkway
  strips at −12/−100/−104/−152...), l2c (the wall rays at the hand-back vs the guards' level),
  l2d (the deep-pit scan: the −156 bottom, the 400-unit None).

## What I ran

`cargo test --offline --release -p nf-view --test walkreal_0700004a_live` → 1 passed, 1 ignored
(the parked walkreal). `--test fix_landing_matrix_live` → 1 passed. `--test fix_finale_live` →
4 passed, 0 failed (includes `the_0700004a_lift_ride_reaches_the_lobby`). No src changes — the
port already matched the game chain, so nothing else could regress.

## Full suite

`tools/test/suite.py` was started in the first minutes per the brief, but background jobs are
killed between tool calls in this sandbox (the level-1b worker logged the same) — the first
start died silently. Restarted at 20:49 with setsid: build ok (734 test binaries in 743 s), the
run is in progress at the deadline. Four binaries CRASHed with exit -9 (OOM) — my own concurrent
foreground cargo runs starved the box while the suite ran; that is my fault, not the tree's.
Foreground chunks on the final tree (above) are the verified results; see the suite total below.

## Suite total

The run did not finish inside the window (734 binaries; the 743 s build plus the OOM churn ate
it). At the final read (21:33 UTC): **287 binaries completed — 944 passed, 0 failed**, 0
TIMEOUT and 0 FAILED lines, 8 CRASH lines (exit −9, OOM — my concurrent foreground cargo runs
starved the box while the suite ran; my fault, not the tree's — the four early ones are listed
above, the rest are the same cause). The task-relevant binaries all green in the suite's own
run: walkreal_0700004a_live 1 passed + 1 ignored (the sister + the parked walkreal),
walk_0700004a_live 1 passed; plus the foreground chunk (fix_finale_live 4 passed,
fix_landing_matrix_live 1 passed). The 0700004a interact binary was still pending in the
suite's order at the deadline.

## Left for the next round

- The live checks above — they decide the post-enable world and unblock the walkreal rework.
- The walkreal's scrape/fall-exempt logic reworked around whatever the game does.
- If the game's catch is a port-side gap (a room-linked cel, a moving solid, a script), port it
  with its [M]/[D] evidence and un-ignore the walkreal.

## Usage

Context/input/output token, cost and credit figures for this session are not visible to me — no
token or context-window readout is exposed in this environment, so there are no numbers to copy.
