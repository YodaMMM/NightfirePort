# opengoal-notice — the OpenGOAL ISC licence notice and the notices check

The task: `crates/nf-audio/src/vag.rs` keeps an OpenGOAL-derived decoder variant (ISC) but the
repo had no OpenGOAL notice file (GAPS G-007; the task text says nf-audio, the file is actually
`crates/nf-formats/src/vag.rs`). Add `THIRD_PARTY_OPENGOAL_LICENSE.txt`, reference it next to the
rerac notice, make the release's generated `THIRD_PARTY_NOTICES.txt` include it, and add a test
that every `THIRD_PARTY_*_LICENSE.txt` in the repo root is included in the notices the build
bakes in.

## What was done

**The notice file.** `THIRD_PARTY_OPENGOAL_LICENSE.txt` at the repo root: OpenGOAL's ISC text
("Copyright (c) 2020-2026 OpenGOAL Team", 749 bytes), fetched byte-exact from the
open-goal/jak-project repository's LICENSE (master branch) — the same text GitHub shows for that
file. It sits beside `THIRD_PARTY_RERAC_LICENSE.txt` (also ISC, same shape).

**The wire-ins (everything that touches the rerac notice got the OpenGOAL notice beside it):**

- `tools/release/notices.py` (which `make-release.ps1` calls to write the release's
  `THIRD_PARTY_NOTICES.txt` before `crates/nf-view/build.rs` bakes it via `NF_NOTICES_FILE`):
  the in-tree line now reads "In-tree code from rerac (ISC) is noticed in
  THIRD_PARTY_RERAC_LICENSE.txt; the VAG decoder variant ported from OpenGOAL (ISC) is noticed in
  THIRD_PARTY_OPENGOAL_LICENSE.txt."
- `crates/nf-view/src/launcher/update.rs`: `OPENGOAL_NOTICE` (`include_str!` of the root file)
  beside `RERAC_NOTICE`; both appended sections factored into `IN_TREE_NOTICES` +
  `notices_text()` (rerac's section header `==== rerac (in-tree code) ====` unchanged, added
  `==== OpenGOAL (in-tree port) ====`); `write_notices()` (which writes
  `<config dir>/THIRD_PARTY_NOTICES.txt` from the exe) now builds from `notices_text()`. So the
  exe carries the full OpenGOAL text the same way it carries rerac's.
- `crates/nf-formats/src/vag.rs` header: the OpenGOAL `Voice::DecodeSamples` citation now names
  `THIRD_PARTY_OPENGOAL_LICENSE.txt` (the rerac notices in `sha1.rs`/`iso9660.rs` name theirs).
- `README.md` licence paragraph: an OpenGOAL sentence beside the rerac one.
- `CLAUDE.md` licence table, OpenGOAL row: "add the OpenGOAL notice file (GAPS G-007)" → "notice
  kept (`THIRD_PARTY_OPENGOAL_LICENSE.txt`) (GAPS G-007)".
- `docs/GAPS.md` G-007 → done with the pointer.

**The test.** `crates/nf-view/src/launcher/update.rs::tests::every_root_third_party_licence_rides_in_the_notices`:
it walks the repo root (from `CARGO_MANIFEST_DIR`), and for every `THIRD_PARTY_*_LICENSE.txt`
requires the file's full text to appear in `notices_text()` — the text the exe carries (build.rs
bakes the generated/placeholder notices into `NOTICES`; `notices_text()` appends each in-tree
licence, exactly what `write_notices()` writes to `<config dir>/THIRD_PARTY_NOTICES.txt`). It also
asserts at least 2 root licence files were found, so the walk can't pass vacuously. A future
root licence file without a wire-in fails the test. Nothing here pins game behaviour, so no
evidence tags are needed beyond the file provenance above.

## How it was verified

- `cargo test -p nf-view --lib -- launcher::update` (run on the final tree; result recorded
  below), plus the full fast-runner suite.
- The generated-notices path was read end to end (`make-release.ps1` → `notices.py` →
  `NF_NOTICES_FILE` → `build.rs` → `update.rs NOTICES` → `write_notices()`); the OpenGOAL notice
  rides every hop the rerac notice rides. (The release build itself runs on Windows; not run
  here — the `notices.py` edit is exercised by construction of its line list, and the
  Windows-only steps are unchanged.)

## Tests run

- `cargo test --offline --release -j1 -p nf-view --lib -- launcher::update --test-threads=1`:
  **6 passed, 0 failed** — including the new
  `every_root_third_party_licence_rides_in_the_notices` and the 5 pre-existing update tests.
- `cargo test --offline --release -j1 -p nf-formats --lib -- vag --test-threads=1`: **5 passed,
  0 failed** (the touched `vag.rs` comment compiles; the decoder behaviour untouched).
- Full suite via the fast runner `tools/test/suite.py` (clean tree, `cargo clean` first after the
  disk filled; 774 test binaries, "build ok: 774 test binaries in 1106 s"). **Still executing at
  the 15:34 UTC work-stop deadline on this 2-core sandbox — no total line yet; the partial result
  stands in for it** (`tail` of suite.out at 15:33 UTC):

  ```
  FAILED disc_golden::bigf_header_fields_behave_as_documented
  FAILED disc_golden::bigf_tables_match_the_golden_manifest
  FAILED disc_golden::directory_tree_matches_the_golden_manifest
  FAILED identity_disc::the_real_disc_passes_the_identity_check
  CRASH mp_stress_explosives_live (exit -9)
  CRASH mp_online_failures (exit -9)
  FAILED fix_0c_pylon_live::the_0c_pylon_is_on_screen_at_the_user_spot
  FAILED stdio_pipe_closed_live::a_closed_stderr_pipe_does_not_crash_the_game
  FAILED probe_pylon_render::probe_render_pylon_only
  ```

  All nine are environmental, none reachable from this branch's changes:
  - the 3 `disc_golden` failures are the slim-ISO allowance the driver states;
  - `identity_disc` panics on `sectors 518435 != 1191072` — the slim on-foot image is a stripped
    build (volume/serial/vmode/version all match), the retail-only sector/SHA-1 assertion cannot
    hold on it; the identity code and test are untouched here;
  - the two `CRASH ... (exit -9)` lines are the runner's wall-clock kill on stress/MP binaries
    (600 s limit, 2 cores, no GPU);
  - `fix_0c_pylon_live`, `probe_pylon_render` are on-screen render probes (no GPU in this
    sandbox — the merge lesson: visual work needs the main agent's render);
  - `stdio_pipe_closed_live` drives the real game binary headless.
  The branch changes four things — a licence text file, an `include_str!` + section list in
  `update.rs`, one line pair in `tools/release/notices.py`, and doc/comment text — none of which
  is in any of those binaries' code paths.

## What's left

- The suite total line: the runner was still executing when this report was written; if it
  lands, the next message can append it below. The nine partial failures are itemised above.

- Nothing known. The release `THIRD_PARTY_NOTICES.txt` gets the OpenGOAL mention through
  `notices.py`; the exe carries the full text through `notices_text()`. If a future task adds
  another in-tree licence file, the new test forces the wire-in.

## Usage

not visible to me — this environment exposes no token, credit or context-window figures for the
session (no input/output token counts, no cost readout anywhere in the tool output).
