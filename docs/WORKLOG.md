
### level-0c-display-hatch (started 04:28 UTC, 2026-10-25)

Task: 0700000c — two `fix_0c_hatch_live` tests are ignored. (1) `bond_crouches_through_the_open_
vent_to_the_display_ground`: leg 1, the ascent to the duct top, unidentified — the game's climb is
ladder-only [M 0x196118] and the vent has no ladder [D]; find how the player reaches the duct top
in the game [D + M — crates/ledges/jumps, script or lift, the Q-Claw grapple points on 0c] and
walk it with real input. (2) `from_the_duct_top_the_open_hole_carries_bond_down`: legs 2-3 work
(the open hole drops the crouched capsule onto the inner ledge 14.72); leg 4, the descent into the
display case (floor ~7.3 at (5.4, -113) under the 11.27 mid floor) is not mapped — map and walk
it, fix any port divergence [M]. Un-ignore both tests as they go green.

Plan: branch s/level-0c-display-hatch from origin/master e6e3c899; work-slim + ISO downloads;
fast suite runner started early; census 0c floors/crates/lifts/grapple placements [D] + the
climb/ladder/jump decompiles [M]; real-input walks through the real path; un-ignore as green;
report + finish.

- (04:40-05:00) Setup done (work-slim 36689089, ISO 369823203 extracted, rust 1.99.1, ALSA user
  prefix /home/z/alsa-root incl. libasound2t64 runtime + patched alsa.pc prefix, cargo fetch).
  Background suite runner dies between tool calls (as rook-door-3 recorded) — restarted twice.
- (04:40-04:50) Two parallel Explore agents: (a) ascent mechanisms [M]: jump = WldGravity·(−0.4)
  → +3.92 vel, max gain 0.784 m (Player_HandleJump 0x1916b0:166-183, Player_Collision 0x194df0:105-116);
  no step-offset constant — effective step ≤0.55 m, ramps with floor ny ≥ 0.5 walk for any height;
  climb entered ONLY by class 0x2D ladder contact (Player_CollLadder 0x196118:44 via
  Player_DealWithObjHit 0x194458:108-112; census of all ChangeSubState callers — no other +0xF6
  writer); lift carry exists (Control_InheritVelocity 0x133e88:74-79) but no lift placements at the
  vent. (b) grapple [M 0x195da0/0x192860]: pull = pos += n̂·min(+0.05t,1.5) direct position write,
  hang until fire released; port matches except a MISS gate (rope-fire-anim done + 15-tick timer →
  phase 4 mid-pull detach) the port omits; 0c grapple points #308/#346/#500/#508/#511 all far from
  the vent (nearest #500 z −189.7) — grapple eliminated for leg 1.
- (05:00-05:15) [D] placement census (nf-extract placements-all 0700000c.bin): the vent bay is
  closed — walkway floor 14.01 ends z −110.02 (wall face 13.33..17.13); bay floor 11.84
  (z −112.34..−110.39, x 4.78..6.74) with ROOF 17.28/17.42 (x 4.20..7.32) = the "duct ramp top
  17.2-17.5"; wall opening 15.03..17.42 with sill lips 13.57/14.81/15.03; balcony 14.61 north edge
  AT the wall plane (z −112.34); mid floor 11.27 (z −112.37..−115.50, x 1.88..5.71) + 11.45 east;
  ground 6.27 (z −109.69..−116.76, x 2.61..5.71) UNDER the mid floor with the jetpack #414 at
  (5.50, 7.55, −110.37) under the bay; crate stack (InvPoly_StorageCrates #443 (4.65, 12.56,
  −114.24)) tops 12.47..14.27; winch cars DEAD as before.
- (05:15-05:25) [D] THE SKYLIGHT: three kind-0xDC touchplates "touchplate_skylight01" (#275-277,
  trigger kind 2 → ch 27) + Touch triggers #274 (out ch 27) / #476 (out ch 102), both lock ch 8,
  at (5.78, 14.02-14.09, −111.05) INSIDE the bay — the bay roof (17.28/17.42 over x 4.2..7.3,
  z −112.3..−110.4) is the display case's GLASS SKYLIGHT; the case is the bay pocket itself
  (floor 6.27/ramp 6.35→7.7, jetpack inside). The touchplate kind-2 = break-on-touch is the game's
  route INTO the case.
- (05:25-05:35) Standable-surface maps (segment_hit grids, closed + open state, nf-view examples
  probe_0c_standmap/probe_0c_openmap): walkway ramp 12.3→14.0 northward; roof 17.2-17.5 flanked at
  z −110.5 by a STATIC 16.3 band spanning x −2..14.5+ (not the blocker — persists in the open
  map); west/east ledges 15.2 at z −111.0; balcony 14.6 south. Roof access still open: 16.3 →
  17.2 needs 0.9 (jump max 0.784 [M 0x1916b0]); ledges 15.2 → roof 2.1. Live check filed: the
  16.3 band's identity + how the real game gets Bond up. Sim walks: leg-1 walk stalls at the
  walkway end wall (z −109.47, capsule 0.55 short of the z −110.02 face); leg-2/3 walk crosses
  the opening onto the balcony then OSCILLATES (15.62/−120.85 ↔ 15.35/−120.67) at the pier ramp
  base — the leg-4 descent (sill → mid floor 11.27 → ground 6.27 → ramp north into the case) is
  the mapped candidate.

## level-0c-display-hatch-2

- (17:01 UTC, 10-26) Branch created from origin/s/level-0c-display-hatch (b828d267), merged
  origin/master. Task: use the driver's four fresh [L 2026-10-26] answers (the bay floor reads
  12.877 not 11.84; the skylight touchplate ch 0x1b pulses on a body passing through its volume
  — the drop into the bay fired it at ~14.1; the roof walk never fires it; the mid floor's north
  edge holds Bond back from the 6.27 pocket) to find the real route into the display case, port
  what the port lacks, re-derive + un-ignore the two ignored 0c tests. Sandbox kept repo/ISO/
  work/target from the previous session; branch pushed as checkpoint.

- (18:25 UTC, 10-26) level-0c-display-hatch-2 SESSION: the route settled — the bay drop from the
  walkway side is the game's way in (the ascent + mid-floor legs are [L]-dead). The player toucher
  ported to the walk capsule's SPINE with no radius inflation [M 0x194df0's A/B; L-forced by the
  rest-state ch 27 = 0] — the bay drop now pulses ch 27 exactly like the game; nf-game green (the
  gatehouse test's -0.4 root hack fixed for the spine model); the bay-drop test re-derived from
  the [L] and UN-IGNORED GREEN (3 passed / 1 ignored in fix_0c_hatch_live). The roof walk stays
  ignored: the port's SS #179 rests risen (matrix y 20.0) and a dynamic poly at z ~ -110.66
  freezes the mouth — the game walks it unburned [L 2]. Report + live checks filed; suite on the
  final tree running.
- (18:20 UTC, 10-26) Toucher settled at the walk CAPSULE with r 0.55 (the r-0.0 variant broke the
  master-green chain/gatehouse touches — measured, reverted); the bay-drop test pins the pulse +
  the floor pin, the rest-state ch 27 divergence documented (the volume-extent live check).
  fix_0c_hatch_live 3 passed / 1 ignored. Suite restarted on the final tree.
- (19:20 UTC, 10-26) level-0c-display-hatch-3 SESSION START: branch from origin/master + merge of
  origin/s/level-0c-display-hatch-2 (the toucher + bay-drop test). Task: the three fresh [L
  2026-10-26] answers (the bay floor's SOUTH side is a wall — no pocket drop from z -111.4; the
  mid floor's real root 12.39 — the port's 12.36 is right, no 0.93 gap; SS #179's rest matrix y
  still unread — the runtime address unknown) say the case is NOT entered from the bay's south
  edge nor the mid floor. Find the remaining route into the 6.27 pocket / jetpack #414 at
  (5.50, 7.55, -110.37) from the level's own data [D placements, triggers, scripts, SS #179's
  keyed motion] + the game's rules [M], port what the port lacks, un-ignore
  from_the_duct_top_the_open_hole_carries_bond_down or replace it with the real route's test.
  Suite started in background.
- (20:30 UTC, 10-26) level-0c-display-hatch-3: the route + the glass reject landed. [M] Collide_Filter
  0x1ed888:7-9 rejects surf & 0x40 (glass) under the 0x800 collide-flags word Player_Collision
  0x194df0 writes in every substate (:205/:229/:257/:282); Intersect_CylGeom 0x109da0:533-540
  consults it — the port's capsule_tri_contact now skips mat & 0x40 (the roof walk's z −110.66
  blocker was the STATIC skylight-lantern glass room_sk1 [D mat-0x40 quads y 17.21..24.47], not
  SS #179). [D] #179's 0x19 record 5: key0 = the placement (5.7526, 16.2985, −112.2475) = rest
  (the vent glass DOWN over the opening y 15.03..17.42), key1 = 20.0 = risen — the round-2
  "rests risen" note read the probe's post-burn dump; rest pin added to the_hatch_sweep. The
  route: the duct roof walk's [L]-proven end is the BALCONY (15.6536 = 14.61 + 1.04); the bay's
  south end is a solid wall (mat 0x06, z −112.34, y 11.62..13.57) + ventblocker_01/02 glass
  (mat 0x40, z −112.36) — the case is the OPEN GROUND FLOOR (room_storage01 6.15/6.27 floor east
  to x 11.19 [D], guards #397/#400 patrol it [D]), the jetpack display spot = Camera_Subject
  #414 (5.50, 7.55, −110.37) [D]. The ignored test REPLACED by two green tests:
  from_the_duct_top_the_roof_walk_descends_to_the_balcony ([L] waypoints + ch 27 stays 0) and
  the_ground_floor_walk_reaches_the_jetpack_display (west walk across the case at root 7.31).
  fix_0c_hatch_live 5/0; nf-game --lib 578/0; the 0c walk/chain/interactable batteries green.
