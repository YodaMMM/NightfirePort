## Your task: `menu-stats-page`, the Mission Stats page draws like the game's

The Stats page (`P_NFSTATS` 0x209938, reached by Del from the mission Results page on 07000048) shows the right numbers but is drawn badly: every row label prints as its raw string id (`<0x165>`, `<0x15a>`, `<0x15b>`, `<0x15c>`, `<0x16e>`, `<0x15d>`..`<0x161>`, `<0x162>`, `<0x164>`, `<0x163>`), there are no column headers except TOTAL, the value columns overlap (`100%50,000`, `08:2037,352`, `----25,000`), and the page has no backdrop/frame. Resolve the labels through the game's string lookup for those ids [M the page's 0x4c build :31-38 and its text controls; the language bank the other pages use] [D 08000002's Stats page tags], and lay out every control - headers, columns, widths, alignment, backdrop sprites - from the page's own control data and draw code [M P_NFSTATS + the menu control draw paths] the way the other ported pages already do. Test the page's control list/text/positions against the [D] page data; the main agent renders and compares to the PS2.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/menu-stats-page`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-28T03:24Z

---
## Report (done, 2026-10-28T05:00Z)

