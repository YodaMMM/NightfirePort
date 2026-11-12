## Your task: `level-0c-vent-crawl-2`, the 0c vent route walked end to end on real input, out through the crawl

Branch from origin/master. Live facts [L PCSX2 fresh boot 0700000c, 2026-11-11]: from the display-case vent shaft floor (5.6, 12.95, -111.2) the only way out is crouched (the crouch toggle) crawling SOUTH under the sill: Bond ends at (5.6, 11.882, -113.54) in the next room behind the storage crates. Standing south stops at z -111.395; north (standing and crouched) stops at z -110.94; east/west stop at x 6.187 / 5.332. The port matches all of these from a placement (crouched south (5.6, 11.924, -113.540)).
Do: a real-input walk with no position writes from the level load: the walkway, through the burned hatch and the duct, off the sill lip into the shaft (plate #274 fires mid-fall), crouch, crawl south out, stand, and walk on toward the next objective. Assert each leg's end against the live numbers above (0.1 tolerance on the crawl exit). If any leg needs a different input than a player would use, report it. No draw changes.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/05-level-0c-vent-crawl-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-11-12T00:02Z
