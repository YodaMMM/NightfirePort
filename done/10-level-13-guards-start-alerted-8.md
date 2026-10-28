## Your task: `level-13-guards-start-alerted-8`, the vision-bit census with Countdown Part 1's last leg green

Continue `origin/s/level-13-guards-start-alerted-7` (merge `origin/master`). The escort and Rook reds are fixed there; one red remains: walkreal_07000014_live `walk_the_infiltration_to_the_delta_exit` wedges at leg 6 ("to the Delta exit #944") at (23.31, 1.14, -24.64), target (27.79, 1.28, -25.32), segment_to_target Some(0.1227), cel 16, no bot within 3.0 - world geometry the RNG-shifted route now steers into (master walks it green). Diagnose the geometry [D cel 16's faces at that spot] and the route difference, then fix the port if a port collision rule is wrong [M Player_Collision], or give leg 6 the player's real approach with the cite. Run the FULL suite and quote its total line; Double Cross Part 1 and Countdown Part 1 must stay finishable.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-guards-start-alerted-8`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-28T16:02Z

---
## Report (done, 2026-10-28T17:27Z)


---
## Report from the worker (done, 2026-10-28T17:27Z)

# level-13-guards-start-alerted-8 — the delta-exit leg 6 walked as the player's real approach

Branch `s/level-13-guards-start-alerted-8` = `origin/s/level-13-guards-start-alerted-7` (b60674aa)
merged with `origin/master` (8a966fa3 — the merge was the switch_wall probe + docs), plus one test
change (leg 6's approach) and one new probe test. **No sim/code changes this round** — the wedge was
the walk's approach line, not a port rule, and the diagnosis below is why.

## The leg-6 wedge: what the geometry actually is

`walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit` wedged at leg 6 ("to the Delta exit
#944") at (23.31,1.14,−24.64), target (27.79,1.28,−25.32), `segment_to_target Some(0.1228)`, cel 16,
no bot within 3.0. The new probe (`crates/nf-view/tests/probe_14_delta_leg6.rs`, assertion-free,
--nocapture) reproduces the wedge from a teleport in ~3 s and dumps the spot. Findings, all from the
probe on this tree (the port's own parsed level = [D] the disc's collision/door data):

1. **The blocker is the doorway's east reveal**, not a bot and not a port collision bug: the ray to
   the target from the wedge hits a west-facing face (normal (−1,0,0)) at the shelf's east edge
   (the x ≈ 24.06..24.46 / y 0..2.35 slab the tri dump shows); Bond pressed against it walks with
   ZERO displacement for 2400 ticks (the probe's per-60-tick trace: position identical, yaw already
   facing the target — the steer has nothing left to do).
2. **The wedge spot is a lower shelf**: teleporting to (23.31,1.14,−24.64) settles Bond DOWN to
   y = 0.65 (the shelf floor ~9e-5 below his feet); the doorway approach floor and the exit sit at
   y ≈ 1.0..1.3 (the doors #121/#122 anchor y 1.2, the exit target y 1.28). The direct steer from
   the wake-touch spot descends onto the shelf and presses into the reveal; master's unshifted
   route slipped the doorway diagonally instead — that is the whole route difference.
3. **The exit pocket is OFF-NAV**: `nav_route` from the wake-touch spot (20.3,0.9,−23.5) to the
   exit returns None — the level's own bots never route there; it is a player-only pocket. The
   game's player enters it THROUGH the double slide doors #121/#122 (24.02,1.2,−22.8 [D], kind 2).
4. **The doors answer the player's E from every state the walk meets** — CLOSED (0x2, the
   spawn-state probe) and CLOSING (0x10 pos01 0.97, the real run's state): E is a TOGGLE [M
   `Door_Activate` 0x135be0] and the activate turns a closing slide back to OPENING (the port's
   door.rs `activate` cite). The probe's E leg: `use done (reacted true)`, the doors end
   0x4 / pos01 1.00 (OPEN).

## The fix (the player's real approach, with the cite)

Leg 6 is now three walked legs (the test's comment carries the same chain):

- `the delta doorway approach` → (22.9,1.0,−22.8) r 1.2 — stay on the approach floor, off the shelf;
- `E the delta slides` → Act::Use at the doors (24.0,1.2,−22.8 [D placements 121/122]) — the
  player's own use, a toggle [M 0x135be0];
- `to the Delta exit #944` → (27.79,1.28,−25.32 [D]) r 1.0 — unchanged target.

**Verified**: the probe walked the three legs green from BOTH the real leg-6 start (21.39,1.04,−22.57
— the run's own resupply spot) AND the exact wedge spot (23.31,1.14,−24.64 — self-healing: the first
leg walks Bond out of the zero-movement pocket north-west, off the shelf), with E reacting and the
exit leg arriving (+115 / +137 ticks). The full binary: `walkreal_07000014_live` **5 passed; 0
failed** (the delta walk included — E at the exit fires the fallback write + the 07000015 load and
the asserts hold). Countdown Part 1 is finishable again on the RNG-shifted tree.

## What ran

- Build: `cargo test --offline --release --no-run -p nf-game -p nf-view` clean (twice — after the
  merge and after the leg-6 edit; the Act import fixed the test's compile).
- Direct runs (cargo test --release --test-threads=1):
  - `walkreal_07000014_live` **5 passed; 0 failed** (Countdown Part 1 finishable — the round's goal);
  - `walkreal_07000013_live` **1 passed; 0 failed** (Double Cross Part 1 finishable);
  - `probe_14_delta_leg6` (the new probe, 3 tests) — evidence, assertion-free.
- The fast runner (`tools/test/suite.py`) on the final tree, foreground slices (this sandbox kills
  background jobs between tool calls — setsid included — so the runner cannot be left running; the
  -6/-7 windows hit the same wall):
  - slice 1 (the mission-walk family, 14 binaries, `--only`): **50 passed; 0 failed** total across
    walkreal_07000004 (5), party_door_lock_07000007 (2), walkreal_0700000c (2), switch_wall_probe
    (3, master's new test on the merged tree), walkreal_07000016 (6), walkreal_07000007 (2),
    walkreal_07000011_route (1), walkreal_07000002 (1), walkreal_0700004a (1), walkreal_0700000a
    (5), walkreal_07000014 (5), walkreal_07000005 (8), walkreal_0700000d (1), walkreal_0700001b
    (8) — zero FAILED/TIMEOUT/CRASH lines.
  - the full-suite total line did not fit the window: the audio-crate binaries alone burned a
    9.3-min slice at 54 binaries, and the remaining walkreal binaries exceed the per-binary 600 s
    limit inside a single tool call (walkreal_07000001_live's full castle walk). The next message
    resumes from the branch with the runner for the complete sweep; the changed surface this round
    is one test file, and every binary that file's family shares ran green above.

## What's left

1. The full four-crate suite total line on the final tree (the window constraint above; the next
   message's runner).
2. The -7 report's live checks (Rook's post-wake states; the delta-exit render at
   `--bond 23.3,1.3,-24.6` facing east) still stand — this round's static diagnosis answers the
   render question: the east reveal of the #121/#122 doorway and the y-0.65 shelf south of it.

## Live checks for the main agent

1. The -7 report's check 1 (Rook's post-wake states, gdbwatch +0x58/+0x228) — unchanged, still the
   sight-chain question the -7 round flagged.
2. **The delta doorway walk-through** (new, confirms this round's read): fresh boot, 07000014, walk
   Bond to the #121/#122 doorway (24.02,1.2,−22.8), press E, walk through east, then to the exit
   (27.79,1.28,−25.32). Expected: the doors toggle open, the passage is the doorway (the south
   shelf at y 0.65 is not part of the walk), the exit trigger fires.

## Usage

not visible to me
