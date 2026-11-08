## Your task: `pane-ride-on-3`, intact glass really stops Bond on every level

Branch from origin/s/pane-ride-on-2, merge current origin/master, and read its docs/reports/pane-ride-on-2.md. That session's tree is unchanged code with two proven laws and a recipe, so take "What's left" in order: re-land Laws 1 and 2 and the crack law from the recipe, bisect the ghost coupling on 00 Agent, then audit the 00 Agent hall bleed, which is the last blocker. Then set `PANE_RIDE` on and remove both dead_pane_live ignores. Keep the kind-0/1 props out of the set and carry the live checks forward. Real input only, no top-ups, never lower damage or health, and level helpers stay gated to 07000001. Run dead_pane_live, glass_pass_every_level_live, every walkreal_*_live, the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/pane-ride-on-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-08T01:10Z
