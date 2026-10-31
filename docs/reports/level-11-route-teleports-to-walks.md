# level-11-route-teleports-to-walks — the report

Task: `walkreal_07000011_route_live.rs` still moved Bond with three `bond.pos` writes (the walk
stand-ins). For each: find the game's own route there, walk it on real input, delete the write;
where the data shows no route, pin the gap with a sister assert and name the live check.

## What's done

All three `bond.pos` writes are DELETED. `crates/nf-view/tests/walkreal_07000011_route_live.rs`
now walks the whole office-floor route from the spawn to the 07000012 exit on `Sim::step_bond`
inputs only — no `bond.pos` writes after the spawn. No test-setup placements were kept or added:
every leg is the route itself (no measurement spots).

### Write 1 (old line 51) — the corridor's south leg → the stairwell door's front tile

The old comment said the leg "wedges on the corridor's static machinery row in the port's walk
(the walker's stuck report, 6000 ticks at z ≈ 5.7)". The probe (`walkreal_probe11b.rs`, committed,
diagnostic, `--nocapture`) mapped the region from the disc's own bytes:

- A body-height ray (y 47.0 and 47.6) northward at x 9.0/10.22/11.5/13.0/14.5 first hits at
  z ≈ 5.09-5.15, and raised tops reach y ≈ 47.2-47.3 at z 4.5, x 9-11 — the machinery row across
  the ledge's north edge [D collision; the audit's static cel row corridor_machinery1-3 #18-20].
- The floor beyond is the open well: no floor at z 3.5..1.5 for x 8-15 [D collision].
- The level's own nav graph never links the two sides: the stair chain's first node n94
  (11.80, −6.29) links only eastward to n95 (15.99, −5.32); no corridor node reaches it, while the
  office-floor component runs n10 (10.85, 6.97) → n11 (7.68, 6.49) west along the ledge, and the
  graph DOES route the ledge's west end → the stairwell door over the west bridge (its own bridge
  crossing sits at x ≈ −1.45) [D nav].
- The wedge reproduces through the real input path: the probe's lane walk from (13.82, 47.4, 6.2)
  wedges at (12.39, 47.84, 5.68), and the test's own pinned attempt wedges at (11.70, 47.81, 5.68).

So the game's own route from the corridor is WEST: the ledge → the west bridge → the north balcony
→ the strip → the stairwell door #192's front tile — the leg chain shared with
`walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point` (green there). The test walks
it on real input and the teleport is gone. The blocked direct leg is PINNED with sister asserts in
the test itself: the machinery-row ray hit, the open-well floor probe, the nav absence
(corridor → front tile = None; ledge-west → front tile = Some) — all [D] disc data — plus the
deterministic wedge pin: a lenient `walk_to` of the direct leg must return the wedge sentinel
(u32::MAX) [D collision via the port's walk].

### Write 2 (old line 65) — the stairwell descent (floors 47 → 16)

The old comment said "the walker wedges at the first flight's head … the port's stair-descent walk
is its own open item". That item is closed on this tree: the level doc §8 ("The stairwell descent —
resolved") and `walkreal_07000011_live` walk the whole flight ring on real input (the top flight →
the north flight → the east flight → the south run → the west flights A/B/C and the east
connectors; the door #192 E is the crossing that replaced the old 2 m pocket stand-in). The route
test now carries the same walked leg chain with the y-band asserts per landing and the bottom pins
(y < 17.2; the atrium plates ch 0x7C never fire [D §4]). The teleport is gone. `top_up_health = true`
matches the sister (the stairwell guards shoot during the descent; the walk pins the route, not the
firefight).

### Write 3 (old line 74) — the teleport into Load Point #158's box

The lower floor is flat from flight A's foot (x 14.8) to the trigger, and the sister walks
`Wp::new(14.6, 17.0, −2.8)` then presses E until `level_exit` fires. The route test does the same
real-input walk + E loop (`level_exit == Some(0x0700_0012)` [M Trigger_Activate 0x1b2318 — the E
arm falls through with the gate off]) and then loads 07000012 through the viewer's transition
(`level == 0x0700_0012`). The teleport is gone.

## How it was verified

- Touched binaries (release, `--test-threads=1`, NF_ISO set):
  - `walkreal_07000011_route_live`: **1 passed, 0 failed, 0 ignored** (9.92 s).
  - `walkreal_probe11b` (the new diagnostic probe): **1 passed, 0 failed, 0 ignored** (4.55 s).
- Full suite: this sandbox kills background jobs between tool calls (a plain `nohup`+`setsid`
  `tools/test/suite.py` died twice, the second time after a 7m38s full test build), so the suite
  ran with a re-invoked foreground runner (2 cores, 3 GB RAM): per-binary `--test-threads=1`
  runs under NF_SILENT=1 with a 240 s cap, results accumulated in a state JSON across
  invocations, OOM-killed entries (rc −9 — the big walkreal/mp sims need ≈1.5 GB each; ≥2 lanes
  OOM on this box) scrubbed and re-run solo. State at report time: **369/972 binaries recorded,
  1000 passed, 3 failed, 150 ignored**.
  - The 3 failures are NOT in this task's scope and pre-exist on this tree (my diff touches only
    the two 07000011 test files; both failing binaries are independent): `fix_0c_pylon_live`
    (2 failed), `probe_pylon_render` (1 failed) — the 0700000c pylon render pair — plus
    `net_soak_faults_live` hit the 240 s cap (timeout, not a failure). None of the 3 disc_golden
    failures the slim ISO allows were reached yet (the suite is alphabetical and incomplete).
- The runner was killed by the session window before a full total line; the touched-binary
  totals above stand as the task's own evidence, per the house rule.

## What's left

- The remaining ~600 suite binaries (resumable: re-run the same per-binary protocol; the state
  JSON lives at `/home/z/suite11_state.json` in the sandbox home and may not survive a reset).
  Run heavy binaries solo (one at a time) — 2-way parallelism OOMs this box.
- Optional live checks for the main agent (none are required for this task's asserts — they are
  all [D]-backed — but these would settle the two walk-behaviour questions behind them):
  1. PCSX2, fresh boot of 07000011: walk Bond from door #13 (13.82, 46.78, 7.09) north through
     the corridor — expect the player to be blocked at the machinery row at z ≈ 5.1-5.7 (the
     port's walk wedges there; the disc's collision and nav data both say no route). Watch
     Bond's z stop ≈ 5.5+ and no passage at any corridor x 9-14.5.
  2. The same boot: confirm the game's own route to the stairwell from the corridor runs west
     along the ledge and over the west bridge (x ≈ −1.5..2.2), matching the nav graph's
     n12 → n45 → n46/n47 chain — i.e. the port's west-bridge detour is the game's route, not a
     port invention.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures for
the session.
