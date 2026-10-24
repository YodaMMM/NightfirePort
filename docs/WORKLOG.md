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
