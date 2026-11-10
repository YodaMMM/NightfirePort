## Your task: `pretrans-arms-0c-11`, the 0700000c handoff and east route and the 07000011 d3 pocket walks are green

Branch from origin/s/pretrans-arms-5, merge current origin/master, and read its docs/reports/pretrans-arms-5.md item 4 and docs/reports/pretrans-arms-4.md, items 5 and 6 under "Ledgered ignores". First the 0c handoff and east-route ground route: the wire #282 B-end reach miss, by the ride-release lever in its ignore message. Then 07000011's d3 pocket: the connector-2 mouth stands and the re-ledger from ffe3ed6a5. One level per tool call, per-level walk repairs, no sim changes. Helpers stay gated to their level and difficulty. Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run walkreal_0700000c_live with its eastroute and barrelrow binaries, walkreal_07000011_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/pretrans-arms-0c-11`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-10T23:57Z
