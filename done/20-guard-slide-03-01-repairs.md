## Your task: `guard-slide-03-01-repairs`, 07000003's and 07000001's Agent walks are green with the guard slide rule everywhere

Branch from origin/s/guard-slide-gate-sweep, merge current origin/master, and read its docs/reports/guard-slide-gate-sweep.md, items 1 and 2 of "What is left". The slide law is the game's and stays. First the 07000003 repair: diagnose the wedge and re-take the lane exit, gated to 07000003 and the test's difficulty, following the ledger on the test. Then continue the 07000001 Agent repair: re-time the Mayhew sister per leg, then the 900-tick step-out, tracing the 13556 leg with NF_RF_TRACE. One level per tool call. Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000003_live and walkreal_07000001_live binaries and their interactables binaries.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-slide-03-01-repairs`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-11T14:09Z

---
## Report (done, 2026-11-11T15:21Z)

The full report lives on the branch: docs/reports/guard-slide-03-01-repairs.md
(s/guard-slide-03-01-repairs, pushed). Summary:

**Done:** the 07000003 wedge DIAGNOSED with the ledger's first lever (probe_plate_wedge_spot):
no slid guard — the parked capsule is the level's own courtyard POCKET at (6.35, −15.7..−17.7)
(the 16-ray sweep from inside stops 0.05–0.21 u on every side but south; Bond parks grounded
false y 7.02–7.21 AND grounded true y 6.41–6.61; the A* rides path3's upper-row nodes
(10.8/11.28, 9.64) through the band; the door-8 mouth walk's A* from the west loot end
(3.69, −28.84) parks him, 21 give-ups, then 29 on the plate leg). The lane-exit re-take
(off_the_courtyard_obstacle_03, d1-gated, test-local, self-gating) landed at the four
servant-loop points; two attempts measured (v1 east/west episodes — the lane anchor
re-funnelled Bond in; v2 south-extract — he was back in the pocket each next leg). The d1 test
stays IGNORED with the full ledger + three next levers (the ground-level exit map, the
extraction-anchor shape, the east-lane start gate on the mouth/servant walks).

**Verified:** walkreal_07000003_live whole on the final tree: 5 passed / 0 failed / 4 ignored
in 197.7 s — the d2/d3 sisters green with the helper in the tree. Real input only, no
top-ups, no health/damage writes, no test weakened.

**Left:** the d1 extraction (the three levers), the whole walkreal_07000001_live +
interactables binaries and the 07000001 Agent repair (the mayhew per-leg re-time, the
900-tick step-out, NF_RF_TRACE the 13556 leg — the four-lever ledger on the Agent test's
ignore stands), the full suite with the fast runner on the final tree.

## Usage

not visible to me (no token, context-window or cost figures are exposed in this session).
