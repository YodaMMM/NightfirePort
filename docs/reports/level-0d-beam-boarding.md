# level-0d-beam-boarding — report

Task: settle the last stand-in in `walkreal_0700000d_live::walk_arrival_to_the_c5_door` — the
beam boarding + the west ride on SP #115's shuttle (137.54 <-> 177.22 over 3000 ticks [D]) and
the drop into the yard — settle the access from the data where possible, walk the boarding and
the ride on real input, delete the stand-in; if only a live run can settle the access, pin the
exact question with a sister assert. Branch `s/level-0d-beam-boarding` from origin/master
(21bfe192, batch-1031h — master has not moved since; the branch is master + test-file changes
only, no source changes).

## Result: the ride and the drop are now REAL INPUT. The stand-in is narrowed to ONE
## documented write — the boarding placement. The boarding itself is pinned as the exact
## question with sister asserts; the data closes every route the port's statics offer.

## 1. What the disc data pins about the mover (all measured this window)

- **The mover is a twin-girder gantry, not a simple beam.** Script 0x060009EF entity 1's shape
  0x0200_0a80 is 1643 tris, local bounds (±4.17, −2.98..+2.98, ±17.01) [D probe20_static_map
  section D]; at the rest origin (137.538, 11.334, −287.108) the girders run z −270.1..−304.1.
  The new `probe20_beam_deck` column pairs (up AND down rays, z −280) read: the WEST girder top
  ~13.42–13.47 spanning x 135.1..136.5, the SLOT x 136.5..138.6 (both ray directions empty —
  open down to the yard 4.0, matching the fine-map down-rays), the EAST girder x 138.6..140.1,
  void outside both. A surface at 15.99 rides over the east girder (a rail/cable), and a 16.8
  band runs above the east girder's full length.
- **The riding 0x0200_0a81 is NOT a car.** It is a 2-tri FLAT plane at +2.95 (y 14.28, x ±2.44,
  z ±16.80) [D shape bounds] — a 0xEA trigger volume riding the gantry (the earlier
  crane-transit report tied it to ch 0x67). Both entities share the origin matrix exactly
  [D the beam-watch trace: identical origins every sample].
- **The sweep profile** [D the beam-watch trace, both entities]: rest 137.54 → east extreme
  ~177.22 at t ≈ 1230 → west park ~137.59 at t ≈ 2760 of the 3000-tick SP → re-sweeps east
  (143.68 by ~t 3160). No dwell at either extreme — the keys ease in/out per leg.
- **The east-side boarding candidates are closed from the data.** The elevated chain east of
  the gantry's max reach: the mezzanine 16.8 (x 168..183), the 13.76 step at x 184 (the
  interior-scan column; the new test asserts 13.4..14.2 there), the 14.7 pad (x 185..194,
  asserted 14.4..15.0) — and the column at x 182.3 (between the gantry's max east face
  181.4 = 177.22 + 4.17 and the ledge) has NO floor in 10..16 [D, asserted]. The gap is ≥ 2.1 m
  with the pad/ledge ABOVE the girder tops, and the jump is VERTICAL-ONLY [M Player_HandleJump
  0x1916b0 zeroes the velocity before the impulse] — no 2.1 m gap is crossable onto a higher
  surface. From the bay floor (9.5) there is no stair/ladder up (the ladder census: #0/#1 at
  x 212..218, #2 at the catwalk's west end [D probe0d_route_scan]), and the gantry's own frame
  rises 9.5 m from the yard 4.0 with no access either. Sister asserts for all of these are in
  the test now.

## 2. What is real input now (the stand-in's replacement)

After the E on #460 (unchanged, real):

1. **The east leg plays out under Bond's feet** — the test samples the origin to its east
   extreme and asserts `> 176.5` [D the key trace].
2. **THE RIDE, real input** (after the one documented boarding write, below): the carry [M
   Player_FeetOnPoint 0x194188 → Control_InheritVelocity 0x133e88; the port's
   `CollisionWorld::ground_motion`] holds Bond at a constant offset on the west girder through
   the whole westbound leg — the test asserts the offset at boarding |off| < 0.8 and the drift
   at the west apex < 0.75 (measured 0.60 — the ease-out lag only; without the carry it would
   be ~13 m). The apex assert: origin ∈ 137.3..138.2 [D ~137.59].
3. **THE DROP, real input**: the girder edges WALL the walk (measured: the held walk input
   yields zero displacement across an edge — the capsule's bottom sphere meets the edge face
   below the top; both directions attempted). The game's way off is the JUMP [M
   Player_HandleJump 0x1916b0 — the vertical-only launch] plus the walk's own air drift [M
   Player_Move runs every walk tick]: the test jumps at the park, holds east ~20 ticks (the
   smoother's ~1.7 m drift centres Bond over the open slot), releases, and Bond falls ~9.5 m
   through the slot onto the yard 4.0. The jump is a PLAIN jump — the icon context re-arms
   from proximity every tick [M 0x1949a8:22 clears BL+0x95F; scene.rs's per-tick icon arm] and
   the wire icon #297 is ~75 m away, so no wire grab. The landing is asserted inside the
   probe-proven yard band (x 136.0..139.2, grounded, health > 40 [M 0x1949a8:117-126 — the
   9.1 m fall ≈ 24 damage]).
4. The yard legs (ST-3 plate → the photograph → the C5 kill → the mission end) are unchanged
   and still all real input.

**The one remaining pos write** (documented in the test as [S] unverified): the boarding
placement itself — Bond is placed over the west girder at the westbound leg's start
(`origin − 2.54, y 15.5, z −280`), the exact placement `probe_the_ride_carry` rode. Everything
after that write is the game's own rules.

## 3. Verification

- `walkreal_0700000d_live`: **1 passed / 0 failed / 0 ignored (51.8 s)** on the final tree
  (the full mission: the crane chain, the walk-off, the catwalk, the ladder+roof descent, the
  bay, E #460, the boarding question's sister asserts, the east-extreme assert, the real ride,
  the real jump-drift drop, the yard, the plate, the photograph, the C5 kill, the mission hand-
  off to 0x0730_000d, all four objectives complete).
- `interactables_0700000d_live`: **11 passed / 0 failed / 0 ignored (42.2 s)** on the final tree.
- New print-only probe `probe20_beam_deck` (`#[ignore] --nocapture`) — the deck's cross-section
  and z-profile column pairs, every number in §1 cites it. Re-ran `probe20_beam_watch`,
  `probe20_static_map`, `probe20_roof_scan`, `probe20_interior_scan`, `probe20_west_scan`,
  `probe0d_route_scan` this window for the quoted numbers.
- The suite fast runner (`tools/test/suite.py`, started 15:07 after the alsa-sys link fix): the
  build completed; by 16:20 it had processed ~396 binaries with 5 failed tests — the 3
  `disc_golden` tests (the expected slim-ISO set) + `identity_disc::the_real_disc_passes_the_
  identity_check` + `fix_0c_pylon_live` (2 tests) — and 2 OOM CRASHes (`mp_stress_explosives_
  live`, `worldobj_live`, exit −9 — the 4 GB sandbox ran my probe/walkreal runs alongside).
  **The runner was still executing when this window ended** — no total line. All 5 failures +
  2 crashes are on the INHERITED tree (my branch changes test files only); the two binaries my
  task touches were re-run green by hand on the final tree (the totals quoted above). The next
  window should re-run the runner on a quiet box (or `--only` the crashed/failed binaries) for
  the total line.

## What's left

- **The boarding** — the exact question, pinned in the test's comment block and the sister
  asserts: WHERE does the game's player stand to board the gantry, and how do they REACH that
  surface? The data closes the mezzanine/ledge/pad chain (≥ 2.1 m gap, vertical-only jump), the
  bay (no stairs/ladder), and the gantry frame from the yard. Candidates a live run can check
  fastest are in § Live checks.
- The full-suite total line on the final tree (the runner was mid-run at the window's end).

## Live checks for the main agent

1. **The boarding resolver.** Fresh boot 0700000d, E the dock plate #460, then watch the player
   for the 3000-tick sweep: WHERE does the player stand when the gantry is at the west park, and
   what surface do they step/jump FROM? Watch `Player_FeetOnPoint` 0x194188's ground object: if
   the boarding surface is a DYNAMIC the script poses (a crate/pallet the port's statics lack),
   its glist hash will appear in the feet contact while the port sees none — that hash is the
   missing collision. If the boarding uses the gantry's own upper structure (the 15.99 rail or
   the 16.8 band over the east girder [D probe20_beam_deck]), a climb route the port lacks is
   involved — watch `Player_CollLadder` 0x196118 / the climb substate at the bay's west wall.
2. **The walk-edge rule on moving ground** (for the drop): confirm a real player WALKS off the
   girder edge or JUMPS (the port blocks the walk across an edge on the moving girder — the
   bottom sphere meets the edge face; if the game walks off, the port's edge rule differs on
   dynamic ground and the drop input should change from jump+drift to a held walk).
3. **The 0xEA riding trigger** (from the crane-transit report, still open): does crossing the
   gantry's trigger plane (0x0200_0a81, y 14.28) write ch 0x67 in the game, and does anything
   read ch 0x67? (The port carries the trigger but nothing reads the channel.)

## Usage

not visible to me (no token, cost or context-window figures are exposed to this session; the
only proxies are the shell timestamps: setup+branch 15:01–15:06 UTC, the alsa-sys link fix and
the runner start 15:07–15:10, the disc-data probes 15:24–15:50, the test rewrite and the
ride/drop debugging 15:53–16:12, the greens + the push 16:13–16:17, the report 16:18–16:30 UTC
2026-10-31).
