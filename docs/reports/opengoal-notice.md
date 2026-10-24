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
  (result appended below after the run).
- Full suite via the fast runner `tools/test/suite.py` on the final tree (total line appended
  below).

## What's left

- Nothing known. The release `THIRD_PARTY_NOTICES.txt` gets the OpenGOAL mention through
  `notices.py`; the exe carries the full text through `notices_text()`. If a future task adds
  another in-tree licence file, the new test forces the wire-in.

## Usage

not visible to me — this environment exposes no token, credit or context-window figures for the
session (no input/output token counts, no cost readout anywhere in the tool output).
