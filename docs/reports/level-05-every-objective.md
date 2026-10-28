# level-05-every-objective — report

Task: make `crates/nf-view/tests/walkreal_07000005_live.rs` walk the whole mission from a
fresh load on real input, every objective the level's mission data lists asserted complete,
ending on the level exit firing. Branch `s/level-05-every-objective` (from origin/master
3c312512).

## What the mission data actually lists [D]

Dumped the MissionData table straight from the disc's ACTION.ELF `.data` (table 0x2a4350,
24 × 40-byte records; probe sources committed under `crates/nf-view/examples/`):

- Mission 0x07000005 = **6 objective records over 4 parts**: part 0 = 07000005 (1 record),
  part 1 = 07000006 (1), part 2 = 07000007 (2), part 3 = 07000008 (2).
- Level 07000005's own record is the single "Breach the castle walls" (text 0x04000000,
  cond ch 0x46, enable ch 0x45, flags 0) at 0x2a3e28 — the port's
  `mission/c07000005.rs` parse is exact against the disc bytes.

## Which objectives the current tests already cover

- **Objective 0 ("Breach the castle walls") — COVERED end to end** by
  `walkreal_07000005_live.rs::walk_to_the_gatehouse_and_ride_the_truck`: fresh cold-start
  load, real-input walk to the gatehouse, the scripted truck ride, the ch-0x70 park, the
  level exit `Some(0x07000006)`; then the warm 07000006 hand-over asserts ch 0x51 →
  OR #498 → ch 0x46 → `states[0][0] == 4` (complete) and the part-1 record revealed
  (`states[1][0] == 3`). The game's own order (complete at the next part's arrival plate,
  because ch 0x46 has no writer on 07000005 [D the full placement census]) is what the test
  pins.
- **Objectives 1–5 are covered by the sister part files**, each walked on real input:
  `walkreal_07000006_live.rs` (the castle interior walk to the upper gallery),
  `walkreal_07000007_live.rs::walk_the_party_start_to_finish` (exit #418 → 07000008,
  objective 2 completes on the next level's plate), `walkreal_07000008_live.rs` (the
  gondola/briefcase walks and the mission-end LevelToEndTo).
- **NOT covered anywhere: the 07000005 cable route's final legs.** The file's own header
  (route 2) promises "E on the zip door #112 (fallback 0x1E) loads 07000006", but the cable
  tests stopped at the notch ride — no test ever walked from the wire landing toward the
  zip door, and no cable-route test asserted the exit.

## What this task added

1. **`the_cable_landing_walks_the_south_walkway_toward_the_zip_door`** (new, green): the
   whole cable route from the fresh cold start (the slab band, the gap, the west pieces,
   the seam, the kink ramp, the notch grab, the strafe ride — the round-8 legs verbatim),
   then the NEW legs: the ride's landing asserts, then a real-input south walk. The walk
   **pins the blocker** (a [D]-backed negative test, no ignore): Bond stalls at the
   landing's south rail (best z −0.45, never crossing −10).

2. **The blocker's disc evidence** (probes committed: `probe_south05.rs`, `probe_nav05.rs`,
   `probe_wire251_05.rs`):
   - Wire #251 "Wire_Cable" is only the 8-unit hand-over-hand cable (world span z 2.22..10.30
     at y 34.47 [D its object bounds × placement matrix]) — there is no zipline object on
     the level.
   - The floor map over x −26..50, z 4..−76 (down-ray top hits below y 55): the landing's
     south piece (x −26..−22, z 0..−8, floors 30.4-31.0), a west walkway south (z −16..−36,
     floors 32.3→39.8) — then **z −40..−64 is void at every x** (no floor, no roof).
   - The zip door #112 "Locked_Door_Top_I" (37.25, 49.00, −69.00) [D placement; params
     p3=0x1E, p0=0x07000006] stands on a y-49 band (x 16..36, z −68) reached along a
     battlement chain the game's own nav graph carries as an ISOLATED 7-node AI path
     (#149-155, (13.7, 37.4, −72.2) → (43.6, 44.6, −73.5), degree-1 dead ends at both ends),
     patrolled by the door's guards #271/#272 [D placements]. No geometry connects it to the
     wire landing's side in the disc's data.
   - So the cable route's exit cannot be walked with the collision the disc carries; the
     game's real route must use something else (a scripted carry, a climb volume the port
     does not model, or a 07000006-side approach to the same coordinates).

## How it was verified

- `cargo test --release -p nf-view --test walkreal_07000005_live -- --test-threads=1`:
  **9 passed; 0 failed; 0 ignored** (69.87 s) — the file is green with the new leg.
- nf-game/nf-formats/nf-audio are untouched by this branch (test-file + probe-example
  changes only).
- The full suite (`tools/test/suite.py`) was started in the background per the brief; the
  sandbox's gateway keeps killing the background process (three launches, see WORKLOG), and
  the 879-binary run does not fit the remaining window on 2 cores. At push time the runner's
  output is at `suite.out` in the repo root (untracked scratch); its FAILED/TIMEOUT lines
  were empty before the last relaunch.

## What's left (the honest blockers, each with evidence)

1. **A single continuous whole-mission walk (05→06→07→08) is blocked at the 06 open leg**:
   `walkreal_07000006_live.rs`'s documented "OPEN LEG" — the upper gallery → dressing-room
   (LoadPoint #746) connection has no walked route in the port (the gallery's north lip and
   the east corridor are walled [D its probes]; the game's own nav has ai point 5 patrolling
   a 9-node path above the tunnel). The LoadPoint chain itself (ch 0x83 → ch 0x02 →
   level_exit 07000007) is pinned green through the real trigger by `walk_07000006_live.rs`,
   but its entry is not walked.
2. **The 05 cable route → zip door #112** — this task's pinned blocker above.
3. Everything else the task names (the objective list, the per-objective completion asserts,
   the exit) is covered and green as itemised above.

## Live checks for the main agent

1. **The zip-door route question** (the pinned blocker): fresh boot of 07000005, poke Bond
   to the wire landing (−23.5, 33.5, 3.1) with `fieldprobe.py`, then walk south by input and
   screenshot at z −10, −20, −36: does the real game let a player reach the south walkway and
   beyond (i.e. is the disc's void band real, or does a scripted carry/climb exist that the
   port misses)? Watch `switch_channels` for the fallback ch 0x1E write if the door is
   reached.
2. **The gatehouse plate**: `gdbwatch.py` on the game's switch_channels+0x5C during the
   walked gatehouse approach — the walkreal walk logs ch 0x5C but the toucher stand-in's
   reach over the plate (98.49, 27.05, 106.36) was never proven against the real game's
   toucher behaviour (the ch-0x45 stand-in miss is documented; ch 0x45 itself is set at boot
   [L ee_start_05]).
3. **The zip door E**: on a fresh boot, E beside door #112 — expected: fallback ch 0x1E
   writes, level 07000006 loads [M Trigger_Activate 0x1b2318:91-107], the 0x1E start
   #510/#511 is picked [D level_carry_live's census].

## Usage

not visible to me
