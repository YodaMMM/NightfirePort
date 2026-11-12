## Your task: `level-0a-civilian-slide-cels-2`, finish the portal-visible drawn gate

Start from origin/s/level-0a-civilian-slide-cels (the drawn gate on the game's portal-visible room rule; nf-game celgraph.rs; Drones::tick_with_cel_graph) merged with current origin/master. The main agent's check: `walkreal_0700000c_eastroute_live::the_ground_route_crosses_to_the_east_side` PASSES on master and FAILS with the branch: after "the ledge to the arc" Bond is at (8.99, 2.01, -222.09), off the 12.4 ledge, and the walk wedges at the arc west end (bond 11.36, 1.13, -217.52); guard #15 stands in state 0x5f at (11.36, 12.24, -218.92) right at the arc end. With the portal rule that guard now reads drawn (on screen) where the old ray gate left him undrawn, so his drawn-side behaviour (physics, collision push, movement) now runs. Find which drawn-gated arm moves Bond off the ledge (or blocks him) and check it against the decompile; if the game does the same, re-tune that walk leg on real input (no writes). Then the full suite (the drawn bit also feeds auto aim, seek targets and BondTalk on every level: list every walk that moved and why). Then the main agent renders the 0a vantage and compares with PCSX2 before merging.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-level-0a-civilian-slide-cels-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-11-12T03:10Z

---
## Report (done, 2026-11-12T04:33Z)

