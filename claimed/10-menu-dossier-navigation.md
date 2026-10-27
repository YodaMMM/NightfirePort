## Your task: `menu-dossier-navigation`, the Dossier pages navigate like the game's

In the front end, the Dossier (page 0x4000002b and the pages it opens, including the one reached from the mission Results page's Tab) is hard to move around in, and from it the player can land on the mission-select page with no way back to where they came from. Port the front end's page stack and per-page Back / Cancel / cursor arms exactly as the game runs them [M MenuManager_Create 0x21b908's page tags, the MenuManager push/pop and back-key handling, each Dossier page's handler and its control focus order]: every page reachable from the Dossier returns with Back to the page that opened it, the cursor moves through each page's controls in the game's order, and no path strands the player. Drive it through the real menu path in a test (main menu -> Dossier -> each sub-page and back; Results -> Tab -> Dossier -> back) with the page ids and focus order cited [M].

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/menu-dossier-navigation`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-27T08:45Z
