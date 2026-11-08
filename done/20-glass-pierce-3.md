## Your task: `glass-pierce-3`, the 07000008 walks win their Hind fights with rounds passing glass as in the game

Branch from origin/s/glass-pierce-2, merge current origin/master, and read its docs/reports/glass-pierce-2.md. Take items 1 and 2 of "What's left". On the shifted timeline the topped-up 08 walk's Hind took all nine rockets the walk carries (about 34 each against about 101 on master), because the detonations fall at the blast's edge. Change the fight's launch discipline: wait for the machine's closer approach so the contact blast is full strength, or aim at the body sphere's centre instead of the chase point, then re-pin the ledger. The 08 d3 sister cleared the mission and died at 28105 to a 62.5 blast at about 3 u; fix it the same way. The flight pin is not the problem. Never lower damage or health, and keep the rocket count the walk really carries. Run walkreal_07000008_live, the bullet binaries, glass_pass_every_level_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/glass-pierce-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-08T04:19Z

---
## Report (done, 2026-11-08T05:09Z)

