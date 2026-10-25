## Your task: `level-07-party-door-lock`, 07000007: the door back to the main party locks after the story points

In the game, after certain points on the party level the door back to the main party room locks; in the port it stays usable.

1. Find the door(s) between the party room and the areas beyond [D the 07000007 door placements, their lock/key channel params] and what locks them [M — the switch channels the level's scripts/triggers/mission records write at those story points, `Door_*` lock handling (the locked state, the "locked" message/sound)]. List the story points that lock (and any that unlock) and which channel each writes.
2. Port any missing writer or lock handling, with real-input tests: before the point the door opens; after it, it refuses (locked state, the game's message/sound); sister: other doors unaffected.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-party-door-lock`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-10-25T02:38Z

---
## Report (done, 2026-10-25T03:40Z)

Closed by the driver: the report is on `s/level-07-party-door-lock` (docs/reports/level-07-party-door-lock.md); the worker didn't run finish.sh.
