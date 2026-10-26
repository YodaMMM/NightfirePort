# mission-outros-2 — the outro hold on the REAL disc

## What was wrong

On an image with the real movies (`07300004.PSS` / `07300013.PSS` present), both outro tests
ended the state-6 movie hold after **1 tick** in the headless suite. Root cause, fully traced:

- The suite builds the test binaries **without** the `movies` cargo feature (opt-in decoder),
  so `Player::open` took its no-feature path — which returned `Ok` with `ended: true`
  ("reports done immediately") for a file that had **opened fine**.
- The glue then set `movie_playing = true` for exactly one tick; the next poll saw
  `finished()` and state 6 popped with `movie_ticks == 1`.

That model contradicts the game: a **successful** open starts the movie —
`BGFMVPlaying = 1` [M psiStartBackgroundMovie 0x1081a8: the `strFileOpen` success arm], and it
is cleared only when the decode pump finishes the file [M psiDecodeBackgroundMovieFrame
0x108060: `DecodeBGFMVFrame` returns 1 → the non-looping arm frees everything and
`BGFMVPlaying = 0`; main 0x109080 pumps it per vblank pair]. `psiMovieFinished` is
`BGFMVPlaying == 0` [M 0x1082e8]. Only a **failed** open (not whitelisted, file absent, 0-byte
stub) leaves `BGFMVPlaying == 0` and pops state 6 at once [M 0x1081a8's strFileOpen failure arm].

## The fix (branch s/mission-outros-2, commit 82367ac7)

`crates/nf-view/src/movie.rs`:

- `Player` now carries the movie's **real playback length in 60 Hz game ticks** — computed off
  the disc at `open()` from the PSS's own audio track (`Audio::seconds()`; the audio track runs
  the whole file and the game paces the picture to it [M audioDecSendToIOP 0x107100 + main
  0x109080]; no audio stream falls back to counting MPEG pictures at the 30 fps the sequence
  header names). Disc evidence for the two outros [D golden/pss_movies.tsv, measured off the
  retail image]:
  - `07300004.PSS` = 2 384 128 PCM samples at 48 kHz = 49.67 s = **2980 ticks**
  - `07300013.PSS` = 2 040 064 PCM samples at 48 kHz = 42.50 s = **2550 ticks**
- When no decoder runs (no `movies` feature, or a declined worker) the opened file still holds
  for that real length: `headless_advance()` steps the stand-in down once per host poll; at 0 it
  sets `ended` — the game's end-of-file arm. A live decoder ignores it.
- A **declined** start (not whitelisted / file absent / 0-byte stub) still reports finished
  immediately — the game's un-started movie, `BGFMVPlaying == 0` [M 0x1081a8].

`crates/nf-view/src/lib.rs`: the three host BGFMVPlaying polls (the state-6 glue, the intro
poll, the attract poll) call `headless_advance()` once per frame before reading `finished()`.
The state-6 hold therefore lasts the movie's real length on the real path, and the subtitle
script runs beside it the whole time [M 0x1d6360's SP_Create beside psiStartBackgroundMovie].

`crates/nf-view/tests/mission_outros_live.rs`: the two tests now

- measure the PSS length off the image at run time (`movie_ticks60`) and, **when the file is
  present**, assert the hold ran it (`movie_ticks.abs_diff(pss) <= 2`, citing 0x1081a8 +
  0x108060 + 0x1082e8 and the disc bytes);
- keep the `>= 2690` / `>= 2280` track-coverage assertions — now disc-backed on both sides
  (the track entries [D 07000500.bin] and the movie lengths [D golden/pss_movies.tsv] both
  exceed them);
- fall back to the no-movie stand-in assertions **only when the image stubs the file**, where
  the game itself pops at once and the port's [S] track-length hold applies.

## What was verified here

- `mission_outros_live` — 3/3 green on this sandbox's slim image (the stub branch: the [S]
  stand-in hold, unchanged behaviour).
- Adjacent green set, all ok: `reward_after_outro` (3), `doublecross_transitions_live` (6),
  `fix_mission_live` (4), `walkreal_07000004_live` (3), `movie_playback_flow` (9),
  `gameflow_stack_sp_run` (4), `front_end_chain_live` (2), `menu_audio_flows` (9).
- `fix_0c_vent2_live` 3/3 green in isolation (see suite note below).

## Full suite on the final tree (s/mission-outros-2 = origin/s/mission-outros + origin/master)

737 fresh test binaries across nf-game / nf-formats / nf-audio / nf-view (single-threaded inside
each binary, `NF_ISO` + `NF_SILENT` set, per-package cwd — the same shape as tools/test/suite.py).

- 308 binaries completed with results: **940 passed, 2 failed** — both failures in
  `fix_0c_vent2_live` were sandbox memory-pressure artifacts (the 4-way parallel run triggered
  the OOM killer mid-run; the binary passes 3/3 in isolation).
- ~30 binaries were killed mid-run by the same OOM pressure (no result line); the remaining
  ~392 did not get a window before the sandbox deadline (at -P 2 the heavy live tests ran
  ~15 s/bin and the 2-hour window ran out).
- This sandbox's image stubs every /MOVIES PSS [D — the ISO tree: all 23 records are
  size 0], so the **real-movie branch of the fix is not executable here**; it is exercised by
  the two updated tests on any image with the retail movies.

## What's left for the next session / the main agent

- Run `mission_outros_live` on the real-disc machine: the two tests should pass with
  `hold ≈ 2980` / `hold ≈ 2550` (±2). If the hold is off by more, the count mechanism (±1 from
  the state-6 entry path) needs a look, not the model.
- Optional live check (PCSX2, fresh boot): the Double Cross outro's state 6 stays up ≈ 49.7 s
  (2980 ticks) with the subtitles running beside it, then routes to the front-end map.
- Complete the remaining ~392 suite binaries on a machine with more RAM/time.

## Live checks for the main agent

1. On the real-disc machine: `NF_ISO=<full image> cargo test --offline --release -p nf-view
   --test mission_outros_live -- --test-threads=1` — expect 3 passed; the two hold assertions
   print nothing (they pass silently) and the eprintln shows the track entries.
2. PCSX2 fresh boot → Double Cross last part → the kill: the outro movie plays its full ~50 s
   with subtitles, THEN the results page. A save-state resume from mid-outro would skip the
   boot path and is not a valid measurement.

## Usage

Not visible to me — this session's environment shows no token, context-window or cost figures
anywhere I can read, so there are no numbers to copy.
