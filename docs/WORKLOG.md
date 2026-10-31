
## level-14-door-guard-release (started 07:49 UTC 2026-10-31)

Task: make all the released 07000014 door guards reach the locked door —
`crates/nf-view/tests/door_guards_locked_door_07000014_live.rs` (~line 90) is #[ignore]d with
two blockers; take blocker (2) only (NoBunching scan flip is 20-drone-nobunching-side-bits, do
not touch NB_SCAN_LIVE): after the release only ONE guard reaches the door in the port (#25,
cycling at (-50.1,41.7)); #18/#19/#20 stay in state 0x38 near (-51.3,32.4)/(-47.9,33.5)/
(-40.2,25.2). In the game two guards stop 2.4 apart at (-51.6,41.2)/(-51.2,38.8) [L 2026-10-30,
in the test note]. Trace the 0x38 state handler and its exits plus the release writer, fix the
port so the released guards route to the door as the game's do, pin it with a test, run the
touched crates plus every 07000014 test binary. Branch `s/level-14-door-guard-release`, from
origin/master 7d0091a0.

Progress log:
- 07:49 branch created from origin/master 7d0091a0 (sandbox reused from the level-02 session:
  repo, work/, ISO, nf-queue and cargo all present; freed overlay disk for the rebuild). Branch
  pushed with this entry.
