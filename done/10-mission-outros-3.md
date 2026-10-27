## Your task: `mission-outros-3`, the outro tests pass under cargo test but fail under the suite runner

Start from `origin/s/mission-outros-2` (merge it, then origin/master). With the full disc image, `cargo test -p nf-view --test mission_outros_live` passes, but the same binary run by `tools/test/suite.py` (each binary alone, NF_SILENT=1, its own working directory/env, a wall-clock limit) fails both tests:
- the_double_cross_outro_runs_its_subtitle_script, the_phoenix_fire_outro_runs_its_subtitle_script ("the outro stayed up through the track's entries ... got 1")

Find what differs between the two runs that makes the hold end after 1 tick (NF_SILENT muting the audio sink that the movie length or the player reads? the cwd for the PSS path? a thread/decoder start under the runner?), and make the hold depend only on the game's rule (the movie's real length [M psiMovieFinished 0x1082e8]), independent of the audio mute and the process environment. Run the binary through `python tools/test/suite.py <repo> <target> --only mission_outros_live` with NF_SILENT=1 before reporting, then the full suite.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mission-outros-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox6 at 2026-10-26T05:36Z

---
## Report (done, 2026-10-26T07:01Z)


---
## Report from the worker (done, 2026-10-26T07:01Z)

# mission-outros-3 — the outro tests pass under cargo test but fail under the suite runner

## What was wrong (root cause)

The failure is the **tree's hold rule, not the runner's environment**. The two outro tests
exist only on the mission-outros branches — `origin/master` has neither
`mission_outros_live.rs` nor the fix [M: `git show origin/master:crates/nf-view/tests/
mission_outros_live.rs` → "not in 'origin/master'"; only `s/mission-outros` and
`s/mission-outros-2` contain the test file]. The tree the failing suite ran carried the tests
WITHOUT the mission-outros-2 fix: `origin/s/mission-outros`'s `Player::open` in the no-`movies`
build (the suite and plain `cargo test` both build without the opt-in `movies` feature) returns
`Ok` with **`ended: true`** for a file that **opened fine** [M origin/s/mission-outros
crates/nf-view/src/movie.rs:234].

With the retail movies present the start therefore SUCCEEDS [M psiStartBackgroundMovie
0x1081a8's strFileOpen arm: BGFMVPlaying = 1], the state-6 glue reports the movie as playing
for exactly one poll, the next poll reads `finished()` (`ended`) and state 6 pops —
`movie_ticks == 1`, and both tests die on "the outro stayed up through the track's entries ...
got 1". That contradicts the game: BGFMVPlaying is cleared only when the decode pump finishes
the file [M 0x108060] and `psiMovieFinished` is `BGFMVPlaying == 0` [M 0x1082e8]; only a
DECLINED open (not whitelisted, file absent, 0-byte stub) pops at once [M 0x1081a8's
strFileOpen failure arm].

None of the runner's differences is in the hold path at all:

- **NF_SILENT** only gates `nf_audio::Output::open` [M nf-audio lib.rs:339] and
  `open_pcm_out` [M movie.rs:492] — neither runs in the no-`movies` hold path (the App's audio
  sink is never opened by the flow tests).
- **The cwd** is never read: every file read in the hold path goes through `NF_ISO`
  (`Player::start` → `IsoImage::open(iso)`; the subtitle track and the tests likewise).
- **Threads**: the no-`movies` player starts none (the decoder worker is `cfg(feature =
  "movies")`); the suite's per-binary run starts nothing extra.
- **Wall clock**: the hold's `headless_advance` is a tick-driven countdown with no `Instant`.

## The fix

Already landed by mission-outros-2 and carried here (branch start: `s/mission-outros-3` =
`origin/s/mission-outros-2` + `origin/master`, merge clean; master re-merged at 2757d099): the
opened-but-undecodable movie holds for the movie's **real length in 60 Hz ticks** — the PSS's
audio-track duration [M 0x1081a8 BGFMVPlaying = 1 on the successful open; 0x108060's
end-of-file clear; 0x1082e8; D golden/pss_movies.tsv: 07300004.PSS = 2 384 128 PCM samples at
48 kHz = 2980 ticks; 07300013.PSS = 2 040 064 = 2550] — advanced once per host BGFMVPlaying
poll (`headless_advance`), independent of NF_SILENT, the process environment and the cwd. A
DECLINED start still reports finished immediately — the game's un-started movie. **No further
code change was needed**: "the hold depends only on the game's rule" is the state of this
branch, and this task's job was to prove it on the real-movie path and pin the difference.

## Proof on the real-movie path

This sandbox's image stubs every `/MOVIES/30_FPS/*.PSS` (size 0 [D — the ISO tree]), so the
real-movie branch was made executable locally:

- `scripts/inject_movies.py` (session script, not committed) builds minimal valid PSS files
  whose PCM audio tracks carry the **retail durations** [D golden/pss_movies.tsv — 07300004 =
  2 384 128 samples = 2980 ticks; 07300013 = 2 040 064 = 2550 ticks], appends them to an ISO
  copy and rewrites their ISO9660 directory records (extent + size, both-endian) →
  `Nightfire-movies-test.iso` (everything else identical; verified by the same walk).
- **A/B on the SAME injected image, the SAME runner shape (NF_SILENT=1, per-package cwd):**
  - old tree (`origin/s/mission-outros`, its test binary rebuilt from a worktree): **1 passed,
    2 FAILED** — `the outro stayed up through the track's entries [S the hold is the port's
    stand-in; ...], got 1` for BOTH outro tests — the driver's exact symptom, reproduced on
    demand;
  - this branch: **3 passed, 0 failed**, and the direct binary run prints no "is a stub" line —
    the `Some(pss)` branch ran, i.e. the hold matched the injected PSS's real length (2980 /
    2550, ± 2) [M 0x1081a8 + 0x108060 + 0x1082e8; D the injected track's sample count].
- **Both invocation shapes on the injected image**: `python tools/test/suite.py <repo>
  <target> --only mission_outros_live` with NF_SILENT=1 → **3 passed, 0 failed** (twice: before
  and after the final master merge, 818-binary build); `cargo test --offline --release
  -p nf-view --test mission_outros_live -- --test-threads=1` without NF_SILENT → **3 passed,
  0 failed**. The subtitle entries printed beside the hold (0600095d: 8 entries, first
  "The code key you found in my safe..." at t249 [D]; 06000968: 4 entries at t468 [D]).

## What was run (final tree, before the final push)

- **Full suite** (s/mission-outros-3 with origin/master 2757d099 merged; 818 test binaries;
  standard image; NF_SILENT=1; `tools/test/chunk_suite.py` — this sandbox kills background
  jobs between tool calls, so the resumable chunk runner replaced the nohup form, and the
  2-hour window cut it): **81/818 binaries recorded — 401 passed, 0 failed, 11 ignored**;
  zero FAILED/TIMEOUT lines. 737 binaries had no window left.
- **Targeted sweep** of the 11 outro/movie-adjacent binaries (`suite.py --only
  mission_outros_live,reward_after_outro,movie_playback_flow,movie_trigger_live,
  movie_pacing_live,movie_stream_bounded_live,briefing_plays_live,front_end_chain_live,
  gameflow_stack_sp_run,menu_audio_flows,cutscene_sync_live`, standard image): **34 passed,
  0 failed**.
- The known slim-image allowances (`identity_disc`, 3 × `disc_golden`) sit in binaries outside
  the recorded window; they are image-calibration failures unrelated to this task (the same
  ones the previous full run showed on the same image).

## What's left

- The remaining ~737 suite binaries (window; the resumable state is sandbox-local:
  `suite_mo3_state.json` next to the repo — re-running `tools/test/chunk_suite.py` continues
  from it).
- On the real-disc machine the same two commands should hold ≈ 2980 / 2550 (± 2) with no
  "is a stub" eprintln — the asserts now measure the image's own PSS lengths, so no further
  change is expected there.

## Live checks for the main agent

1. `NF_ISO=<retail image> python tools/test/suite.py <repo> <target> --only mission_outros_live`
   → expect `3 passed, 0 failed`; the holds run the retail PSS's own audio-track durations.
2. PCSX2 **fresh boot** → Double Cross last part → the kill: the outro movie runs its full
   ~49.7 s (2980 ticks) with the subtitles beside it, THEN the results page. A save-state
   resume from mid-outro skips the boot path and is not a valid measurement.

## Usage

Not visible to me — this session's environment shows no token, credit, context-window or cost
figures anywhere I can read, so there are no numbers to copy.
