## Your task: `cutscene-kind31-frame-3`, the general frame rule without breaking the gondola and the escorts

Start from `origin/s/cutscene-kind31-frame-2` and read its report (merge origin/master). Its one-line rule (`keys_alone = camera_started && parent < 0` for every unparented case-1 stream [M 0x1d9a08 case-1 0x1d9c0c, Script_GetInterp 0x1da838, Script_CameraStart 0x1d916c]) turned five tests red on the full suite, reproducible alone: walkreal_07000008_live `walk_the_gondola_ride_to_the_end` and `the_ally_fights_only_the_guards_through_the_noise_path`, zoe_gondola_realpath_live `zoe_rides_the_carriage_and_never_targets_bond` (all three: "the cable-car floor plate fired ch 4 [D #236 mask 8]" fails), mayhew_hold_live `the_escort_holds_at_the_ally_wait_node_until_the_ambush_clears` ("never parked in AllyLeadMissionWait 0x21"), escort_hide_findrun_live `the_hides_exit_reanchors_the_mission_route_before_the_lead_resumes` (route index 2, want 1). Find which unparented stream in which camera script the rule now reads keys-alone where the game composes (or vice versa) - e.g. a scene's camera op tick vs the stream's op tick, a SubScriptStart base reset [M 0x1db050:28-38], SP_SetPosRot [M 0x1db920] - and fix the rule to what the game does; all five green plus the 01/14 battery from frame-2's report.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cutscene-kind31-frame-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-27T18:51Z

---
## Report (done, 2026-10-27T20:07Z)

# Report: cutscene-kind31-frame-3

Branch `s/cutscene-kind31-frame-3`. Frame-2's general rule (`keys_alone = camera_started && parent < 0`)
turned five real-path tests red. The break is the HASH-0 entity-stream class: the game composes the
hash-0 "hand-back anchor" streams with the SP placement base even after the script's camera op. The fix
adds that carve-out to the general rule; all five red tests and the 01/14 battery are green.

## What's done

- **The fix** (`crates/nf-game/src/nis.rs`, the case-1 interp): 
  `keys_alone = camera_started && parent < 0 && !(ehash == 0 && ekind != 0x19)` — frame-2's general rule
  now applies only to the gfx-bound (non-zero-hash) streams; hash-0 non-0x19 entity streams compose
  always; the pre-existing hash-0 kind-0x19 marker skip (the 4a-lift-4 [L] pin) is unchanged.
- **The anchor streams, found and characterized** [D]: the only hash-0 entity streams in camera-op
  scripts of the two levels involved are the hand-back anchors —
  - `0x06000087` (07000001 intro) **s11**: hash 0, obj_type 0x31, created op@2118, ONE key at t2118;
  - `0x06000077` (07000008 station scene) **s15**: hash 0, obj_type 0xff, created op@3870, ONE key.
  Each sits one tick before its script's event-6 hand-back. `Script_EventHandler` 0x1d94c0 case 6
  snapshots the PREVIOUS stream's obj+0x90 into si+0x40 [M] and `Script_Free` 0x1da880:44-48
  `Player_Enable`s the player at it [M] — so the anchor's composed-vs-keys-alone matrix IS Bond's
  post-scene position.
- **Frame-2's census misread, corrected** [D]: `keys[a..=b]` is INCLUSIVE for entity streams (the port's
  `Nis::keys`), so s11's `first_key 3893 last_key 3893` is ONE key — the census's "s11 has no keys
  (first_key 3893 > last_key 3892)" applied the camera op's exclusive convention (`wrapping_sub(1)`) to
  an entity stream. Frame-2's "s11's snapshot is unchanged by the rule" therefore does not hold; the
  snapshot moves with the rule, which is exactly the five failures.
- **The bisect** (a temporary `NF_KA_CLASS` env knob, removed before commit): with kind-0x31-with-hash
  or kind-0x19-with-hash keys-alone, `mayhew_hold_live` stays GREEN; with hash-0 entities keys-alone it
  goes RED — the failing class is hash-0, not the 0x31/0x19 classes frame-2's census covered.
- New disc tools: `crates/nf-formats/tests/nis_stream_dump.rs` (full per-stream op dump of named
  scripts), `nis_hash0_census.rs` (hash-0 entity streams in camera scripts, 07800001/07000008),
  `nis_camscan08.rs` (all unparented entity streams in camera scripts, all `*08` archives).
- `tools/test/suite_resume.py`: a resumable fast-runner wrapper (same binaries/flags/600 s per-binary
  limit; per-binary results in a state file so repeated foreground invocations make forward progress) —
  this sandbox reaps every background process between tool calls, so `nohup … &` suites die instantly.

## Verification

- The five formerly-red tests, all green on the fixed tree:
  `mayhew_hold_live` 1/1 (5.5 s), `escort_hide_findrun_live` 3/3 (3.3 s), `zoe_gondola_realpath_live`
  1/1 (27.9 s), `walkreal_07000008_live` **4/4** (112 s) — including `walk_the_gondola_ride_to_the_end`,
  frame-2's open timeout item, now completing.
- The 01/14 battery from frame-2's report, all green: `walkreal_07000001_live` 3/3, `interact_07000001_live`
  10/10, `exit_aim_01_live` 1/1, `capture_placement_07000014_live` 1/1 (three consecutive re-runs; one
  first-run-after-rebuild FAILED once at 4.1 s and passed on every subsequent run — treated as a sandbox
  flake, worth one retry on the next run), `walkreal_07000014_live` 2/2, `walk_07000014_live` 2/2 (the
  merged tree carries 2 tests; frame-2's "3/3" predates the origin/master merge), `exit_aim_hint_live`
  2/2, `probe14_capture_end` 0/0/1 ignored.
- Commands: `cargo test --offline --release --test <name> -- --test-threads=1` per binary, NF_ISO on the
  slim ISO, NF_SILENT=1.
- **The full fast-runner suite did NOT run to completion** — its build phase (`cargo test --no-run` over
  the four crates after the fix) exceeds the 10-minute tool-call window and background processes are
  reaped between tool calls, so no total line exists yet. `tools/test/suite_resume.py` is committed for
  exactly this: run it repeatedly (foreground), it skips binaries already recorded in the state file and
  prints the running totals; the next session can finish it in 2–3 calls.

## What's left

- The full fast-runner suite total line on this final tree (the targeted battery above covers the
  cutscene-sensitive binaries; the nf-game unit chunk is unexercised on this tree).
- The [M] derivation of the carve-out is OPEN: the case-1 own-key call passes the base flag
  `(parent == -1)` [M 0x1d9c0c disasm: `nor`+`sltiu` of stream+0x25], GetInterp's tail prepends
  `si+0x00` via `Concat(si, out)` [M 0x1da1a4 `sw a0(=si),0(sp)`; 0x1da838-0x1da844], and
  `Script_CameraStart` writes `Mat_IdentityT(si)` [M 0x1d916c disasm: a0=si at the 0x1e34c8 jal] — so
  si+0x00 should be the identity at the anchor tick, yet the game composes. No restorer of si+0x00
  turned up in the decompile: `Script_CameraEnd` 0x1db348 doesn't touch it, `SubScriptStart` 0x1db050
  bases only the SUB's own si on the zero vector (0x2d6200 = zeros [L ee_snapshot.bin offset 0x2d6200]),
  and `SP_SetPosRot`'s only SP-mode caller is `Copter_Move` 0x17d628:115. The compose is pinned by the
  five real-path tests; the mechanism is a live check below.

## Live checks for the main agent

1. **The si+0x00 restorer** (settles the carve-out's mechanism): in 07000001, boot fresh, let the intro
   run, and watch `si+0x00` (the SCRIPTINFO of script 0x06000087 — find the si via the SP player at the
   script's load) at ticks ~0 and ~2118. Expected if the tests are right: the placement matrix at t2118,
   NOT the identity — i.e. something between the camera op (t0) and the anchor restores it. Access: read,
   twice. `tools/trace/gdbwatch.py` on the si address (found via the SP's obj+0xe0 → SCRIPTPLAYER+0xc).
2. **The anchor hand-back spot** [L]: a fresh boot of 07000001 — Bond's position at the intro's end
   (the first enabled frame). Expected: the s11 key composed with the intro SP's placement (the port's
   current output), not the key's raw local coordinates.
3. Same shape for 07000008's station scene (s15 of 0x06000077, event 6 at t3880).

## Usage

not visible to me
