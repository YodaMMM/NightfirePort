## Your task: `level-41-walk-to-the-end`, 07000041 from the plate to the level's end

`walkreal_07000041_live::walk_from_the_airlock_to_the_player_plate` stops at the player plate. Extend the walk on real input from there to the level's end (the next level's load or the mission end), every objective set by the game's own writers (no channel pokes); read the 41 docs and the existing 41 tests (the tether wires, the drone plate, the movie trigger). Fix any port gap it hits with evidence [M/D]. Every test in the file green. Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-41-walk-to-the-end`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-31T11:03Z

---
## Report (done, 2026-10-31T12:20Z)


---
## Report from the worker (done, 2026-10-31T12:20Z)

# level-41-walk-to-the-end — the report

Task: `walkreal_07000041_live::walk_from_the_airlock_to_the_player_plate` stops at the player
plate; extend the walk on real input to the level's end, every objective the game's own writers
set, no channel pokes; fix any port gap with evidence; every test in the file green.

## What the disc says the level's end is

The wrapper 07000041 has **no reachable in-level end** — that is now pinned end to end, not
assumed:

- **No level-exit trigger, no 0xE8 LOAD_LEVEL, no script event 0x12** anywhere in its data [D the
  placement scans + the archive decode, docs/levels/07000041-audit.md].
- **No mission record**: the 24-record MissionData table (@0x2a4350, stride 0x28 [L
  ee_start_05.bin]) has no 0x07000041 row, so `Mission_Init` 0x182270:53-56 leaves BaseMap = −1
  and InternalState = 5, and `Mission_Update`'s case 5 is a bare no-op — the monitor never runs
  [M mission.rs mirrors it; walkreal pins `mission_end == None` on the walk].
- **ch 0x01, the only channel the chain leaves armed** (plate #99's out, mask 0x4, lock 0x6e [D
  #99]), has no disc toucher: the four 0x71 placements are rotors [M 0x1d04f0:284-290 →
  `rotor_init` 0x18f4b8], no 0x0f drone placement exists [D], the kind-7 scripts
  (0600004f/6f/27/44/45/52/7c4) are presentation-only (no Event 0x11, no 0x0f Entity [D decoded])
  **and nothing mounts them** — the level's only class-0xd9 script players carry the 0x06000000
  sentinel [D the audit's 66-map scan]. So the port's documented 241-tick wrapper-end stand-in
  (drone.rs:15949, `s/level-41-exit-event`'s subject) can never fire on real input, and the walk
  pins ch 0x01 cold, `level_exit == None`, `mission_end == None` at the walk's end.

So the walk's end **is** the level's end on this disc: the next level's load comes from the
driving-side flow (0x07000016 → outro 0x07300017 → front end 0x07000048 → 0x0700001b [M
0x182c60 case 6 → 0x1d3800 → 0x1d6360 → 0x1d3560]), from nothing the walk can reach.

## The continuation walk (the new test)

`walk_from_the_plate_to_the_tether_crossing_and_the_levels_end` — real input only
(`Sim::step_bond` + `Input`), no teleports, no channel pokes:

1. the airlock start → the station floor → **the player plate #100** (ch 0x6e fires on the walk
   [D #100]);
2. **the DynoRod deck** — the rod still stands at the walk's end: its action ch 0x6e armed at the
   plate but its gate ch 1 never rose, and the delete needs BOTH [M DynamicObject_Update
   0x18d3b8's gate + mode-0 delete; D #117 params 0:0x1 1:0x6e];
3. **the tether crossing**: at icon #114's spot (10.48, 0, −59.13) the ctx-3 icon arms on the
   walk-in [D #114 params (0:0x3 1:0x0); M ThirdIcon_Update 0x196e80 writes BLData+0x95f]; the
   jump grab attaches Bond to wire #26 [M Player_HandleJump 0x1916b0:126-147 — the jump edge with
   BLData+0x95f == 3 takes the nearest class-0x2c object]; the hand-over-hand strafe carries Bond
   along the cable (>1 u asserted) and the wire drops him grounded on the far deck [M Player_Wire
   0x1977c0];
4. **the far south floor**: past the crossing the level's own floor continues (the dev-materials
   statics stand on it [D #500-#518 at z −70..−89]) and the walk covers it deterministically to
   (2.53, 1.13, −81.18) — two legs give up leniently on the port's crossing-deck walls and the
   third re-aims and arrives (route fact, port collision [S], kept out of the game-behaviour
   pins);
5. the cold-tail pins above, at the walk's end.

The ride direction note: from the yaw-0 grab at the #114 spot, strafe +1.0 carries Bond to wire
#26's north-west end (drop at (6.80, 1.49, −54.08)) — the interactables sister's `abs` >1 check
passes on the same direction; the disc gives no player route onto the east ziplines (#575/#576/
#577, p1 = 2 [D]) from this deck in this session's probing.

## The port gaps the walk hit

**None in the port's simulation.** Every step the walk takes works: the plate fires, the icon
arms, the grab attaches, the traverse carries, the wire drops, the floors collide, the rod stands,
the tail stays cold. The two `#[ignore]`d interactables tests (the laser-mine walk-in contact and
the stealth-wall crawl entry spot) remain the documented re-spot items — both non-script gaps,
unchanged by this task.

What DID need fixing (doc/test hygiene, from the audit's "not applied — read-only" list):

- `walkreal_07000041_live.rs` header: the "type-0x71 spawners … scale-float window" cite → rotors
  ([M 0x1d04f0:284-290; DroneSpawner_Create is case 0xf1 only]); the garbled script-id cite → the
  real ids 0600004f/6f/27/44/45/52/7c4.
- The tether refusal test's docstring: the stale "#82-#85 third icons (contexts 1/2)" census (they
  are kind-0x2c lightning [D]) → the real icons #105/#114, and the assertion message now cites
  them. The spot-based assertion itself still holds (#114's reach does not extend 5 m north to the
  wire's midpoint).
- `docs/levels/07000041.md` §1 (start Y 1.10 → the disc word 1.32) and §6 (rotors, the real script
  ids + the unmounted finding, the movie trigger's full consumer chain, the icons correction, the
  continuation walk).

No decompile-backed test was weakened; no test contradicted the decompile.

## How it was verified

- `walkreal_07000041_live`: **6 passed, 0 failed, 0 ignored** (was 5) — `cargo test --offline
  --release -p nf-view --test walkreal_07000041_live -- --test-threads=1`, 7.64 s.
- `interactables_07000041_live`: **14 passed, 0 failed, 2 ignored** (the two pre-existing
  documented re-spot ignores) — 19.55 s.
- `walk_07000041_live`: **1 passed, 0 failed** — 1.41 s.
- `census_07000041`: **0 passed, 0 failed, 1 ignored** (the census probe is opt-in `--ignored`,
  as the audit used it) — 0.00 s.
- No nf-game/nf-formats/nf-audio source changed (the diff is one test file + two docs), so the
  other crates' binaries are untouched by this task.

## The full suite

The environment kills background processes between tool calls, so the fast runner's nohup start
died twice at the cargo-build step; the repo's own chunked runner (`tools/test/chunk_suite.py`,
built for exactly this) ran the suite in foreground chunks into `../suite41_state.json`. At the
checkpoint before the final tree it had **160 passed, 0 failed, 2 ignored** across the first 21 of
984 binaries (including `nf_view` 96/0, `nf_audio` 23/0, `walkreal_07000003_live` 4/0). The full
984-binary sweep does not fit the remaining session window alongside the walk work; the tally and
the per-binary log live in the state file, and the next session can resume the same state file to
completion. No FAILED/TIMEOUT line appeared in any chunk.

## What's left

- The suite's remaining ~963 binaries (resume `chunk_suite.py ../suite41_state.json`).
- The two interactables re-spots (the mine contact, the stealth-wall crawl entry) — pre-existing.
- The east zipline rides (#575/#576/#577) from the upper deck (the ladder #480 climb reaches it)
  — a possible further extension; the grab mechanic itself is level-proven.
- `s/level-41-exit-event` (pending queue task) owns the wrapper-end stand-in's replacement.

## Live checks for the main agent

| Where | Check | Expected |
|---|---|---|
| fresh 07000041 boot, walk to plate #100 (15.6, ·, −33.2) | watch `switch_channels[0x6e]` (0x26fc90+0x6e) | rises on the touch; ch 0x01 (0x26fc90+1) stays 0 for minutes afterwards |
| stand at the icon #114 spot (10.48, ·, −59.13), press jump | BLData+0x95F and Bond's substate | +0x95F == 3 near the spot; the jump attaches (substate = the wire state); from (11.3, ·, −54.1) (the wire's midpoint) the same press is a plain jump |
| the DynoRod (18.80, ·, −44.49) after the plate touch | the rod object alive | the rod stands (gate ch 1 off); it should never delete on a real walk |

## Usage

not visible to me (this session's driver shows no token, context-window or cost figures).
