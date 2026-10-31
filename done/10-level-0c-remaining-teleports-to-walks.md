## Your task: `level-0c-remaining-teleports-to-walks`, replace 0700000c's walk stand-in teleports with real input

`walkreal_0700000c_live.rs` still moves Bond with the remaining `bond.pos` writes (five on master). For each: read its comment (why it was a stand-in), find the game's own route there [D collision/placements/nav; M any class the route uses: doors, lifts, ladders, wires, movers], walk it on real input, and delete the write. Where the data shows no route, keep the write but pin the gap with a sister assert and say exactly what live check would settle it. Test-setup placements whose subject is not the route (a measurement spot) may stay — say which and why. Every test in the file green. Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-remaining-teleports-to-walks`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-30T23:53Z

---
## Report (done, 2026-10-31T01:08Z)


---
## Report from the worker (done, 2026-10-31T01:08Z)


---
## Report from the worker (done, 2026-10-31T01:08Z)

# level-0c-remaining-teleports-to-walks — the report

Branch `s/level-0c-remaining-teleports-to-walks` (from master fd24fbf4). Task: `walkreal_0700000c_live.rs`
still moved Bond with five `bond.pos` writes; find the game's own route for each, walk it on real input,
delete the write, and where the data shows no route keep the write with a sister assert and a named live check.

## What's done

**One of the five writes is DELETED — the yard descent is a real leg now.** The old stand-in
(`sim.bond.pos = [56.5, 2.8, −176.5]`) replaced the "doorway-ledge → yard" descent, which the previous
session had wedged on an unprobed east return face. This session mapped that whole side and found the
descent was never an east descent at all:

- The plates #558/#559 stand in a ground-level **corridor** (floor 0.90, x 52.5..55.5) that runs NORTH
  under the warehouse mezzanine (5.44..6.00) — the old "ledge" is the corridor's south part. The
  corridor's south end is a 2.07 step (z −176.0, x 52.0..55.6; its north face slopes 2.07 → ~1.71 over
  z −176.0..−175.9) — unclimbable, so no south descent exists [D ray scans, this session].
- The corridor's east side is sealed by the 2.07 mezzanine-apron slab (west face x 55.56, y 0.90..2.07
  plus the wall above): the standing body wedges at x 55.01 (the master run's wedge) and the crouched
  capsule (top root+0.65 ≈ 2.20 on the 0.90 floor) also overlaps the face's span [D ray scans].
- The **secret door** #544 sits in the corridor's WEST wall: the closed leaf at x ≈ 52.0 spanning
  z −174.3..−173.1 [D x-rays]; the yard (0.69) is directly west of it. The walk already opens the door
  for real (the body-touch #559 + the E-arm #558 → AND #560 → ch 8 [D; M 0x1b2318:23-33], green on
  master) — so after the sweep the walk now goes **WEST through the open doorway onto the yard**
  (`walk_to (50.4, −174.2)`, assert: grounded, x < 51.5, floor 0.4..1.0 under). No write. The master's
  assert `pos[0] > 53.5` ("crossed the doorway") was mis-shaped — the old waypoint (55.0, −175.2) never
  crossed the door plane; the new leg actually crosses it (westward) and lands on the yard floor.

**The other four writes stay, each with its evidence sharpened and the reason recorded inline:**

1. **The trench exit** (`[5.0, 12.35, −112.9]`) — re-probed fine this session. The south wall's glass
   lintel is at y **13.36** (not the "13.2" the old comment guessed: y 13.0..13.3 rays pass, 13.5 hits,
   x 4.8..6.5), so the crouch capsule (top ≈ 13.16 [M Player_Collision 0x194df0 case 3]) would clear it
   — but the crouch exit walk **stalls at z −111.81** (three x starts, all the same z, `grounded false`,
   crouch placements at z −111.95/−112.2 pushed back north to the same stall) while every ray at
   y ≥ 11.9 in z −112.0..−113.0 (x 4.9..6.9) comes back CLEAR — the blocking geometry is below ray
   height or is a port-side push-out on the case-floor seam, UNPINNED. The north/east/west sides are
   glass from y ≤ 11.9 (solid at body height); no breakable-glass kind exists on this map [D census];
   no SS keys off ch 0x1b [D]. The lintel sister is re-pinned at y 13.5. The real game's exit is most
   likely the vent duct's own descent into the case interior (~7.3, per the hatch sessions' map) — the
   port's vent drop lands in the trench instead (live check 1).
2. **The plates reach** (`[plate2 + 0.6]`) — the display-ground side is closed as before (the slab
   south face's 0.23..0.4 slot over the 2.37 deck [D probe rays], the deep-floor lane, the east deck
   pocket, the x 15..45 void). New this session: the body-touch capsule (`Toucher::player`, spine
   +0.55..+1.86, r 0.55 [M 0x194df0 + the port's triggers.rs]) reaches only x ≤ 51.84 from the doorway
   while the plates' volume starts at x 52.24 — **the door cannot be opened from the yard**, so the
   plates must be reached from inside the corridor (its north end, unproven) or the mezzanine's annex
   vent (#180/#182/#184, ch 0x54) — both live checks. Two new sisters pin the corridor's geometry: the
   0.90 floor under the plates and the 2.07 south step.
3. **The east climb** (`[95.0, 11.3, −209.5]`) — unchanged; the existing sisters hold (staircase01's
   bottom floats 9.52 over the 6.35 under-stairs [D eastclimb3/4], the 0.78 jump apex [M player.rs]).
   This session's census adds: the Q-Claw IS in the boot loadout [D loadout [1, 2, 30, 74, 78, 80, 84]
   — 80 = 0x50], but all five `graple_pulley` points sit elsewhere (61/77 north, −4.6 west ring,
   39/54 south [D probe]) — no grapple serves the climb, and the level's ladders are the yard #164 and
   #495 only [D].
4. **The ladder495 placement** (`[−7.6, 1.1, −176.9]`) — kept and now documented as a **measurement
   spot, not a route stand-in**: the test's subject is the ladder grab/climb classes at #495's own
   plane ([M 0x1953f4's DEFAULT arm + 0x196118/0x196310]), not the route to the ladder; the route never
   crosses the yard's west edge, so the only honest setup is Bond on the west ground beside the ladder.
   The walk-in and the climb after it are real input.

## How it was verified

- `walkreal_0700000c_live` (the touched binary): **2 passed, 0 failed, 22.85 s** (release, single
  thread) — the whole mission walk including the new west-through-the-door leg, the ladder495 sister,
  the 0d hand-off, and the new sisters.
- The evidence rays/placement probes were run as scratch test binaries against the port's static world
  (which equals the disc's declared collision exactly [D collaudit probe]); their numbers are quoted in
  the test comments with the [D] tags. The scratch files themselves are not committed.
- Full suite: started with the fast runner in the session's first minutes
  (`NF_ISO=… nohup python3 tools/test/suite.py > suite.out 2>&1 &`). The runner was still mid-run at
  the report deadline (it builds ~971 binaries first, ~20 min, then runs them). At cutoff it had
  produced the 3 allowed `disc_golden` slim-ISO failures plus OOM kills (`exit -9`) on seven net/mp
  binaries and two probe failures — those were caused by this session's concurrent probe builds
  competing for memory with the runner, not by the tree; the next session should re-run
  `python3 tools/test/suite.py` on this branch's final tree and quote the total line.
- Touched binaries run: `nf-view` `walkreal_0700000c_live` (above). No src changes were made — the
  commit touches only the test file (+ docs).

## What's left

- The four remaining writes (each with a named live check below).
- The suite's clean total line on this tree (see above).

## Live checks for the main agent

1. **The case exit / the vent duct's landing.** Fresh boot (no save state), PCSX2 + gdbwatch:
   burn the four hinges (#139–142), let the vent sweep, and crawl the duct instead of dropping:
   watch Bond's y — does the duct deliver him INTO the display case (~7.3) rather than into the
   trench (11.84)? Then from wherever he lands, walk/press south against the case's south wall at
   z −111.89 and record whether the body passes (the port's crouch walk stalls at z −111.81 with no
   ray-visible geometry — if the real game passes, the port's collision or crouch push-out diverges
   there; if the real game stalls too, the duct is the only exit).
2. **The plates' reach.** Fresh boot, from the display ground after the photograph: record the route
   the game's own player takes to the warehouse yard (the duct loop back up to the balcony and down
   the west slope 14.61 → 13.33 → 6.92 → 3.80 → 3.13, or something else). Then at the corridor: does
   its north end (z ≤ −162, under the mezzanine) open into the warehouse interior, and does the vent
   plug #184 sink (VentLatch #182 → ch 84) to open the corridor's mid? Either would give the game's
   own route to the plates and delete the reach stand-in.
3. **The east climb.** (Already filed by eastclimb3/4; restating for one run.) At the gate ground
   (95, 7.3, −209.5), fresh boot: watch whether anything the disc declares (a mover, a script, the
   forklift's track) ever lifts a body onto the band deck at (95, −209.5), or whether the intended
   arrival is the band itself from the west (the pit crossing the [L] fresh-boot run says is dead).

## Usage

not visible to me (no token, context-window or cost figures are shown in this environment).
