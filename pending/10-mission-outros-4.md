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

