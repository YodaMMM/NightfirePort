## Your task: `level-11-d3-pocket`, 07000011 can be walked on 00 Agent through the d3 pocket

Branch from origin/s/pretrans-arms-0c-11, merge current origin/master, and read its docs/reports/pretrans-arms-0c-11.md, the first item of "What's left", plus the 07000011 00 Agent ignore message. Bond reaches the 23.0 strip's east end at 22.0 health, and the 18.9 pocket guards #18, #19, #26 and #36 finish him in the close free-hit band [M 0x179740:113-115]. Take its two levers in order: first thin the pocket before it wakes, with a stand from the 23.0 strip's north window on the #18/#19 pair filing up the connector; then cut the upstream 27.1-strip crossfire so the descent starts with about 40 health, as the earlier tree had. Judge each over several real runs. Helpers stay gated to 07000011 and the difficulty, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000011_live binary and interactables_07000011_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-d3-pocket`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-11T03:32Z

---
## Report (done, 2026-11-11T04:18Z)

