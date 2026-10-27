## Your task: `level-0a-band-settle-6`, land the decoded radial tail without breaking 01's plate walk

Start from `origin/s/level-0a-band-settle-5` (its tree reverted to the settle-3 state; the five deltas are in its report) and read "What's left". Round 5's five deltas reproduced the 0a doorway slide at the game's rate but regressed walkreal_07000001_live's plate walk. Bisect the deltas against that walk with one env flag per delta, find the one that moves 01, and decide it from the decompile [M 0x109da0 / 0x10a53c-0x10a5e4] — if the game's rule really skips those faces, re-derive the 01 test's plate pins from the game's behaviour with the cause cited; if not, fix the mis-encoded delta. The per-tick jamb push table you asked for is already measured [L 2026-10-26, round 3's task file 10-level-0a-band-settle-4: HITTEST+0x10 per frame f2..f15 with Bond's position — x up to 0.061, z up to 0.048, zero from f15]; use it to pin which faces push on which tick. Then un-ignore the three walkreal_0700000a tests and keep every walkreal test green.

A previous session started this on `origin/s/level-0a-band-settle-6` and ended before any change landed (its report only holds the plan). Continue from that branch.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-band-settle-6-b`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-27T02:03Z
