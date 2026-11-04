# trigger-plane-records — the report

Task: land the game's trigger contact — parse the glist's OWN plane records (each tri's 4th u16 →
the leaf s16 area), feed `seg_touches_tri_raw` the record's plane, re-wire `touched_by` (r > 0) and
the standing `player.body` (the e7f80cd6 diff), retune the 0a ring's #614 jump press. Branch
`s/trigger-plane-records`, continued from origin/s/trigger-hittest-segment (6de4f385) + master.

## What landed

1. **The record format, pinned on disc data [D]** (the probe `crates/nf-formats/examples/
   probe_glist_planes.rs`, kept): the plane record sits at the leaf's s16 area base + plane·2
   BYTES and is 10 bytes — the normal 3×i16 · 2⁻¹⁴ (used RAW, |n| = 16383-16384, never
   re-normalised) and the distance as the f32 whose bit pattern spans the record's halfwords +3/+4
   (byte +6..+0xA). Records are 5 halfwords apart; the tri's 4th u16 is a HALFWORD index into the
   same pool the vertex halfwords use. Consistency measured through each record's own tri: 0a's
   laserbeam #621 tri 0 record n (−45,16383,0) with d-bits 0xBA55912C = −8.34e-4 sits the plane
   through the tri (residual 4.4e-4, the i16 quantisation); 07000011 #235's records (n ±16383 x,
   d ≈ 0) and 07000005's spawn plate (n (0,16384,0), d = −0.0) sit exactly through theirs. The
   prior "4-i16 (n×1e-4, d) quad" reading was imprecise: d is an f32, not a scaled i16 — the
   geometric consistency of all three target glists under (n·2⁻¹⁴, f32 d) is the [D] proof.
2. **The parse**: `Collision::triangles_with_planes()` (nf-formats/src/collision.rs) emits per tri
   (verts, material, record normal, record distance) with the same winding-alignment swap
   `triangles_with_material` always did; a missing/degenerate record (|n|² ≤ 1e-9, or the halfword
   pair out of bounds) is marked NaN. The trigger parse (nf-game/src/triggers.rs) transforms each
   record with the placement matrix — n_w = R·n_l (linear part only), d_w = d_l − n_w·t — the
   exact rigid transform of the game's local-plane test, so the one-sided entry's SIGN is the
   record's own, not the winding's. `Trigger` gained `planes: Vec<([f32; 3], f32)>` (both struct
   literals updated); `set_tris` fills winding planes as the fallback (hand-built volumes, tests).
3. **The raw law re-landed**: `seg_touches_tri_raw(a, b, r, t, n, d)` restored from e7f80cd6 with
   the plane as a parameter (the plane-crossing/centroid anchor, the directional skip, the
   one-sided code-99 entry, the strict < radius refine — the [M] asm chain cited at the site);
   `touched_by` (r > 0) runs it per tri with the record planes. The standing toucher switched to
   `player.body` (the HITTEST segment, the e7f80cd6 half; built case-correct at the scene's
   drone::Player site). `low_beam_621_fires_standing` UN-IGNORED; the 0a ring's #614 presses
   retuned 1.0 → 1.8 (both crossings).
4. **The record planes ride the key motion** (found while measuring — see the ring below): the
   laserbeam glists are key-motion triggers (the beams SWEEP across their corridors). `Motion`
   gained `local_planes` (the object-space records) and `step_motion` re-poses them with the
   step's matrix every tick, exactly like the triangles. Without this the beams' planes went
   stale at the parse-time pose after the first motion step.

## What was measured (each binary whole, --test-threads=1, release, NF_ISO set)

- `nf-game` lib: **656 passed / 0 failed / 5 ignored** (the hand-built `plate()` test helper was
  re-wound +y — the disc's floor-plate records all point up [D 05's plate n (0,16384,0)], and the
  raw law is one-sided on that side where the old both-sided capsule was winding-blind).
- `walkreal_0700000a_live`: **9 passed / 4 failed / 2 ignored — `low_beam_621_fires_standing`
  UN-IGNORED and GREEN.** The 4 failures = the office-ring walk + its 3 real-damage sisters.
- `fix_truck_live`: **0 passed / 2 failed** — the 05 spawn-reveal plate ch 0x45 is STILL silent
  under the record-plane law (see the open lever below).
- `interactables_07000011_live`: **15 passed / 1 failed** (the failure list was not re-dumped
  before the window closed; the binary's one red needs a look next session).
- The full suite did not fit the window (the release build of the four crates consumed it; the
  run was cut at the 10-minute tool cap still in the compile phase). The binaries above are the
  ones the touched code can affect; every `walkreal_*_live` beyond 0a was NOT run — next session
  must run them all before calling the wiring landed.

## The ring mechanism — pinned this session, not landed

The office-ring failures are NOT a sign or an arc-height problem: **the beams sweep**. The port's
#614 trigger's live bounds were caught at z 10.73..11.01 while its static (key-0) pose — and the
[L]-measured game line — sit at z 12.90..13.17: the glist rides key motion across the corridor
(the laser slides side to side). The standing HITTEST segment reaches 0.45 under the root (the
box reached 0.05), so the jumper's DESCENT through the beam's swept pose touches it (measured:
the 0x22 writes fired for 6 straight frames while the descending walker passed z 10.75 with the
beam parked there; the closest-point distance was 0.53 < 0.55). A player times the crossing
against the sweep — the sweep-aware press windows were implemented and measured (the history
keeps them: southbound arms when the whole live box sits ≥ 1.7 past the line on the far side;
northbound when it still straddles the line just starting its far-side slide; Bond HOLDS at the
lane's edge otherwise), but each retune re-cascades the route's later leg timings (the walk-back
nav, the band legs), and the window closed at 9/4 with the blind 1.8 presses — the honest state
pushed. The lever for the next session: re-time the ring's beam crossings against the sweep
(the watch code is in the history at "the sweep-watch route experiments" commit), or pin the
beams' key paths [D] and place the crossings in the measured gaps.

## The 05 spawn plate — open

The plate's record plane is exactly through its own tris [D probe] and transforms to the world
plane y = 24.41 (normal up). The crouched ride's anchor (B = pos − 0.1, cc 0.55, the park 0.607
over the plate) sits 0.507 above the plane — front side, distance 0.507 < 0.55 — the law SHOULD
fire, and the boot-tick standing case likewise (the anchor 0.557 below the hip is above the
plane). It does not fire: the remaining lever is the boot tick's actual toucher state (the
spawn-drop cc, which HITTEST arm the game runs on the boot tick) or a lock/mask state on the
plate row — needs the live trace or a focused boot-tick probe before the wiring can claim the
truck chain.

## Live checks for the main agent

1. **The 05 spawn plate, boot tick**: with a debugger watch on the plate object's obj+0xD0
   contact list at the boot tick — does the standing/crouched HITTEST segment touch the glist at
   all, and what does `Dist2Tri` return for the plate's quad (the signed plane distance and the
   feature code)? The port's law passes the one-sided entry at +0.507 and then needs dist < 0.55.
2. **The 0a beams' sweep**: confirm the laserbeam glists' key motion in the game (the beam
   object's obj+0x90 matrix over time — does #614's beam slide z 13.17 → 10.73 and snap back?).
   If yes, the ring route must be timed against it; a PCSX2 crossing at the same walker ticks
   would settle whether the game's own ring run ever stands in a swept beam's reach.
3. **interactables_07000011_live's one red**: which test failed (the binary was not re-run with
   the list before the window closed).

## Usage

not visible to me (no token/credit/context-window figures are exposed in this environment; the
session's boundary was the ~07:15 UTC sandbox expiry with the 06:55 stop-new-work line — the
finish ran from 06:33 UTC).
