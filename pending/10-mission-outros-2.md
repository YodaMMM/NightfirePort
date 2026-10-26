## Your task: `mission-outros-2`, the outro movie + subtitle script on the REAL disc

Start from `origin/s/mission-outros` (merge it, then origin/master). On a machine with the real disc image (the movies present), its own two new tests fail:
- `mission_outros_live::the_double_cross_outro_runs_its_subtitle_script` — "the outro stayed up through the track's entries [S the hold length is the port's stand-in; the disc track's 9th entry completes at t2690], got 1"
- `mission_outros_live::the_phoenix_fire_outro_runs_its_subtitle_script` — the same, "got 1"

Your sandbox's image had no movie files, so the tests exercised the no-movie hold; with the real 07300004.PSS / 07300013.PSS present the movie path runs and the outro ends after 1 tick in the headless test. Find why the state-6 movie hold ends at once in the headless path with a real movie (the decoder not driven per tick in the test harness? psiMovieFinished read as finished before the first frame [M 0x1082e8]?) and what the game does: state 6 holds until BGFMVPlaying == 0 (the movie's real length) and the subtitle script runs beside it [M 0x1d6360:101-112]. Make the hold last the movie's real length on the real path, and make the tests assert that with the real disc (and the no-movie stand-in only when the file is absent). Run the full suite against the full ISO before reporting.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mission-outros-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
