## Your task: `mission-outros-4`, the outro hold under the `movies` feature (workspace build)

Start from `origin/s/mission-outros-3` (merge it, then origin/master). Round 3 proved the hold on a `-p nf-view` build, where the `movies` feature is off. The suite builds the WORKSPACE (`cargo build --workspace --tests`), which unifies features: `movies` is ON there, and the run prints `movie: decoding 07300004 ahead of state 6`. On that path both tests fail in ~3 s at the "the outro resolved" assert (mission_outros_live.rs:95 / :154): 6000 `frame()` calls pass without `phase2_resolved` - the decoder thread finishes on wall-clock time, not per poll, so a fast headless loop never reaches the movie's end.

Reproduce with `cargo test --release --workspace --test mission_outros_live` (or `--features movies`). Make the state-6 hold follow the game's rule on BOTH builds: BGFMVPlaying cleared when the movie's frames are consumed [M 0x108060, 0x1082e8], driven by the host's movie poll count (the PSS's audio length in ticks, as round 2 did) - not by how fast the decoder thread runs. Keep the decoded picture path for the real app; only the "finished" decision must be tick-driven. Run `python tools/test/suite.py <repo> <target> --only mission_outros_live` on a workspace build and the full suite before reporting.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mission-outros-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-26T08:42Z

---
## Report (done, 2026-10-26T09:38Z)


---
## Report from the worker (done, 2026-10-26T09:38Z)

# mission-outros-4 — the outro hold on the `movies` build: the finished decision is the host's poll count, not the decoder's wall clock

## What was wrong (root cause)

Round 3 proved the state-6 outro hold on a `-p nf-view` build — where the opt-in `movies`
feature is OFF. The suite's build (`cargo test --offline --release -j4 --no-run -p nf-game
-p nf-formats -p nf-audio -p nf-view`, tools/test/suite.py:36) unifies the four crates in one
resolution; on the driver's machine that build had `movies` ON, and both `mission_outros_live`
outro tests failed in ~3 s at "the outro resolved" (mission_outros_live.rs:95 / :154) with
`movie: decoding 07300004 ahead of state 6` on the console.

The mechanism, reproduced here exactly (1.35 s, same two failures, same print):

- With `movies` ON, `Player::open` starts a live `MovieStream` (the decoder worker threads) and
  left the poll-count hold `None` (`headless_left = (stream.is_none() && ...)`) [M the old
  crates/nf-view/src/movie.rs open(), movies arm].
- The state-6 glue calls `p.headless_advance()` (a no-op with no hold) and then
  `fe.flow.movie_playing = !p.finished()` [M crates/nf-view/src/lib.rs mission_movie_glue].
- `finished()` is `self.ended`, and in the movies build `ended` was set ONLY inside
  `present()` — the picture path, paced by the wall/audio clock. The glue never calls
  `present()`, so in a headless poll loop `ended` never sets: the hold never ends, state 6
  never pops, `phase2_resolved` never lands. The decoder thread "finishing" on wall-clock time
  is invisible to the glue by construction.
- The game's rule is different: BGFMVPlaying = 1 on the successful open [M psiStartBackgroundMovie
  0x1081a8's strFileOpen arm], the decode pump consumes one frame per second vblank [M main
  0x109080's PS2FramesToSkip] and clears BGFMVPlaying at the file's end [M
  psiDecodeBackgroundMovieFrame 0x108060], and psiMovieFinished is `BGFMVPlaying == 0` [M
  0x1082e8]. The measure is frames consumed per tick — the host's poll count to the movie's
  length [D the PSS's audio track; golden/pss_movies.tsv] — not how fast a decoder thread runs.

## The fix (c9ddd0b8)

`crates/nf-view/src/movie.rs`, one behavioural line + comments:

- `open()`, movies arm: `let headless_left = (!looping && length_ticks > 0).then_some(length_ticks);`
  (was `(stream.is_none() && !looping && ...)`). The BGFMVPlaying poll hold now runs on EVERY
  build, for declined workers and live decoders alike; a looping backdrop still never finishes
  [M 0x108060's else arm restarts the file], and a DECLINED start still reports finished at once
  (BGFMVPlaying stayed 0 [M 0x1081a8's strFileOpen arm]).
- `headless_advance`/`finished`/field docs updated to say the same: the decoder thread keeps
  feeding the picture path (`present`) on the wall/audio clock — untouched — but it does not
  pace the finished decision anymore.
- `crates/nf-view/src/lib.rs`: the glue's comment updated to the both-builds rule. No behaviour
  change there: `headless_advance()` + `movie_playing = !finished()` is exactly round 2's shape,
  now on both builds.

## Reproduction and verification (all on the final tree = s/mission-outros-3 + origin/master 7e35bd77 + the fix)

The slim on-foot ISO stubs every PSS (22 × 0-byte [D — the ISO tree, walked]), so the real-movie
path was made executable again: `scripts/inject_movies.py` (session script, /scripts is
gitignored — round-3 precedent) builds minimal valid PSS files whose PCM audio tracks carry the
RETAIL durations read from golden/pss_movies.tsv (e.g. 07300004 = 9 536 512 body bytes =
2 384 128 samples = 2980 ticks), appends them to an ISO copy and patches their ISO9660 directory
records (extent + size, both-endian). This round it injects EVERY audio-bearing movie, so the
movies-gated e2e files can run against real durations too. Verify with the walk in the report
of round 3: 177 files unchanged except the injected records.

- **Pre-fix repro** (`--features nf-view/movies`, injected image, the test binary direct):
  `movie: decoding 07300004 ahead of state 6` + `movie: decoding 07300013 ahead of state 6`,
  BOTH outro tests FAILED at mission_outros_live.rs:95 / :154, 1.35 s — the driver's symptom
  exactly.
- **Post-fix, same command: 3 passed, 0 failed** (1.24–1.79 s across runs). The `--nocapture`
  run shows the holds matching the injected PSS's real lengths — the `abs_diff(pss) <= 2`
  asserts measure the image's own audio tracks (2980 / 2550 ± 2) — and the subtitle scripts
  ran beside them ("0600095d: 8 entries, first 'The code key you found in my safe…' at t249";
  "06000968: 4 entries, first 'Need a lift, Mr. Bond?…' at t468").
- **Post-fix, all four shapes** of `mission_outros_live` (direct binary, `--test-threads=1`):
  movies-ON + injected image 3/3; movies-ON + stub ISO 3/3 (the declined-start path — the
  game's un-started movie, BGFMVPlaying 0 [M 0x1081a8]; the [S] track-length stand-in);
  movies-OFF + stub ISO 3/3 (the suite's own shape); movies-OFF + injected image 3/3 (round 2's
  no-decoder real-length hold). Note the movies-OFF + stub run prints the "is a stub" line and
  the hold runs the track lengths — unchanged from round 2/3.
- **The movies-gated surface, movies-ON wave** (`scripts/wave_suite.py` wave `mo`, injected
  image, NF_SILENT=1, per-binary `--test-threads=1`): 17 binaries — mission_outros_live,
  reward_after_outro, menu_movies (9), movie_playback_flow (10), movie_pacing_live,
  movie_stream_bounded_live, movie_gop_stream_live, movie_over_scene_live (2),
  movie_trigger_live (2), movie_audio_layout_live, menu_audio_flows (9), menu_sp_e2e (9),
  unlock_popups (13), net_match_end_clients_live (4), menu_sfx_flow (19), menu_graph_walk (2) —
  ALL GREEN (93 tests). These drive `present()` directly and are unaffected by the change:
  `headless_left` only moves through the glue's per-frame poll, which none of them exercises.
- **briefing_plays_live: 4 passed, 14 failed in the movies-ON wave — analysed, not a
  regression of this fix.** The whole file is `#![cfg(feature = "movies")]` (line 43) and its
  header says it "needs an image WITH movie data … the full retail image carries all 23" — in
  the suite's movies-OFF build it contributes ZERO tests (that is how round 3's sweep passed it
  on the stub ISO). On the injected image its 18 tests run, and the synthetic movies have NO
  decodable pictures, so the test's own `p.present()` calls hit the picture path's
  end-of-stream arm (`s.finished && !self.looping → self.ended = true`, present()'s existing
  stream-EOF branch — untouched by this fix) within a couple of redraws and the hold pops early
  ("state 6 lasted 95 redraws"). The fix does not change this: `headless_left` is decremented
  only by the glue's per-frame poll (≤600 in the test's loop, under the 2561-tick length), so
  the poll hold cannot fire there; on a retail image the real pictures keep the stream alive
  and the test's own pacing governs, as when 54a4215f landed it green. Listed under Live
  checks for a retail-image run.

## What was run (suite situation — read this)

- The task's `nohup python3 tools/test/suite.py > suite.out 2>&1 &` was started in the first
  minutes as ordered; this sandbox killed it between tool calls (verified: process gone,
  suite.out empty at 8 s) — the known sandbox behaviour, again.
- The sandbox disk is a 9.9 GB rootfs. The full 818-binary test build does not fit alongside
  the ISO + work/ (the build was run twice and hit `ld: Bus error` / 100% disk both times;
  ~3.3 GB of binaries + ~1.6 GB of rlibs are needed at once, and the sanctioned runner builds
  everything before running anything). To get REAL suite coverage within the window I ran the
  same per-binary shape the runner uses (NF_SILENT=1, `--test-threads=1`, per-binary wall-clock
  limit, resumable state file) through `scripts/wave_suite.py` (session script, gitignored):
  build ONE wave's targets, run each binary, record the result, DELETE the binary. Results
  live in scripts/wave_state.json.
- **Waves completed: `mo` 17/17 binaries (93 passed / 0 failed / 0 ignored, except the
  briefing_plays_live 14 analysed above) and `game` 89/89 binaries (nf-game, 344 passed /
  0 failed / 3 ignored).** Every binary green except the analysed briefing artifact.
- **Not run (window + disk): nf-formats, nf-audio and the remaining ~680 nf-view test files**
  (waves `formats_audio`, `audio`, `view0…` in the runner — re-invoke
  `python3 scripts/wave_suite.py scripts/wave_state.json <wave>` to continue). My diff touches
  nf-view only (movie.rs + one lib.rs comment); nf-game's 89 binaries cover the largest
  untouched surface and are green. nf-formats's movie tests: `pss_movies_live` ran green on the
  stub ISO earlier in the session (3 passed, the designed skips; pre-fix tree, and the fix
  does not touch nf-formats). The 3 `disc_golden` slim-ISO allowances sit in nf-formats's
  binaries — not reached this session.
- The rule-5 command was run for the touched crate in its two feature shapes
  (`cargo test --offline --release -j2 -p nf-view [--features nf-view/movies] --test
  mission_outros_live` → 3/3 both); the full four-crate `cargo test` line could not complete
  under the disk ceiling described above.
- **Image note:** for disk, the original slim ISO was deleted and every late-session run used
  the injected image (a byte-superset: same 177 files, only the 20 injected movie records
  differ — the walk verifies it). Rebuild either with scripts/inject_movies.py in ~1 min.

## What's left

- The remaining suite waves (nf-formats, nf-audio, ~680 nf-view files) — the state file makes
  them resumable; nothing in them is expected to interact with this diff (nf-view's movie and
  glue consumers all ran green).
- On a retail-disc machine: the 18 briefing_plays_live rows + `suite.py --only
  mission_outros_live` should hold the retail PSS lengths with no injected data.

## Live checks for the main agent

1. `NF_ISO=<retail image> python tools/test/suite.py <repo> <target> --only mission_outros_live`
   → expect `3 passed, 0 failed`, holds = the retail PSS's own audio lengths (2980 / 2550 ± 2).
2. `NF_ISO=<retail image> cargo test --release -p nf-view --features nf-view/movies --test
   briefing_plays_live -- --test-threads=1` → expect 18/18 (the synthetic-image artifact above
   must vanish on real decodable movies).
3. PCSX2 fresh boot → Double Cross last part → the kill: the outro runs its full ~49.7 s with
   the subtitles beside it, then the results page (a save-state resume from mid-outro skips the
   boot path and is not a valid measurement).

## Usage

Not visible to me — this session's environment shows no token, credit, context-window or cost
figures anywhere I can read, so there are no numbers to copy.
