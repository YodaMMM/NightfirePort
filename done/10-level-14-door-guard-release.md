## Your task: `level-14-door-guard-release`, all the released 07000014 door guards reach the locked door

`crates/nf-view/tests/door_guards_locked_door_07000014_live.rs` (~line 90) is #[ignore]d with two blockers. Take blocker (2) only (the NoBunching scan flip is `20-drone-nobunching-side-bits`, do not touch NB_SCAN_LIVE): after the release, only ONE guard reaches the door in the port (#25, cycling at (-50.1,41.7)); #18/#19/#20 stay in state 0x38 near (-51.3,32.4)/(-47.9,33.5)/(-40.2,25.2). In the game two guards stop 2.4 apart at (-51.6,41.2)/(-51.2,38.8) [L 2026-10-30, in the test note]. Find why the three stay at 0x38: trace their state, route request and the release trigger/channel that should move them [M: the 0x38 state handler and its exits; the release writer]. Fix the port so the released guards route to the door as the game's do, and pin it (guard count at the door, their states) with a test. Run the crates you touch plus every 07000014 test binary.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/14-door-guard-release`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-10-31T07:37Z

---
## Report (done, 2026-10-31T08:57Z)

