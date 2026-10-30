# level-14-mezzanine-descent-2 — the mezzanine descent is WALKED; the delta-exit walk is un-ignored and green

Branch `s/level-14-mezzanine-descent-2`, base origin/master 774b1cbf, then merged origin/master
814c8976 (batch-1029r/s: the crane transit, the cascade servers, the wreck draws). Window
03:39–05:1x UTC. Task: walk the mezzanine descent into the facility on real input, fix any port
collision gap [D/M], un-ignore
`crates/nf-view/tests/walkreal_07000014_live.rs::walk_the_infiltration_to_the_delta_exit` with
the rest of the walk green, run the FULL suite and quote its total line.

## What's done

**The descent is walked, on real input, and the test is UN-IGNORED** — the walkreal_07000014
binary ran **6 passed / 0 failed / 0 ignored** (was 5 passed / 1 ignored).

The route (every leg real input through the shared Walker; the probe that found it is the
untracked scratch `crates/nf-view/tests/scratch_l14d_live.rs::l14d_door834`):

1. The [M] hand-back stands Bond on the mezzanine pad (≈ (40.7, 2.8, 39.0)). West across the pad
   to (34.63, 2.80, 37.94), south to (34.57, 2.80, 37.18), the 0.43 step down onto the 1.72
   landing at (33.09, 2.37, 35.38) — the level-14-mezzanine-descent window's walked legs,
   reused unchanged.
2. West off the landing's edge onto the **low machine floor**: the floor measured **0.80** under
   every walked point (x 31.95..32.94, z 34.28..35.36) [D rays through the port's disc-cel
   collision]; the grid band 0.2..1.4 spans x 31.5..33.5, z 33..35.5 [D rays].
3. Down onto the **facility corridor floor (0.0)** [D rays] — the corridor's own nav lane runs
   nodes 57/58/257-265 at x 27.3..28.3, disc y 0.0 [D nav dump] — then south to the leg-2 start.
   The test's descent landed **(29.81, 0.64, 37.65)**, inside the leg's assert box
   (x 27..33.5, root 0.5..1.6, z 36.5..40.5 — the box the old [S]-teleport route aimed at; the
   walk reaches it without any teleport).
4. Two real-input pieces the walk needed, both walked observations from the test's own logs:
   - The low floor's east margin sits **under the landing slab's overhang**: a grounded wedge at
     (33.68, 1.15, 35.20) with a CLEAR segment to the target (the walker's stuck dump); escaped
     with the player's own push — steer south-west around the slab corner, hop when grounded
     (the walker's wedge recovery). The overhang mechanism is [S] inferred (the wedge and the
     clear segment are measured).
   - The door wall's **west jamb bars the direct south steer at x ≥ 30.3** (the segment to the
     target hits 0.17 along from (30.33, 0.55, 35.44) [D rays]) — the corridor crossing walks at
     x ≈ 29.7, which is also where the probe crossed ((29.99, 0.61, 38.00) grounded).

**No port collision gap found on the route — nothing in `crates/` changed.** The descent route
exists in the port's disc-cel collision as-is. The prior window's blockers ("the wall at x 31.5,
z 38.4..40.8", "no walkable strip south of z 38.4") were the pad's south-west steering trap, not
the route: the descent exits **north-west from the LANDING**, not south-west from the pad.

**The east lane / machine room is NOT the descent.** The machine-room patrol webs link only east
(path 1 node 1 (43.05, 0.4, 42.81) → n2 (49.1, 0.4, 42.8)) and south (n0 → n7 (37.4, 0.4, 53.0));
path 0's machine-room loop (n44-47-48-49-50) reaches the corridor east via
n140→n52→n45→n42→n43→n41→n40 [D nav dump] — no west link anywhere, matching the prior window's
west walk bound at x 37.1. The #330/#331 grapple/icon spot under the lane stays the machine
room's climb-back marker, not the facility descent.

**Door #834** (plc 834, sim kind 2, group 0, ch_open 0, ch_lock 0 [D params]; leaf
x 32.00..32.65, y 1.76..2.52, z 35.72..35.82 [D]) **opens with E** — the probe's use press moved
it to state 0x4, pos01 1.00 — but the walked route passes WEST of its wall run (the leaf plus the
fusebox column #833 x 31.93..32.65 [D]); the leaf plane is never crossed. Its role (route vs set
dressing) is the live check below.

## How it was verified

- The probe `scratch_l14d_live::l14d_door834` (untracked scratch, not committed) teleported Bond
  to the walked landing spot (teleport only to set the probe start, outside the #301 touch
  volume) and walked the whole chain green, landing (29.99, 0.61, 38.00) grounded; its runs also
  produced the [D] evidence above (the two-deck ray map, the nav-web dump, the door open state).
- `walkreal_07000014_live` on the pre-merge tree: the single test green (30.47 s), then the full
  binary **6 passed / 0 failed / 0 ignored** (187 s).
- After merging origin/master 814c8976 (scene.rs +49 changed under nf-view): the full binary was
  re-run on the merged tree — see the suite line below.
- Crates run: `nf-view` (the walkreal_07000014 binary, then the FULL suite below). No nf-game /
  nf-formats / nf-audio code changed.

## FULL suite (fast runner, final tree = origin/master 814c8976 merged + this branch)

SUITE_RESULT_PLACEHOLDER

## What's left

- The door #834's role (live check below).
- The machine room's west exit: no nav link exists [D]; if the game has one it is player-only
  and off-web — the descent does not need it.
- The lane drop into the machine room (the prior window's stuck steer at (45.13, 43.58)) was
  NOT needed for this task and remains unprobed; the machine-room interior probes
  (l14d_lane_drop) are in the untracked scratch only.

## Live checks for the main agent

1. **The descent in the real game**: fresh boot 07000014, let the vent scene end at the
   mezzanine, then play down. Does the player walk pad-west → landing → west off the landing's
   edge (this port route), or open door #834 and go through it (its leaf plane is crossed in the
   game)? A gdbwatch on `glb_players+0x30` (write) sampling the descent gives the route points.
2. **Door #834**: stand on the 1.72 landing, press E at (32.3, ~2.1, 35.8) — the port opens it
   (state 0x4, pos01 1.00). In the game, what is through it — floor or the shaft? (The port's
   rays show the notch south of it as void [D rays], and the game's nav never crosses the notch
   either [D nav].)

## Usage

not visible to me — this environment shows no token, credit or context-window figures (no
status line or /usage output available); nothing to copy, and per the task's rule I did not
estimate.
