# WORKLOG

## opengoal-notice

- (13:57 UTC, 10-24) Branch `s/opengoal-notice` created from `origin/master` (26543436).
  Task: add `THIRD_PARTY_OPENGOAL_LICENSE.txt` (OpenGOAL's ISC licence, from the jak-project
  repository's LICENSE) — `crates/nf-audio/src/vag.rs` keeps an OpenGOAL-derived decoder variant
  (ISC) but the repo has no OpenGOAL notice file; reference it next to the rerac notice, make the
  release's generated `THIRD_PARTY_NOTICES.txt` (make-release.ps1 / crates/nf-view/build.rs)
  include it, and add a test that every `THIRD_PARTY_*_LICENSE.txt` in the repo root is included
  in the notices the build bakes in. Note: master dropped the root WORKLOG and docs/WORKLOG.md
  (batch-1023c); re-adding docs/WORKLOG.md on this branch per the current task-branch convention.

- (14:03 UTC, 10-24) Notice added and wired. `THIRD_PARTY_OPENGOAL_LICENSE.txt` = OpenGOAL Team's
  ISC text, byte-fetched from open-goal/jak-project master LICENSE (749 bytes). Wired next to the
  rerac notice: tools/release/notices.py's in-tree line (make-release.ps1 calls it), update.rs
  (OPENGOAL_NOTICE include_str + IN_TREE_NOTICES + notices_text(); write_notices now builds from
  notices_text), vag.rs header, README.md, CLAUDE.md G-007 row, GAPS.md G-007 → done. New test
  `every_root_third_party_licence_rides_in_the_notices` (nf-view update.rs): walks the repo root,
  requires every THIRD_PARTY_*_LICENSE.txt's full text inside notices_text (>= 2 files). Suite
  restarted after a full-disk cleanup (cargo clean, 6.8 GB); full run pending — this checkpoint
  pushes before that verification completes.
