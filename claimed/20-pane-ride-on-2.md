## Your task: `pane-ride-on-2`, intact glass really stops Bond on every level

Branch from origin/s/pane-ride-on, merge current origin/master, and read its docs/reports/pane-ride-on.md. The bisection is settled and the Operative 01 real-damage walk passes with `PANE_RIDE` on (3.0 health left, with finish_fights). What blocks the flip is the Agent sister: the west-door SMG pack #3, #12 and #17 shred Bond at d 1.0-4.6 with no return-fire fight ever fired at them, because the shifted patrol timeline keeps the pack out of every fight window, and the walk dies at tick 5774. Give the pack a fight window (a pre-clear or a leg gate, gated to 07000001), then reach and fix the 00 Agent sister, which that session never got to. Then set `PANE_RIDE` on and remove both dead_pane_live ignores. Real input only, no top-ups, never lower damage or health. Keep the kind-0/1 props out of the set and carry that report's live checks forward. Run dead_pane_live, glass_pass_every_level_live, every walkreal_*_live, the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/pane-ride-on-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-07T15:41Z
