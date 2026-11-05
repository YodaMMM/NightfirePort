# fx-muzzle-repin-3 — the last muzzle reds

Continue from s/fx-muzzle-repin-2 and read its report. Merge master first; the voice draws are landing, which moves the stream again. Red: fix_npc_doors_live, walkreal_07000011_live real-damage walks, walkreal_07000014_live unmeasured (run it in parts). Remove or reorder no draw. Report each binary.

---
claimed-by: sandbox1 at 2026-11-05T13:49Z

---
## Report (done, 2026-11-05T15:05Z)


---
## Report from the worker (done, 2026-11-05T15:05Z)

# fx-muzzle-repin-3 — the last muzzle reds: all three green, zero code changes

Status (2026-11-05, window closed ~15:05 UTC): **all three named reds are green on the shifted
stream, with ZERO code changes this session — no draw removed or reordered, nothing under
`crates/*/src/` touched, no shared test helper touched.** Continued from s/fx-muzzle-repin-2:
the branch is cut from origin/master 94b312675, which already contains the audit-voice-triggers-4
voice draws (the stream move the -2 task said was landing), so `git merge origin/master` reports
`Already up to date.` by construction. The only walker.rs change since -2 is master's own opt-in
`shield_bloom` flag (default false; only the 01 escort test sets it) — lesson 7 satisfied by the
merge itself, and every walkreal binary below was run after it.

## The three named binaries (each run whole, quoted)

1. **fix_npc_doors_live — was 0/1 (the escort stall at (4.56, 6.34, 6.31)): now 1 passed /
   0 failed, 6.63 s.** The voice-draw stream shift un-stalled the escort lead; the -2 next step
   (the wedged-lead ladder, the lead resumes out-of-way band [M 0x15ea28 case 3]) was not needed.
2. **walkreal_07000011_live — was 1/2/1 (both real-damage walks wedge in the descent): now
   3 passed / 0 failed / 1 ignored, 49.13 s.** `walk_spawn_to_the_lower_floor_load_point`,
   `_real_damage` and `_real_damage_agent` all pass; the ignored test is the documented
   pre-existing 00agent ignore (round 2 levers in its message), not a failure. The descent wedge
   (stuck y 24.0 at (15.79, 24.01, −2.67)) and the point-blank chewing (#14 at 0.7 u) both
   resolved with the stream; the -2 next steps (the head-hunting fight-back lever, the
   arrival-spot probe) were not needed.
3. **walkreal_07000014_live — was unmeasured (the binary exceeded the 400 s tool cap): now
   measured in parts, 9 passed / 0 failed / 1 ignored:**
   - the six non-infiltration tests (e_use switch cascade, east duct, skipped cutscene, vent
     drop, vent grill, tripbomb route): **6 passed, 73.91 s**;
   - `walk_the_infiltration_to_the_delta_exit`: **passed, 25.74 s**;
   - `_real_damage`: **passed, 16.42 s**;
   - `_real_damage_agent`: **passed, 34.38 s**;
   - `_real_damage_00agent`: **ignored** — the pre-existing documented wedge ignore (the '1.72
     landing' resupply leg floats Bond 0.41 over the floor at (33.84, 0.82, 30.95) at tick
     26340, 40 fights / 22 hits for 50.0 by then; the levers — a d3-gated re-ground, a direct
     walk_to the drop's cell, the stairwell AIM-pair stands — are named in the ignore message).
     Not a failure, and unchanged by this session.

## Lesson-10 owed binaries (drone.rs lineage in the merged tree)

- **npc_doors_live: 6 passed / 0 failed / 1 ignored, 27.08 s.**
- **fix_guard_misc_live: 6 passed / 0 failed, 7.69 s.**

## Every walkreal_*_live level-walk binary (the -2 report's not-run ask — all green)

| binary | result | time |
|---|---|---|
| walkreal_07000001_live | 8 passed / 0 failed | 73.39 s |
| walkreal_07000002_live | 3 passed / 0 failed / 1 ignored | 81.72 s |
| walkreal_07000003_live | 6 passed / 0 failed / 1 ignored | 417.91 s (whole) |
| walkreal_07000004_live | 10 passed / 0 failed | 13.04 s |
| walkreal_07000005_live | 14 passed / 0 failed | 73.73 s |
| walkreal_07000006_live | 6 passed / 0 failed / 1 ignored | 34.83 s |
| walkreal_07000007_live | 5 passed / 0 failed | 17.63 s |
| walkreal_07000008_live | 6 passed / 0 failed | 144.30 s |
| walkreal_07000009_live | 8 passed / 0 failed | 179.22 s |
| walkreal_0700000a_live | 13 passed / 0 failed / 2 ignored | 49.72 s |
| walkreal_0700000b_live | 7 passed / 0 failed | 40.17 s |
| walkreal_0700000c_live | 3 passed / 0 failed / 1 ignored | 33.51 s |
| walkreal_0700000c_barrelrow_live | 1 passed / 0 failed | 7.65 s |
| walkreal_0700000c_eastroute_live | 1 passed / 0 failed | 11.16 s |
| walkreal_0700000d_live | 1 passed / 0 failed / 2 ignored | 18.46 s |
| walkreal_07000011_live | 3 passed / 0 failed / 1 ignored | 49.13 s |
| walkreal_07000011_route_live | 1 passed / 0 failed | 6.72 s |
| walkreal_07000012_live | 3 passed / 0 failed | 24.54 s |
| walkreal_07000013_live | 3 passed / 0 failed | 35.82 s |
| walkreal_07000014_live | 9 passed / 0 failed / 1 ignored (parts, above) | ≈150 s |
| walkreal_07000015_live | 6 passed / 0 failed / 1 ignored | 31.77 s |
| walkreal_07000016_live | 14 passed / 0 failed / 2 ignored | 75.94 s |
| walkreal_0700001b_live | 17 passed / 0 failed / 2 ignored | 118.47 s |
| walkreal_07000041_live | 15 passed / 0 failed | 25.35 s |
| walkreal_0700004a_live | 4 passed / 0 failed | 24.53 s |
| walkreal_0700004a_satchel_live | 2 passed / 0 failed | 8.89 s |

## The -2 lineage greens hold on the shifted stream

- **smg_close_range_live: 1 passed / 0 failed, 40.30 s** (the SMG legs stay expected-exact).
- **fix_pickups_trace_live: 2 passed / 0 failed, 2.03 s** (the Raptor drop ledger holds).
- **start_corridor_guard_07000011_live: 8 passed / 0 failed, 14.40 s** (the d2 re-pin holds).

## Determinism evidence for "no draw touched"

- **cross_process_determinism: 3 passed / 0 failed, 6.73 s.**
- **fps_determinism_live: 4 passed / 0 failed, 2.36 s.**
- **split_runs_identity_live: 1 passed / 0 failed, 0.01 s.**

## The non-view crates (run whole)

- **nf-game: src/lib.rs 678 passed / 0 failed / 5 ignored; every test binary 0 failed** — all
  ~105 binaries green, including ai_live 37/0, mp_match_rules 18/0, voice_triggers_live 2/0
  (the audit-voice-triggers-4 required binary), rng_live 1/0, stuck_guards_sweep 1/0 (7.41 s).
- **nf-formats: src/lib.rs 92/0; beam_satchel_probe 1/0; bullet_range_census 1/0;
  calltree_census 1/0; carmodel_golden 2/0; disc_golden 11 passed / 3 failed** — the three
  allowed slim-ISO failures (`bigf_header_fields_behave_as_documented`,
  `bigf_tables_match_the_golden_manifest`, `directory_tree_matches_the_golden_manifest`;
  `/DRIVING/MIS01.MUS: too small for a BIGF header` — the slim ISO strips the audio streams).
- **nf-audio: src/lib.rs 24/0; bank_live 1/0; fix_sound_remainder_live 4/0; music_live 1/0.**

## What did NOT run

The nf-view crate has ~800 test binaries. Building all of them in one
`cargo test -p nf-view --no-run` exceeds the sandbox's 10-minute single-call cap, and background
processes are killed between tool calls (a 13-minute background attempt emitted 0 links before
being killed). Everything the task named, everything the -2 report owed, every walkreal level
walk, the -2 lineage greens, the determinism core and the three non-view crates ran whole and
are quoted above. The remaining nf-view census/probe/fix binaries did not run this window; since
this session changed no code, no lesson-10 grep obligation exists beyond the binaries above.

## Live checks for the main agent (carried from the -2 report, still unanswered)

1. **The d2 idle-death question (start_corridor_guard_07000011)**: in PCSX2, load 07000011 at
   d2, leave Bond idle at the spawn (−0.40, 25.83) — does the game's idle Bond still die
   (~f1113 per [L docs/live/2026-11-01-07000011-start-corridor-guards.md])? The port's corrected
   stream has him surviving 3000 ticks with the exact 12-per-hit damage landing.
2. **The Raptor drop landing (fix_pickups_trace)**: gdbwatch the drop's fall for a guard killed
   in the west area near (−11.9, 2.9) — the port's drop rests at y 5.34, under the walking floor
   (~6.3); watch whether the game's own drop-fall (`Pickup_Update`) also clips the sloped tri
   there or rests it ON the floor.

## How this session was verified

Every claim is a quoted `test result:` line from a run on this branch (master 94b312675 + the
WORKLOG commits) with `NF_ISO` set and `--test-threads=1`, executed between 13:56 and 15:02 UTC.
The three named reds were run BEFORE any other work, exactly as reported; the rest of the ladder
followed. Nothing was fixed because nothing was broken on this stream; the -2 reds were stream
realizations, and the stream moved again — forward.

## Usage

not visible to me
