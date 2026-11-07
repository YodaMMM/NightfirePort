# level-0c-vent-crawl — the vent-shaft trap identified, the shaft mapped sealed, the game-side question filed as live checks

## What was asked

Near (5.33, 12.49, −110.94), behind the crates, Bond cannot crawl through the vent. Find the
crouch and duck rules, the grille, and the collision at the opening, and walk it end to end with
real input.

## What this session found

**The spot is the display-case vent shaft ("the bay"), and the player is trapped in it.**
(5.33, 12.49, −110.94) decomposes exactly [D probes + prior session data]:

- y 12.49 = the crouched root on the shaft floor 11.84: floor + 0.646 (cc 0.55 + the capsule's
  −0.10 bottom [M Player_Collision 0x194df0 case 3; the crouch-root model]) = 12.486;
- z −110.94 = the shaft's north wall plane z −110.39 minus the capsule radius 0.55 — the body
  pressed against the north wall;
- x 5.33 = the port's known bay-floor rest spot (5.33, −110.95), already carried as the Δz 0.43
  live check in `fix_0c_hatch_live.rs:175-178` (the game's fresh-boot drop rests at
  (5.42, 12.8773, −111.38) [L 2026-10-26 hatch-2] — 0.43 further south than the port's).

"Behind the crates" = the shaft is the space behind/north of the `InvPoly_StorageCrates` stack
#443 at (4.65, 12.56, −114.24) [D audit:323], whose tops (14.61/14.81, new probe column dumps)
connect the 11.27 mid floor to the 14.61 balcony.

**The route that puts the player there** (the game's own vent route, [D probes this session]):
the ST-1 walkway (floor 14.01 at z −107.5) runs south and ends at z ≈ −110.2; the shaft (floor
11.84, x 5.0..6.6, z −110.4..−111.9) is 2.17 below the walkway's south end; the vent hole
(y 15.03..17.42 in the wall plane z −112.25..−112.34, open at y 15.2..17.0 to the crouch rays,
solid 14.6..15.0 below and 17.4 above) sits at the shaft's south top over the sill stack
14.81 (z −111.9) / 15.03 (z −112.2). The player crouch-walks the duct through the burned hatch
(#138, the fix-level-0c chain) and steps off the sill lip into the shaft — a 3.2 m one-way drop
that fires plate #274 mid-fall (objective 1; the walkreal leg).

**The port's shaft is a sealed box at body height** (new probe
`crates/nf-view/tests/probe_vent_north.rs::probe_trench_north_map`, all [D port-collision
probes]): rays from inside the shaft hit at EVERY body height y 11.9..17.4 north (z −110.39,
x 4.8..6.6), and y 11.9..14.3 east/west (the x 4.78/6.74 walls). The only opening is the vent
hole, 3.2 m over the floor — unreachable (the jump apex is 0.78 [M player.rs]) and ladderless
(the level's ladders are the yard pair #164/#495 [D]). The crouched capsule (r 0.55, top =
floor + 1.30 [M 0x194df0 case 3]) fits nothing. So in the port, the player who takes the vent
route down cannot crawl back out — the complaint, reproduced as a state description; the
real-input walk of the last leg (walkway → off the lip → the shaft floor → the direction tries)
is the one piece this session did not finish (the naive south-walk wedged at the spawn — the
leg needs the `walkreal0c.rs` walker's route logic; the harness and the probe are on the
branch for the next session).

**The decompile side (crouch/duck/crawl rules), from the research pass over
`work/decomp/ACTION.ELF`** — all [M] with addresses, no divergence found in the crouch rules
themselves:

- Crouch is substate 4 (`Player_SSCrouch` 0x1924C0), entered by the duck action (8) from the
  walk substate (`Player_HandleJump` 0x1916B0:89-97) with the 60-tick lockout
  (`Player_ChangeSubState` 0x190630:47-48); stand-up needs a fresh press AND no ceiling flag
  0x20 AND a clear 0.95 LOS from the position snapshot (0x1924C0:84-105). The port mirrors all
  three (`player.rs:1697-1708`, `headroom` = the same 0.95 LOS).
- Capsules per substate (`Player_Collision` 0x194DF0:201-342): crouch r 0.55 (A = pos +
  0.1 − (cc − 0.55), B = pos + 0.45 − cc); walk r 0.55 (A = +0.275, B = +0.55 − cc); climb/creep
  r 0.275. The port's literals match to the f32 (`player.rs:706-713`).
- There is NO prone/belly-crawl and NO step-height or crawl-space volume test: blocking is the
  two-sphere capsule pushed out of geometry (`Intersect_CylGeom` 0x109DA0) plus the two 0.95
  headroom rays; a "crawl-through" low passage is emergent (crouch top ≈ floor + 1.30 vs stand
  ≈ floor + 1.91). The game's `Player_Creep` 0x1969D0 wall-hug needs a 0x3F CreepWall placement;
  0700000c has none [D kind-census] — the crouch is the only way through a low opening here.
- The differences the research pass did flag (none pinned to this symptom yet): the game's cc
  is a live anim-driven value (AnimObject+0xCC) where the port steps a constant pair; the port's
  headroom LOS mask is 0xd where the game passes 9 (`0x1924c0:101`) — bit 4 makes the port's ray
  skip disabled objects the game's would hit; the port's walk push keeps the unguarded
  through-push (a documented divergence); the game's ceiling flag 0x20 keys on any contact above
  the A arm while the port keys on push[1] < 0 (glancing contacts differ).

## How this was verified

- The decomposition of (5.33, 12.49, −110.94) — new ray/column probes against the port's
  collision world (`probe_vent_north.rs`, built and run green this session), cross-checked
  against the prior sessions' [L] rests (hatch-2/hatch-3) and the audit rows.
- The sealed-box map: the probe's A/B/D/F ray grids (N wall, the hole extent, E/W walls) — all
  solid at body height except the hole.
- The crouch/duck/crawl rules: the decompile function addresses above, read this session from
  `work/decomp/ACTION.ELF/`.
- NO engine code changed, so no previously-green test could move; the full four-crate suite did
  not fit this session (the release build alone exceeded the remaining budget — the sandbox
  restarts dominated the first hour). Nothing I touched is in any other test's path: the only
  files are the new probe test and docs. Next session MUST run the full suite before the fix
  lands.

## What's left

1. **The game-side answer** — live checks below. If the game's body also stays sealed in the
   shaft, the port is faithful and the report's answer to the player is the game's own: the
   vent route is a one-way drop; the route on is the balcony SE → east deck 12.14 → staircase
   #359 → the ground (the mapped descent) — reached by NOT taking the drop (walk the balcony
   south). If the game lets the body out N/E/W (or through a grille), the port's wall is the bug
   and the next session fixes it against the decompile.
2. The real-input walk of the last leg through the `walkreal0c` harness (no teleport), red/
   green per the answer to 1.
3. The bay-floor rest-region divergence (port −110.95 vs game −111.38) — the cel-glist floor
   shape (carried live check; display-hatch-3's "What's left").
4. The roomvent #480/#481 grille positions (CEL transforms live in the glist, not the audit) —
   if one is embedded in the shaft's walls, it is "the grille" of the task text.

## Live checks for the main agent

Each: fresh boot, load 0700000c, take the vent route down (walkway south end → the open hatch
hole → off the lip into the shaft), Bond rests on the shaft floor 11.84. Then:

1. **The north walk** — hold stick north (toward the display-case wall) for ~10 s.
   Expected if the port is right: the body holds against the z −110.39 wall (like the measured
   south hold [L hatch-3]); record the held z. If the body passes z −110.9 and continues north
   (through the wall plane, into a grille opening), the port's north wall is the bug.
2. **The east/west walks** — hold stick west (toward x 4.78) then east (6.74) ~10 s each.
   Expected if the port is right: holds at the shaft's side walls; record the held x. A pass
   through (a drop into the case interior 6.27) would be the exit the port lacks.
3. **The grille watch** — a `gdbwatch.py`/`fieldprobe.py` look at where the roomvent
   (`roomvent`/`roomvent2`, placements #480/#481) meshes sit in world space and whether any
   collision triangles of theirs (or of a wall behind them) fall inside the shaft's N/E/W wall
   planes (z −110.39, x 4.78/6.74) between y 11.84 and 14.6. If a roomvent grille is set into
   one of those walls, name which wall — that is the grille the player sees.
4. **The eject check** — after the drop, leave the stick neutral ~30 s and then walk north/
   east/west: does any script, checkpoint nudge or volume move the body out of the shaft (the
   trench-exit live check 1, still open)?

## Usage

not visible to me (no token/credit/context-window figures are exposed in this environment).
