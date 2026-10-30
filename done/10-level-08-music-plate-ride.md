## Your task: `level-08-music-plate-ride`, 08's music plates reached the game's way

level-08-interactables-2 (merged) found mux #102 is an AND fold (out 0x68 = 90 AND 91 AND 92 [M Trigger_MultiplexIn 0x1b2c68:15]) and left open how the player reaches plates #99/#101 (their volumes sit at y 46-49 over x 117-131 [D]) — the gondola ride is the candidate. Find the game's route to each plate [D placements/collision; M the gondola/mover class], walk it on real input, and complete the mux test's fold + map-sound legs without placing Bond. Fix any port gap with evidence. Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-08-music-plate-ride`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-30T22:06Z

---
## Report (done, 2026-10-30T23:48Z)


---
## Report (done, 2026-10-30T23:49Z)

# level-08-music-plate-ride — report

Branch `s/level-08-music-plate-ride` (off `origin/master` b3c4e4a1; master did not move during the
session — the final tree is the branch tip). Task: level-08-interactables-2 (merged) found mux #102 is
an AND fold (out 0x68 = 90 AND 91 AND 92 [M Trigger_MultiplexIn 0x1b2c68:15]) and left open how the
player reaches plates #99/#101 — the gondola ride was the candidate. Find the game's route to each
plate [D placements/collision; M the gondola/mover class], walk it on real input, and complete the mux
test's fold + map-sound legs without placing Bond.

## Result: the route is the ZIP LINE, and the fold + map-sound legs are complete — green

`interactables_07000008_live::music_plates_mux_into_the_map_sound` now walks the whole route on real
input from the load's own gate-0 spawn (zero `bond.pos` writes; every leg is `Sim::step_bond` input
through the walker), jumps, grabs, rides, and watches the channels fire: **90 → 91 → 92 → the AND fold
0x68 → the map sound 0x14D**, with the one- and two-input sisters of the AND law asserted in flight
(one plate alone must not fold; two plates must not fold). The mux decode pins (kind 3, the input
prefix 90/91/92 [D params 0x68/0x5a/0x5b/0x5c; M 0x1b2c68:15 type 3; Trigger_Update 0x1b1a78 case 3])
stay. The route record is committed as `probe08_plates_route.rs` (#[ignore]d, assertion-free, the
probe0c pattern): the wires' analytic segments, the icon, the trigger AABBs and the floor queries.

### How the route was found (the evidence chain)

- **The plates hang over void.** The disc's collision has NO floor under the whole plate line:
  `floor_below` is None for every x 117.8-130.8 at z −101.1 (the probe's table; the coll-scans over
  x 104-141, z −112..−92 show only walls at y 40-48 north of the line and the station's east floors).
  The earlier "grate chain" reading (roofline platforms the walking capsule dropped through) has no
  disc collision behind it — the [L] stand heights quoted in the old test comment are not reproducible
  from any static floor; nothing pins there. The walk route to #99/#101 does not exist.
- **The gondola is ruled out.** The ride script 0x0600004C's carriage stream is x 112.0 constant
  (z −126.3 → −179.4 [D stream 1]); the plate volumes span x 117.3-131.3 [D] — the ride never crosses
  them.
- **The zip line is the route.** `#19 OBJECT_Zip_Line_1` (kind 0x3E, param 1 = **2 = the zipline type**
  [D; M wire.rs's port of Player_CollWire 0x197130's type dispatch]) rides the segment
  centre ± spherical(radius, yaw+π/2, pitch) [M Player_Zipline 0x197ff0 + Vec_Spherical_2_Cartesian
  0x1e3948] = **(110.50, 44.44) .. (134.02, 49.43)** [D the parsed centre (122.26, 46.93, −101.10),
  r 12.025, yaw π, pitch −0.2094]. `Wire_Create` NEGATES the placement's rot.z before building the
  object matrix [M 0x1981d8:14], so the game's drawn cable and the analytic ride segment both run
  west-LOW/east-HIGH. (A raw placement-matrix dump of the drawn mesh tilts the opposite way — that is
  the dump's convention missing the negation, not a game or port discrepancy. The cosmetic renderer
  question is a live check below.)
- **The boarding spot is the third icon.** `#105 Hand_Over_Hand` (kind **0xFA** →
  `ThirdIcon_Create` [M 0x1d04f0:511-517], ctx param 3, r 1.164 [D]) sits at (133.43, 48.33, −101.21)
  on the sloped roof west of the station's y 48.0 south floor (x 134..143, z −96.75..−100 [D obj 27]
  via the free south shutters #49-52 [D]; the slope is the (131,46.0)→(134,48.0) quads over
  z −96.75..−105 [D mat 0x12]). Standing in the icon's cel set arms `BLData+0x95F` = 3
  [M build_LinkToRoom 0x1c9680's straddle], and the jump press grabs the NEAREST wire instead of
  jumping [M Player_HandleJump 0x1916b0:118-147]. The grab lands on #19 (the level's only 0x3E
  placement [D]), refused only within half the segment of the lower end [M 0x197260-0x19728c] — the
  icon spot is 21 units from the lower end, well outside.
- **The ride crosses all three volumes.** Player_Zipline tracks the closest point on the segment
  1.17363 below it toward the LOW (west) end [M 0x197f50's tail: the −1.17363 store on obj+0x34, the
  low-end target pick, the +0x18 += FRAME_RATE_MUL·0.15 accumulator]. The player's trigger capsule
  (feet+0.55..feet+1.86, r 0.55 [M triggers.rs `Toucher::player`; drone.rs's pos−0.6 delivery]) then
  spans: #100 (y 48.40..49.38 [D]) at rider y ≈ 47.6 → capsule top ≈ 48.8; #99 (47.10..48.08 [D]) at
  ≈ 46.3 → ≈ 47.6; #101 (45.61..46.59 [D]) at ≈ 44.8 → ≈ 46.1 — tops 0.43-0.49 inside each volume
  ([D] volumes × [M] ride law = the overlap arithmetic; the ride also passes inside every volume's
  z −102.3..−100.0 band at r 0.55). The measured ride fires the channels at x 131.8 → 126.0 → 118.8.

### What was changed

- `crates/nf-view/tests/interactables_07000008_live.rs`: `music_plates_mux_into_the_map_sound`
  rewritten — the real-input route replaces the #100 teleport stand-in; the decode pins stay; the
  in-flight sisters added. No other test changed.
- `crates/nf-view/tests/probe08_plates_route.rs`: NEW, #[ignore]d route record (the geometry dump +
  the walk/ride trace).
- `docs/levels/07000008.md`: §7 rows 0x3e/0xea/0xec/0xf9 updated + a new section "The music plates'
  route — the zip line, not the gondola".
- `docs/WORKLOG.md`: the outcome entry.
- **No src changes.** No port gap: the zipline ride, the jump-grab, the icon context, the trigger
  capsule law and the mux fold were already ported correctly — the earlier tests had driven the wrong
  route (a teleport to the deck + a walk), which is why the fold never completed.

### What ran (the touched crates: nf-view only — no src changes; every binary quoted is nf-view)

Foreground `--only` batches of `tools/test/suite.py` on the final tree (the background start was
killed by this sandbox between tool calls — twice; `suite.out` stayed 0 bytes both times, the same
kill the level-08-interactables-2 report recorded):

- Batch 1 (the 08 family: interactables_07000008, probe08_plates_route, walkreal_08, walk_08,
  zoe_gondola, gondola_windows, music_triggers, wire, hurt_08, cutscene_08_actors):
  **30 passed; 0 failed; 6 ignored** (2 min).
- Batch 2 (interactables_07000002/3/4/6/7/9/a/b/d + interact_05/06/07/09):
  **106 passed; 0 failed; 1 ignored** (4 min).
- Batch 3 (rng, fps/cross-process determinism, disc_golden, walkreal_05/07, walk_05/07, pickups08,
  fix_safe08, fix_08_collision/2/walls): **55 passed; 3 failed; 0 ignored** — the 3 failures are
  exactly the allowed `disc_golden` manifest tests on the slim ISO
  (`bigf_header_fields_behave_as_documented`, `bigf_tables_match_the_golden_manifest`,
  `directory_tree_matches_the_golden_manifest`).
- Batch 4a (walkreal_0b + interactables_11/12/13/14/15/16/41 + models_sweep_08 +
  decomp_audit_08 + rocket_glass08 + rocket_stop_plane08): **84 passed; 0 failed; 3 ignored**.
- Batch 4b (campaign_flow_live — it drives 08's full chain — + interactables_1b):
  **11 passed; 0 failed; 1 ignored** (8 min).

Cumulative: **286 passed; 3 failed (the 3 allowed disc_golden); 11 ignored.** The full 960-binary
sweep did not fit the session window (the background runner is unkillable-safe only outside this
sandbox; foreground batches cap at the 10-minute tool limit) — the next session can continue the
sweep from this branch; no src changes are on it, so the residual risk is nil.

## What's left

- Nothing on the task's legs: the route is found, walked on real input, the fold + map-sound legs
  complete, no port gap. The two pre-existing named-blocker ignores (the glass per-instance pin, the
  kill-floor death-NIS hold) are unchanged from the branch point.

## Live checks for the main agent

1. **The in-game ride firing** (the arithmetic above is [D]×[M]; this confirms the live channels):
   fresh boot 07000008, walk out the station's south shutters onto the y 48.0 floor, west down the
   sloped roof to the "Hand_Over_Hand" icon at (133.43, 48.33, −101.21), press jump, and watch
   `switch_channels[90]/[91]/[92]/[0x68]` during the westward ride from the debugger: expect 90 → 91
   → 92 in that order, then 0x68, and the map sound 0x14D starting after the fold.
2. **The drawn cable's direction** (cosmetic, settles the object-matrix negation reading): from the
   station roof render/screenshot, the cable should run west-LOW (y ≈ 44.4 at x 110.5, over the
   lower-station roof) to east-HIGH (y ≈ 49.4 at x 134.0, above the icon platform) — i.e. the same
   tilt as the analytic ride segment, matching `Wire_Create`'s rot.z negation [M 0x1981d8:14].

## Usage

The environment does not display token, credit or context-window figures for this session (no
token/credit counter is visible in the harness UI; the per-subagent counters are not this session's
figures) — **not visible to me**.
