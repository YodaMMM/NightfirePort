## Your task: `level-06-start-subtitles`, 06: the driver dialogue subtitles at the start

At the 07000006 start the PS2 shows the driver dialogue subtitles ("Can't I rest a moment?...", "Then it's six hours back, old man!...") [L PCSX2 lighting-sweep capture]; the port shows none. First check the port's subtitle option default against the game's [M the settings init / save default]; if the default is on, find where the dialogue's subtitle lines come from [D the script streams / text bank entries for that conversation; M the subtitle post path] and port it. Pin: the lines post at the script's ticks with the option on, none with it off. Run the subtitle/script binaries you touch and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-06-start-subtitles`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-01T07:15Z

---
## Report (done, 2026-11-01T08:49Z)

Closed by the driver: the report is on `s/level-06-start-subtitles` (docs/reports/level-06-start-subtitles.md); the worker didn't run finish.sh.
