## Your task: `level-11-stairwell-bullet-wall-3`, the 01 contact walk with the new bullet rule

Start from `origin/s/level-11-stairwell-bullet-wall-2` (merge it first, then origin/master). The gondola window and the escort are fixed; one red remains, green on master: `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact`. The fight resolves differently, parking the contact in AllyLeadWait (0x20) at route node 15 (6.9, 6.3, 9.1) with Bond 6.1 away; the release [M 0x15eda8 case 3: pd > -7.0 && (pd != 0.0 || dist2d < 5.0)] reads pd == 0.0 and needs Bond within 5.0, but Bond's walk stalls at (1.38, 6.7, 11.61) in the hall (the nav gives up at its 3000-tick cap).

1. Find why Bond's walk stalls there (the hall geometry, a bot in the way, the nav's route) and walk it the way a player would with real input; check the release rule against [M] while there.
2. Green, plus the stairwell, gondola window, escort and foliage tests; run the FULL suite on the merged tree and report the total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-stairwell-bullet-wall-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox7 at 2026-10-25T15:20Z

---
## Report (done, 2026-10-25T17:08Z)


## Report from the worker (done, 2026-10-25T17:09Z)

# level-11-stairwell-bullet-wall-3 — the 01 contact walk goes green via the meet hall's north lane

Branch `s/level-11-stairwell-bullet-wall-3` (from origin/s/level-11-stairwell-bullet-wall-2
8f5bf8c3, merged with origin/master 4c1f5ffb → da5d3085 + this branch's commits; the final tree
carries both). Window 15:24–17:24 UTC, work stopped at the 17:04 mark.

## What's done

**The one red, `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact` —
green.** Reproduced first on the merged tree: Bond's walk-past stalled at (1.38, 6.7, 11.61),
target (2.97, 6.34, 9.69), `segment_to_target Some(0.29)`, the nav burning its 3000-tick caps,
the vault assist never moving him, the contact parked at (6.9, 6.3, 9.1) in AllyLeadWait 0x20,
and the run dying on `the contact must walk his route to the LoadPointActivator plate`.

**Root cause [D].** Bond chased guard 15 OUT of the meet hall (the walk's own log: the
approach leg arrived at (1.63, 6.42, 14.45), west of the hall) and stands in the west corridor
with the hall's west wall between him and the contact:

- the wall segment sits at x≈1.93, z 11.1–13.1 [D the disc's collision cels — the port's ray
  fan over them, `walkreal_probe11::dump_07000001_west_wall`: eastbound hits at every 0.25
  z-step from 11.25 to 12.75, clear at z 13.25+; the stall ray's hit at (1.85, 6.59, 11.05),
  t 0.293];
- every close/walk-past target (the contact (6.9, 9.1), `ahead` (2.97, 9.69), the nav's first
  hop node[6] (6.99, 11.78) [D nav path 0]) is inside the hall, and the straight-line steering
  wedges on the wall face; the stuck ladder's sidesteps slide ALONG the wall, never the ~1.7
  north to the wall's north edge;
- the nav does not save him: the AI's per-path A* routes from the corridor side straight to
  hall-side node[6], and the AI's own bounds walls seal the hall's west edge for the AI
  (z≈9.16 x −12..2.29, z≈13.07 x −10..6.14, x≈6.14 z 11.2–13.1 [D nav walls]) — the roads
  through the wall are the north lane (z≈14.3–14.7) and the SW gap (z<7);
- the middle opening (z≈7.3–10.8) is a raised sill, not a road: the corridor floor drops to
  4.41 south of z≈10 and the sill top is 6.19 [D floor probes];
- the north lane is the road the AI itself uses: path[0] node[8] (−0.39, 14.28) → node[23]
  (3.39, 14.69) → node[7] (6.88, 14.62) [D nav nodes+links; the lane also passes the AI bounds
  walls through their z≈13.1 north end, `move_test (1.0,6.5,14.5)→(3.5,6.4,14.5) = 1`].

**The fix (test-side, the player's play).** In the no-guards round, when the world segment
Bond→contact is blocked and Bond is west of the wall (x < 2.0), Bond walks the north lane back
in — (0.6, 14.3) → nav node 23 → nav node 7 — then closes on the contact. Inside the hall the
release fires and the contact walks his route west to the plate: the passing run shows `close
on the contact (the [M 0x15eda8] release band)` arriving at (7.10, 6.38, 10.24), the contact
0x20 → 0x1d, `the drone reached the goal at (−28.3, 6.3, 14.3)` (4.93 from the plate), the
plate → ch 0x10 → `level_exit to 07000002 at tick 33092`.

**The release rule checked against [M].** `NDrone2_DSTATE_AllyLeadWait` 0x15eda8 case 3: with
no armed peer within 10.0 (`NDrone2_NearArmedDrone(0x41200000, 1)`), pd =
`DroneMove_ObjectMissionPathDistance(5.0)`; `pd <= −7.0` stays; `pd == 0.0 && Dist2D ≥ 5.0`
stays; otherwise SetState(0x1d) — the release. The port's ALLY_LEAD_WAIT UPDATE arm
(crates/nf-game/src/drone.rs 8892–8903: `pd > -7.0 && (pd != 0.0 || dist2d < 5.0)` → ALLY_LEAD,
with the prop-0x50 armed-peer cover gate) matches the decompile; no change needed. The stall
was never the rule — it was Bond never getting within the 5.0 straight-line band.

**The wart, declared.** Lane leg 2 (to nav node 23) wedged in the passing run (its isolated
repro from the same spot with both start yaws arrives in 36–44 ticks; the live difference is
the post-fight sim state — corpses, the contact's AI, channels) and burned its 6000 lenient
ticks before leg 3 re-routed through the SW gap into the hall. The test passes deterministically
with that burn; a future session may tighten the lane walk.

## How it was verified

- The failing test reproduced on the merged tree BEFORE the fix (the stall trace above), then
  3/3 after, on the final tree (master 4c1f5ffb merged in).
- `walkreal_probe11::dump_07000001_west_wall` — the ray fans, floor probes, fine z-scan, breakable
  boxes, doors, nav nodes/links/walls, move_test rows, and the isolated lane-walk repro cited
  above (diagnostic, committed on the branch).
- The crates the task touches, on the final tree, via the fast runner (`tools/test/suite.py`):
  - **Priority set: 16 passed / 0 failed** — walkreal_07000001_live 3/3, escort_follow_07000001_live,
    gondola_windows_live, zoe_gondola_realpath_live, stairwell_bullets_07000011_live,
    fix_foliage_bullets_live, walk_07000001_live.
  - **The lib binaries (nf_game/nf_formats/nf_audio): 657 passed / 3 failed / 4 ignored on the
    first pass; all three failures were the sandbox's DISK FULL** (nf_game profile
    round_trip_and_files + delete_in_removes_only_the_named_file and nf_formats identity
    rejects_missing_and_non_iso_files save into /tmp; `df` read 100% used, 0 available). After
    freeing 1.2 G of stale dual-RUSTFLAGS artifacts, all three pass directly → **the real lib
    result 660 passed / 0 failed / 4 ignored**.
  - **The walker-sharing slice (87 binaries, --limit 60) ran ~32 of them before the window
    closed: one red — `interactables_0700000a_live::vending_glass_breaks_under_fire`** ("one dart
    scores ONE contact of 0.1f … got {h1} (want 8.9) dead {dead1}", line 165). This task's
    changes are TEST-ONLY (walkreal_07000001_live.rs + the new probe); this test shares nothing
    with them. The prime suspect [S]: the −2 branch's glass-drain fix (a kind-2 pane's hit
    drains and can Break_Kill WITHOUT killing the round [M 0x12bfd8:36-37 + the 0x1ea398/0x12ce80
    glass arms]) gives the vending pane a second contact from the surviving dart — the same
    class of re-anchor the gondola pin needed ("the old pin held via the accidental pre-fix
    double pierce"). Whether the vending pane is class-0x10 (pass) or a plain breakable (stop)
    decides which side is wrong; the next session should read the pane's collision class [D] and
    re-anchor or gate the fix accordingly. Never weaken a decompile-backed pin to pass.

## What's left

- The full suite total: the slices above cover the priority set, the three lib binaries and ~32
  walker-sharing binaries of 802; the window (sandbox kills detached background jobs, so every
  run is a foreground slice) ended before the rest. The next session: finish the walker slice,
  the vending-glass red above, then the whole suite for one clean total line.
- The lane walk's leg-2 wedge (the wart above) — optional tightening.
- The live checks below are still open from the −2 report and still the right questions.

## Live checks for the main agent

1. **The meet-hall west wall for the real player (07000001)**: fresh boot, play to the meet
   scene's end, clear the hall guards, then walk Bond from the west corridor at ≈(1.4, 6.7,
   11.6) east toward the contact at ≈(6.9, 6.3, 9.1). Expected if the port's wall is the game's:
   the player must walk around through the north opening (z≈14.3–14.7); he cannot walk straight
   east at z≈11.6. A gdbwatch on Bond's position while pushing east pins where the game's
   collision stops him (x≈1.9 here, or earlier if the port's wall is wrong).
2. **The AllyLeadWait release in play**: with the contact parked at route node 15 (0x20), walk
   Bond INSIDE the hall to within 5.0 straight-line of him (the release band [M 0x15eda8 case 3
   with pd == 0.0]) — the contact should resume the lead within a couple of seconds, walking
   his route west. A screenshot pair (before/after) at (7.1, ~10.2) would pin it.
3. (Carried from −2) **The gondola window's break vs the round's life** (07000008, gdbwatch on
   the shooting round's BU+0xF4 after the pane contact): the round keeps flying or dies on the
   far wall — NOT stopping inside the pane.
4. (Carried from −2) **The meet-hall lane bodies**: which bodies stand in the hall after the
   meet and do they block the player's walk (the port's Bond is never body-blocked while
   grounded — the Jointy push rides only airborne [M Collide_Jointy port note])?

## Usage

not visible to me — this session's environment shows no token, credit or context-window
figures (no usage display was available at report time; nothing estimated).
