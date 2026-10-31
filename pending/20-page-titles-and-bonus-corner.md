## Your task: `page-titles-and-bonus-corner`, two front-end layout leftovers

Two items from origin/s/fix-stats-page (read its notes; it matched the Stats/Results pages to PS2 captures): (1) every page's title sits about 2.5 units higher than the PS2 capture (the shared title draw) — find the game's title placement [M the page draw / title control handler; D the page records] and fix it for all pages; (2) the 007 Bonus page has the same corner-art control (0x10000224) but still draws it in the old position — use the same rule the stats page now uses (and settle U-052 in docs/UNKNOWNS.md if the code explains why that control skips the 640x480 -> 512x448 scale). Branch from origin/master after merging origin/s/fix-stats-page. Render before/after; pin with tests (sisters). Run every menu/front-end binary. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/page-titles-and-bonus-corner`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

