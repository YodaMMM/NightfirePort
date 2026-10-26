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

