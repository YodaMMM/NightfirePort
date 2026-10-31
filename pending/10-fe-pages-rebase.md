## Your task: `fe-pages-rebase`, land the credits/menu pages on top of the stats page

`origin/s/fe-pages-2` (the credits roll, 007 Bonus text, win page movie, Screen Adjust, PARISENUM card, cheat-medal page) and `origin/s/fix-stats-page` (the Mission Stats/Results pages) both change crates/nf-formats/src/hud.rs, crates/nf-view/src/lib.rs and crates/nf-view/src/menu.rs (both added the FONTMedium load and menu-overlay changes). Make a branch from origin/master, merge origin/s/fix-stats-page, then merge origin/s/fe-pages-2 and resolve every conflict keeping BOTH features (one FONTMedium load; both pages' draw code), then merge origin/master again. Every page must still render as both branches' tests pin (fe_pages_draw.rs, fe_pages_draw2.rs, stats_page_elements.rs). Run every menu/front-end/stats test binary. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/fe-pages-rebase`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

