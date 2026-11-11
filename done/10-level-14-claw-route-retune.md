## Your task: `level-14-claw-route-retune`, Countdown's infiltration walks green after the claw wall fix

Branch from origin/master. walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage and its _agent sister are ignored: since the claw wall fix the route's SkyBridge crossing is a stand-in (the claw bolt dies on the duct wall) and the shifted clock puts the descent at (29.63, 0.65, 29.25) instead of the leg-2 start (29.99, 0.61, 38.00), and leaves the Agent run short on the pit band's east jump (46.34, -1.78, 53.19). First check the claw: does the game's claw reach the SkyBridge grapple point from that stand, or is the duct wall really in the way [M the claw's ray and the grapple target]? Then re-tune both walks with real input and un-ignore them. Never lower damage or health, no top-ups. Run walkreal_07000014_live and interactables_07000014_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-14-claw-route-retune`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox7 at 2026-11-11T15:36Z

---
## Report (done, 2026-11-11T16:32Z)

