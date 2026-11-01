# level-0b-lift-exit — the report

Branch `s/level-0b-lift-exit` from origin/master (e79794da). Status: **blocked on the route
identity** — the suspect in the task was refuted at the data level, the real blocker is
identified and pinned, and the teleport stand-in remains because the game's own route up is
not recoverable from the dumps I can reach. Everything below is measured on this branch's tree.

## What the task suspected, and what the disc says

The suspect: "the walk rides the lift's ROOF while the game carries the player INSIDE the car."

- The cab (script entity 0x2000367, obj 'lift', 1435 collision tris [D the map 0x2E block]) is a
  tiered box: base slabs at local −2.89..−1.24, **the platform floor at local −0.15** (140 tris,
  x −2.59..1.48, z 23.60..27.24), a 4-tri south ledge at +1.07, the rail band at +2.26..+2.61,
  **the roof slab at +3.76** and a centre cap at +4.03..+4.10. The keyed y rides matrix[3][1].
- The port's walk rides the keyed+4.1 surface (the roof slab + its cap). That is also where the
  game's own feet ray lands: `Player_FeetOnPoint` 0x194188 casts down from HITTEST+0x30 and the
  first hit from above on the posed entity is the +3.76 slab. Riding "on the roof" IS the
  game's ride law; the player is never inside a walkable interior.
- The interior IS there (the −0.15 floor tier) but it is **sealed**: the parented glass
  0x2000421 (obj 'lift_glass', 56 tris, local y −1.26..+2.46 — body heights) closes every face,
  and the game gives it no walk-through exception. The collide node's kind-exclusion word
  cel+0x98 — the only mechanism that could open it to the player
  (`Collide_Intersect` 0x1ebaa8:301-317: node bit 0x1 excludes hittest kinds 0xd/0xe, bit 0x2
  excludes 0x10, bit 0x800 excludes 0x40) — is written ONLY for room cels
  (`parseentity_fixup` 0x1d00c0:115/142, cel kinds 0xc023/0xc047); the static object parser
  writes ONLY cel+0x94 (`parsemap_block_map_data_static` 0x1d1998:101-103) and the dynamic/
  script-entity create paths write no +0x98 at all. A script entity's node carries +0x98 = 0.
- Measured: at the exec pass (cab y 46.93) all 36 face lines (9 z-lines × east/west, 9 x-lines ×
  south/north, at floor+0.5/+0.9/+1.5) hit; 0 openings. Interior rays (cy+1.6 → cy−0.5 over the
  middle) hit nothing but the −0.15 floor's west/north rim bits. The cage has no door and no
  walkable interior **in the game's own collision model**.

## Why the teleport cannot be deleted yet — every walked exit is refused

From the ride surface (the roof slab), as the cab passes the exec floor:

- **East** (the old route): the waiting-room floor hole — x 2..6, z 24.5..27.5 has NO floor down
  to y 39 [D the room ray grid]; the old walk fell through it and the ~6 m teleport rescued.
  The pit at 45.4 (rim ≈ 1.4 m over the 45.4 floor vs the jump 0.77) is that hole's bottom bowl.
- **South**: the shaft's south wall — a solid band (z ≈ 22.7..23.2) capped at 55.5, hitting
  every southward body-height ray at lid+0.3..+2.1 across x −1.2..1.45. The exec floor's strip
  at z 22.0..22.5 (y 46.8) sits behind it with no doorway the rays can find. I built and ran two
  south-step-off rewrites (level-0b-lift-exit commits 39457aa4's route work): the walker wedges
  on that wall exactly as the ray grid predicts. Not committed as route changes.
- **West/North**: wall/void (the west chasm over the balcony strip, the north face solid).
- The AI nav agrees: `nf-extract route` from the spawn-side balcony (2.3, 14.0, 25.8) to nav n41
  (12.21, 46.9, 19.68) → "no route: code 5". The stairwell doors (#143 at 52.38, #154 at 46.76)
  connect exec↔roof, not balcony↔exec. The docs' own note stands:
  docs/levels/0700000b.md:65 "[S — the route up is not pinned by the dumps]".

## What this session delivered

1. **New pin** `walkreal_0700000b_live::the_paternoster_cab_is_a_sealed_cage` — the face-ray
   grid (every line hits) + the interior emptiness, with the [M] citations (0x194188,
   0x1ebaa8:301-317, 0x1d1998:101-103, 0x1d00c0:115/142) inline in the test doc comment. It
   fails on the wrong result: any change that opens the cage without disc evidence red-flags it.
2. **docs/systems/lifts.md §0b**: the "ride gap (STALE)" row replaced by the resolved
   not-a-route row (the sealed cage, the exit refusals, the nav no-route), and **§7 item 5**:
   three live checks for the main agent (below).
3. The walkreal's own header comment and the ride section keep the documented stand-in; the
   green master route is restored (my two south-route attempts are documented in the WORKLOG,
   not left in the tree).
4. The two `fix_0c_pylon_live` FAILED lines and the two exit −9 CRASH lines in the suite output
   are the environment's (the GPU-less pylon probe; the sandbox OOM-kill on the net/mp stress
   binaries — exit −9, not test failures), matching the previous sessions' suite notes.

## Totals (all run on this branch's final tree)

- `walkreal_0700000b_live`: **4 passed, 0 failed** (the three walks + the new sealed-cage pin)
- `walk_0700000b_live`: **4 passed, 0 failed**
- `interactables_0700000b_live`: **9 passed, 0 failed**
- `models_sweep_0700000b_live`: **2 passed, 0 failed**
- `objectives_0b_start_live`: **3 passed, 0 failed**
- `nf-game`: **98 binaries — 959 passed, 0 failed, 9 ignored**
- Fast suite (`tools/test/suite.py`, NF_ISO set): still running at finish time (1013 binaries;
  ~78 completed when the finish steps began, 0 code failures — the 2 FAILED pylon lines are the
  GPU-less environment's and the 2 CRASH lines are sandbox OOM kills, exit −9). The per-binary
  log is `/home/z/suite.log`; the totals line did not exist yet when this report was written.

## The strongest new lead: the arrival cut-scene's own event 6

The arrival cut-scene (SP #246, script 0x060004F9, [D pack 0780000b]) — the one the walkreal
waits out before the shaft walk — **ends with an event 6 (the re-place) at t 689 of 690**
(stream 11: `Event { kind: 6, params: [0] }` at Marker(2b1) [D the pack's NIS dump]). The game's
event-6 law [M `Script_EventHandler` 0x1d94c0:125-136]: copy `SCRIPTINFO+0x40` (the script's own
matrix) into `glb_players+0x90` (the player's matrix) when the firing stream record's kind byte
(+0x24) is not 1/2 — an Event stream's isn't, and the arrival's event-6 stream (stream 11,
Wait/Marker/Event only) has no entity. So the game re-places Bond at the script's own matrix at
the cut-scene's end — the same law the roof cut-scene uses to land him on the helipad. The port's
event-6 (nis.rs, proven by the roof-scene and level-14 tests) snapshots the previous stream's
object (kinds 1|2) and otherwise leaves Bond where he is — for the arrival, stream 10 is a
Light, so the port keeps him on the balcony. **Whether the disc's SCRIPTINFO+0x40 at t 689 holds
the exec-floor landing** (making the whole balcony→exec transit the cut-scene itself — no walk,
no cab, no teleport) is the next session's first question: it needs the disassembly of
0x1d94c0's +0xab3 stream-record rule and the writer of SCRIPTINFO+0x40 (the script root keys —
the arrival's last keys parse as (0,0,0) scale 0.02 at t 690 [D], a world-origin pose, so the
raw key read alone is not yet conclusive). The script's camera path (camera true, 283 camera
keys spanning t 0..690 [D]) IS the ride presentation; the cut-scene is 690 ticks ≈ 11.5 s.

## What's left (the next session's work)

1. The arrival event-6 question above (the cheapest decisive step; before the live checks).
2. The live checks (lifts.md §7 item 5) — only the main agent's PCSX2 can settle the route:
   (a) the shaft mouth at the balcony during the bottom hold — what the ground object under
   Bond is (`gdbwatch` Player_FeetOnPoint 0x194188's return at 0x1943a8); (b) whether a full
   ride attempt ever raises ch 0x53 (mid-shaft plate #171's out channel — a real ride fires it);
   (c) a fieldprobe of the shaft's south wall band at (0.5..1.5, 46.8..48.5, 22.4..23.2) during
   the exec pass — is there an opening the port's parse reads as solid?
2. If the live checks reveal the game's mechanism (a kind-masked sub-part, an anim that opens a
   cage face, or a different boarding spot), implement it with the address cited, re-point the
   sealed-cage pin if the disc evidence contradicts it (the pin documents its own refutation
   path), delete the teleport, and walk the exec-floor lane (the walked waypoints I measured:
   the landing zone x 0.5..2 z 21.5..22.5 → south along x ~2 → the z 17.0 seam east → north-east
   to n41 — the floor grid in this session's probes shows it continuous at 46.8).
3. Run the full fast suite to its totals line on the final tree.

## Usage

not visible to me — this session's environment exposes no token, credit or context-window
figures anywhere I can read (no usage panel, no token counters in any tool output or log).
