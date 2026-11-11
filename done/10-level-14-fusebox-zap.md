## Your task: `level-14-fusebox-zap`, 07000014's Operative real-damage walk survives the fusebox

Branch from origin/s/level-14-skybridge-retune-2, merge current origin/master, and read its docs/reports/level-14-skybridge-retune-2.md plus the ignore message on walk_the_infiltration_to_the_delta_exit_real_damage. The walk is healthy to leg 3 and then dies to the fusebox zap. First find the zap's source in the game: which object hurts Bond there, its radius and its damage, read from the decompile and the level data with addresses [M]/[D]. Check the port matches it, and if not, fix the port in the game's way and test it red on the old code. Then route the walk around it, using the ledger's 4 u or wider stance spot if the game's radius allows. The SkyBridge claw-up re-route and the 00 Agent walk are not this task. Helpers stay gated to 07000014 and the difficulty. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000014_live binary and interactables_07000014_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-fusebox-zap`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-11T11:29Z

---
## Report (done, 2026-11-11T12:31Z)

