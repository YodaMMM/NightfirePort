# level-06-every-objective — the whole castle walked to the party

Branch `s/level-06-every-objective` (from origin/master ccc714a1). Task: make
`crates/nf-view/tests/walkreal_07000006_live.rs` walk the WHOLE mission of level 07000006 from a
fresh load on real input (`step_bond` movement/aim/fire/use only; no teleports past a blocker, no
channel or state pokes), every objective the mission data lists asserted complete, ending on the
level exit firing.

## The objective inventory (what the level's mission data lists)

07000006 lists exactly **one** objective [L ee_start_05.bin @ 0x2a3e40 — 1 record, re-dumped
against the port's `c07000006.rs`; docs/systems/objectives-all-levels.md]:

| text | says | completes on | revealed by | asserted where |
|---|---|---|---|---|
| 0x04000001 | "Find a way inside to the party" | ch 0x02 — the dressing cut-scene's closing event 0xD [2,1] | ch 0x46 — the arrival plates' OR #498 | `walk_the_castle_interior_to_the_party` (reveal at the courtyard, fold after the dressing scene) |

## What the existing tests already covered (and their stand-ins)

Before this task `walkreal_07000006_live.rs` had 2 tests covering the **reveal half only**: the
truck-ride spawn → the dismount → the well lane → E-doors #47/#59 → the searchlight courtyard →
E-door #71 → the tunnel switchback → E-door #72 → the upper gallery, asserting ch 0x46 on
(objective revealed), state ≠ 4, and `level_exit == None`. The completion half (the LoadPoint →
ch 0x83 → the scene → ch 0x02 → the exit) was only pinned by the headless teleport test
`walk_07000006_live.rs`, whose entry pokes `sim.bond.pos` — the walkreal header called the gallery
→ dressing-room leg "THE OPEN LEG" and said the port's collision gives no walked connection.

Stand-ins the walkreal still carries (unchanged by this task, all load-time inputs, no pokes):
1. the fabricated 05-exit RamSave `carry(&[0x1d, 0x45, 0x70])` + `Inventory::level_start(0x73,
   240)` [S unverified stock — the real 05-exit save's inventory is not sampled; the carried gate
   ch 0x1D itself is [M Player_Start 0x190fd0]];
2. the harness's `top_up_health` and `skip_cutscenes` (the game's own skip action [M SP_Update
   0x1dbda8:129 → Script_FFwd 0x1dac78]; ffwd's catch-up still fires every event).

## The open leg, closed: the game routes the room AROUND the gallery

The old header guessed "a stair volume the port's floors miss, a scripted carry, or the AI patrol
graph". The answer is the third, and the port has **no collision gap at all**:

- **The gallery's north edge is a balustrade.** The deck ends at z −64.8 and balusters stand at
  z ≈ −64.92 spanning y 49.25..51.3 from x 30 to 36.7 [D this tree's fine horizontal-ray sweeps in
  `probe_06_gapmap.rs`: hits at every sampled y, gaps < 0.7 wide]. Bond stops dead against it
  grounded AND airborne (`probe_06_edgetest.rs`: walking north from (35, −65.6) grinds at
  z −65.50 with and without jump pulses). The older probes' "lip at x 20–30" was this same rail.
- **Door #46 Locked_Door_Top_I is not the route**: ch_lock 200 and nothing on the level writes 200
  [D the door dump: state 2, ch_lock 200; M Door_Activate 0x135be0's is_locked 0x136810:10-16;
  U-044 stays open for 05/0d/15 but no 06 writer exists in the census].
- **The game's own walkable web settles it** [D map block 0x05, "Boundary Prime", dumped with
  `probe_06_aipaths.rs`; M AIPath_Parse 0x1d2598]: the web chains the courtyard's 41.15 floor
  (nodes 355 (25.23, 41.15, −47.27) / 371 (24.12, 41.15, −50.0)) → the 43.0 step at the courtyard's
  south wall (370 (19.96, 43.27, −49.95)) → the ledge's north end (363 (17.49, 43.49, −50.34)) →
  south along the x≈17.4 ledge rising 43.5 → 45.3 (364 (17.44, 45.36, −53.74) → 365
  (17.37, 45.26, −64.64)) → east along the 45.25 shelf at z −64.6 (366 (35.49, 45.25, −64.60)) →
  north through the 46.0 sill's open gap at x ≈ 36 (367 (35.49, 45.25, −62.28)) into the room's
  43.1 east strip (377/378) and the LoadPoint. Every floor under that chain exists in the port's
  collision [D `probe_06_gapmap.rs` band maps: the ledge 44.6..45.9 at x 15..17.5, the shelf
  45.0..45.3 over z −64.5..−62, the sill 46.0 with 45.0 holes at x ≈ 32/36, the room 41.0
  (x 21..28) / 43.0 (x 29..37)].

## What changed

`walkreal_07000006_live.rs` (test-only; the three probe examples are new scratch evidence):
- `walk_to_the_gallery` split into `walk_to_the_courtyard` (spawn → the tunnel-door approach) and
  `walk_the_tunnel_and_gallery` (door #71 → the switchback → door #72 → the gallery) — the old
  coverage unchanged, now with the balustrade/locked-#46 findings pinned in its doc comment.
- NEW `walk_the_ledge_to_the_party`: the game's own route on real input — the courtyard's east
  strip → the 43.0 step → the x≈17.4 ledge → the 45.25 shelf → the sill gap → the dressing room →
  the LoadPoint #746 walk-in → poll ch 0x83 → the dressing cut-scene (the game's own skip) → poll
  the objective fold to 4 → assert `level_exit == Some(0x07000007)`.
- The full-mission test `walk_the_castle_interior_to_the_party`: spawn assert → the reveal asserts
  at the courtyard (ch 0x46 on, state 1 or 3 [M 0x182270:98-105 / 0x1827d8:55-67], no exit) → the
  ledge route → the fold/exit asserts. The gallery leg keeps its own test
  (`walk_the_castle_interior_to_the_upper_gallery`). The sister
  (`stopping_before_the_tunnel_never_ends_the_castle`) now halts at the courtyard and still asserts
  no ch 0x83 / state ≠ 4 / no exit.
- The header carries the objective inventory table, the stand-in list, and the OPEN-LEG note
  (the balustrade, the locked door, the web chain, and why the gallery is scenic).

No assertions were weakened; no decompile-backed test changed meaning. No port code needed a fix —
the collision was already correct; the walk was missing the game's route.

## How it was verified

- Direct binary runs on this tree (`NF_SILENT=1 ... --test-threads=1`):
  - `walk_the_castle_interior_to_the_party` — **1 passed, 0 failed, 21.19s** (the whole mission:
    the walk log ends `north across the dressing room: level_exit reached at +78 ticks`; falls
    empty).
  - `walk_the_castle_interior_to_the_upper_gallery` + `stopping_before_the_tunnel_never_ends_the_castle`
    — **2 passed, 0 failed, 35.82s** (two recoverable wedge events in the courtyard nav, both
    self-recovered; the gallery reached at tick 9959 as before).
- FULL suite (tools/test/suite.py, background): TOTAL LINE QUOTED AT THE END OF THIS REPORT (the
  suite runs the tree this branch was cut from plus the WORKLOG commit; the only binaries this
  task's changes touch are `walkreal_07000006_live` — green above — and the three probe examples,
  which are not suite binaries). Environmental failures expected on this sandbox: the 3
  `disc_golden` tests (slim ISO), `fix_0c_pylon_live` (no GPU), and OOM crashes
  (`net_soak_faults_live` exit -9 at 399 binaries in).

## Left

- Nothing on the task: the level's single objective is covered end to end on real input and the
  exit fires. Optional next-session polish: exercise the `cut: true` fusebox Laser route through
  the same walk (the stealth variant is already coded in `walk_to_the_courtyard`), and the
  tux-skin draw gap at the dressing scene ([M Player_Init 0x1910f8:96-107], pre-existing, visual).

## Live checks for the main agent

None required for the asserts (everything is Sim-side and proven by the walk). Optional render
confirmation of the new leg's look (lesson 6), all on the final tree:
1. `--bond 17.4,46.5,-56.5,1.57,0` — Bond on the x≈17.4 ledge mid-route; expect the ledge walkway
   with the tunnel shaft's east wall to the west and the dressing room's roofline east.
2. `--bond 35.4,46.3,-63.5,1.57,0` — on the 45.25 shelf under the gallery; expect the gallery
   deck's underside/balustrade above and the sill wall to the north.
3. `--bond 35.5,44.3,-54.1,0,0` — in the dressing room at the LoadPoint; expect the room interior
   (the cut-scene fires here).

## Usage

not visible to me
