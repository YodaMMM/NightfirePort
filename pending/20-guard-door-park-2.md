## Your task: `guard-door-park-2`, guards find their route nodes from any spot the way the game's do

Branch from origin/s/guard-door-park, merge current origin/master, and read its docs/reports/guard-door-park.md. The game's NodesForPosition 0x1586a8:103-131 expands its search 2.0 units into the neighbouring cels, so a goal in a cel with no nodes but within 2.0 of a neighbour's bounds still finds that neighbour's nodes; the port returns none there. Read that expansion in full with addresses and port it in the game's order, keeping the Rand count. Find a place on an on-foot level where it changes a guard's route [D], and pin it through real play: the guard walks the route the game's rule gives. Then the walkreal sweep the report left (0b, 0c with barrelrow and eastroute, 0d, 0e, 11 with route, 12, 13, 14, 15) plus the 03 Agent wedge baseline, one level per tool call, and repair any walk that shifts, gated to its level. Real input only, never lower damage or health. Run the route and door binaries (grep tests for NodesForPosition), npc_doors_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-door-park-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

