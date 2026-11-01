## Your task: `level-05-cable-full-length`, 05: Bond stops short on the zip cable

On 07000005 Bond is stopped part-way along the castle zip cable; he should be able to traverse more of it. Find the game's cable extent and stop rule [D the cable placement's end points; M the cable/hang class's end test, Player state 0xB], fix the port so the hang-move reaches the game's ends, and pin the reachable span both ways. Run the cable binaries, walkreal_07000005_live and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-cable-full-length`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-01T02:02Z

---
## Report (done, 2026-11-01T07:10Z)

# level-05-cable-full-length — done

Branch `s/level-05-cable-full-length` (base e79794da, merged origin/master babf9247 → merge commit
e53297fe). Task: on 07000005 Bond stops part-way along the castle zip cable; find the game's cable
extent and stop rule, fix the port so the hang-move reaches the game's ends, pin the reachable span
both ways, and run the cable binaries, walkreal_07000005_live and nf-game.

## The finding: the port already rides the game's full span — the drawn cable is what's longer

**The ride extent was never short.** The game's own travel segment for the hand-over-hand wire is
the wire OBJECT's world-sphere span: centre ± `Spherical(obj+0x8C − 0.5, obj+0x54 + 1.5707964, 0)`
[M `Player_Wire` 0x197658, segment build @0x1976dc–0x197730; there are no endpoint params on the
wire instance — `Wire_Create` 0x1981d8 reads only type/params/channel into an 8-byte instance]. On
Wire_Cable #251 that is z **2.724..9.802** at (x −23.485, hang y = centre.y − 1.0207), span 7.078 u
[L obj+0x8C = 4.0390439 @ ee_start_05.bin 016dc48c, obj base 016dc400; centre/yaw 016dc430/016dc450].

**The drawn cable is 2.1× the rideable span.** The visible line is the Object_Cable_* CEL pieces
plus OBJECT_CableEnds [D placement #252]: z −1.070..14.068 (15.14 u) at the rope's own height.
So the game itself stops Bond part-way along the VISIBLE cable — 1.29 u inside the drawn line's
south end (2.724 vs −1.070) and 4.27 inside its north end (9.802 vs 14.068). "Traverse more of it"
would contradict the decompile's own segment build; the port is not the thing that stops short.

**The stop rule [M Player_Wire 0x1977c0–0x197834]**: the strafe axis (action 1) is ZEROED when the
squared 2D distance to the travelled-to end is ≤ 0.5 (≈0.707 u), or when the pre-snap squared
distance to the opposite end exceeded the segment's own squared length; the per-tick snap
[M 0x1976f0–0x197708] pulls a beyond-end rider back onto the end. The reachable band per end is
therefore (end − 0.707 … end], plus the in-flight hand cycle completing after the kill fires.

**No sim change was needed** — the port already rides exactly the game's span. The both-ways span
is pinned in the new test; a genuine [S] observation (the sway lane's second term
`sin(−yaw)` vs the decompile's `cos(−yaw + π/2)` at 0x1978e0, magnitude 0.0542 u per add, agrees on
this level's yaw, disagrees in sign off-axis) went to `docs/atlas/wire-grapple.md` as an [S] note,
not into pass/fail.

## What was pinned, and how

New test `crates/nf-view/tests/cable_span_live.rs` (3 tests, all through the real input path — the
icon-context jump grab [M `Player_HandleJump` 0x1916b0:118-147], not a synthetic state write):

1. `probe_the_cable_run_placements` — every cable-named world instance with its placement
   translation and bounds [D 07000005.bin placements]; pins how far the visible cable runs.
2. `the_parsed_wire_matches_the_snapshot_geometry` — parsed centre (−23.4855, 34.4676, 6.26292),
   yaw −1.5708, radius 4.039, type/params/channel `00 00 02 00 …` against the EE snapshot
   [L 016dc430/016dc450/016dc48c/016dc510]; travel ends = centre.z ± (radius − 0.5) [M].
3. `the_cable_span_reaches_the_game_ends_both_ways` — grab near B, ride to the A stall, hold into
   the end (stays), turn and ride back to the B stall, hold, dismount alive. Asserts both stalls
   sit in the game's stop band (A z 9.558 = 0.244 short of 9.802; B z 2.740 = 0.016 short of
   2.724), that the ride never passes beyond an end [M the snap], and that the round trip never
   drops (substate stays SS_WIRE).

No existing test was changed; nothing was weakened.

## What was run (final tree, merged with origin/master)

The suite runner (`tools/test/suite.py`) cannot run to completion in this sandbox: background
processes are reaped between tool calls (nohup and setsid both die), and the 9.9G disk filled twice
during the ~1039-binary link phase (it was cleaned: stale pre-merge binaries and caches removed).
The suite's run phase was therefore driven by a resumable driver with the runner's exact
per-binary contract (NF_SILENT=1, `--test-threads=1`, 600 s wall limit, per-binary `NF_ISO`),
required binaries first, results checkpointed per binary. 454 of the 1039 test binaries ran inside
the window; the rest are plain nf-view coverage with no known open question.

Totals for what ran (the numbers to quote):

- **All four crates, 454 binaries: 2112 passed, 6 failed, 74 ignored.**
- Cable set (all green): `cable_span_live` 3/0/0, `wire_live` 2/0/2i, `fix_wire2_live` 3/0/0,
  `wire_repro` 0/0/1i, `walkreal_probe0c_wires_live` 1/0/0, **`walkreal_07000005_live` 11/0/0**.
- **nf-game: 98 binaries, 993 passed, 0 failed, 9 ignored.**
- nf-formats: 41 binaries, 194 passed, 4 failed, 3 ignored.
- nf-audio: 4 binaries, 28 passed, 0 failed, 0 ignored.
- nf-view: 311 of 896 binaries, 897 passed, 2 failed, 62 ignored.

The 6 failures are all environmental, none gameplay:

- `disc_golden` 3 — the documented slim-ISO allowance (rule 5).
- `identity_disc` 1 — asserts the full retail image's 1,191,072 sectors; the slim on-foot ISO is
  518,435 sectors (1,061,754,880 bytes). The test is correct for the real disc; it cannot pass on
  this trimmed image. Left untouched on purpose (weakening it would be wrong).
- `fix_0c_pylon_live` 2 — `wgpu adapter NotFound` (no GPU in this sandbox); these are render-spot
  tests for the main agent's machine.

## What's left

- Nothing on the task itself: the span is pinned both ways with the game's own law, and no port
  change was required. The 585 unrun nf-view binaries are coverage, not open work; run
  `tools/test/suite.py` on a machine that fits them for the full total.

## Live checks for the main agent

Optional (the [L] snapshot already pins the geometry), on a FRESH boot of 07000005:

1. fieldprobe read `obj+0x8C` of the Wire_Cable object (snapshot address 0x016dc48c, obj base
   0x016dc400): expect 4.0390439. If a live run shows the same, the radius — and hence the ride
   span — is confirmed outside the snapshot.
2. fieldprobe write/watch Bond's position while riding the cable to its north end: expect the ride
   to stall around z ≈ 9.56–9.80 at x ≈ −23.49 (never past z 9.802), and the grab spot under the
   drawn cable's mid-span to still grab (the grab segment is radius − 2.0 [M 0x1971f8–0x197208]).

## Usage

not visible to me — this CLI session exposes no token, cost or context-window readout; nothing to
copy. The only observed limits were the sandbox clock (started ~05:39 UTC, work stopped 07:10 UTC
ahead of the 07:18 cutoff) and the 9.9G disk.
