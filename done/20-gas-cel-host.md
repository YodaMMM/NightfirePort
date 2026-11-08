## Your task: `gas-cel-host`, gas clouds drift and stop at ceilings as the game's do

Branch from origin/s/markers-world-objects, merge current origin/master, and read its docs/reports/markers-world-objects.md, item 2 of "What's left". The gas code does not resolve which cel the spawning object sits in. So the type-7 gas sites miss their wind term, and the kind-0xb puff misses its ceiling clamp. `Rbodies.cels` and `winds` already hold the data. Read the game's cel lookup for both, with addresses, and port it. Rand draws must keep the game's count. List which levels have type-7 gas sites or kind-0xb puffs [D]. Pin through real play on one of them: the cloud drifts with the room's wind and stops under the ceiling as the game's rule says, and Bond takes gas damage where the cloud really is. Run the gas binaries (grep tests for `gas`), the walkreal binaries for the census levels and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/gas-cel-host`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-08T20:30Z

---
## Report (done, 2026-11-08T21:22Z)

Closed by the driver: the report is on `s/gas-cel-host` (docs/reports/gas-cel-host.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-08T21:22Z)

