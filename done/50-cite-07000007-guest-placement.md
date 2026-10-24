## Your task: `cite-07000007-guest-placement`, a [D] citation for the 07000007 guest on the mission path

`crates/nf-view/tests/civilian_mission_wait_live.rs:32` asserts 07000007 places a dtype-10 guest on a kind-8 mission path node (paths 2/4/5, node 0). Its only evidence is a doc name, with no offsets.

1. Read the placement records and the path/node records out of 07000007.bin and cite them [D] (archive, entry, byte offsets, placement indices).
2. If the data says otherwise, the test is wrong: fix it to the data, with the citation.
3. Add a sister that fails if the placement or the path node moves (for example a different node index).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cite-07000007-guest-placement`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-24T12:15Z

---
## Report (done, 2026-10-24T12:48Z)

Closed by the driver: the report is on `s/cite-07000007-guest-placement` (docs/reports/cite-07000007-guest-placement.md); the worker didn't run finish.sh.
