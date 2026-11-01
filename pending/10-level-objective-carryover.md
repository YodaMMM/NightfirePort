## Your task: `level-objective-carryover`, 06/07/08 open with 05's objective

Loading 07000006, 07000007 or 07000008 directly, the port's objective box opens with "Breach the castle walls" (07000005's objective); the PS2 shows that level's own objective (07: "Rendezvous with undercover agents...") [L PCSX2 direct loads, the start-position lighting sweep]. 0700000b may do the same. Find where the port picks the opening objective [M the mission/objective init per level — the objective text bank index, the level's MissionState setup; D each level's objective entries] and why it lands on 05's for these levels (shared archive? the mission's first map? a stale index?), fix it with the address cited, and pin the opening objective text for every on-foot level on a direct load AND on the 05 -> 06 hand-off (sister: two levels give different text). Run the objective/hud/campaign binaries you touch and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-objective-carryover`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

