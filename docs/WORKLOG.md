
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

## level-4a-ride-cutscene

- (17:22 UTC, 10-26) Branch created from origin/master. Task: 0700004a ride cut-scene (SP #289,
  pack 0x060006cf, ticks ~1611-2186) is missing objects — the satchel on the guard's hand and
  the lift floor under Bond. Decode the pack's Entity/Anim/attach streams, pin the ops [M
  0x1d94c0 family, Script_GetObj 0x1db688], port the missing ops/lookups. Sandbox fresh: repo +
  work-slim + onfoot ISO re-downloaded; branch pushed as checkpoint.
- (17:20-19:15 UTC) Probe session: the pack 0x060006cf decoded from the disc (nf-extract + a
  python levelbin walk); the ride-scene objects pinned live in a new nf-view test
  (ride_cutscene_objects_live): the sim carries the satchel entity 0x02000758 (s7, keyed local,
  base-composed — first seen (3.43,−52.05,−4.21), still moving at t300), the lift floor 0x020006ee
  (s13), NIS_P99 0x020006b1 (s12), NIS_LIFTHATCH 0x020006e1 (s10); the s3 guard anim (0x0500004)
  carries the event-0xB weapon byte 0x18 [M 0x1d94c0 case 0xb]; all three characters pose
  (pack sequences 0x040005fd/fe/ff + 0x040006e6/7/8 loaded). FOUND GAP (not yet fixed — 19:19
  sandbox line): the render's script-anim characters draw the pose root RAW (identity object
  matrix) but the pack's body-sequence roots are CAR-LOCAL [D 0x040005fd ch0 f1 (2.35, 0.85,
  −0.64), the same space as the satchel's local keys] and SP #289's base is the car (0,−53.53,
  −3.38) [D placement; #273 plays the same pack at base (0,0,0)] — the cast (and the satchel on
  the guard's hand datum) draws ~53 units off, at the map top. Fix recipe in the report: carry
  the SP base on Obj::Anim at Script_AnimStart and compose it in frame_state_for's script arm.
  Two SPs (#273/#289) play the pack — the entity draw slots are per-copy k (2 each), the anim
  characters are per-skin singletons following the newest match.
