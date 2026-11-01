## Your task: `gas-rng-draws`, gas puffs: the shared-RNG draws the port leaves out

The port's gas puffs take no draws from the shared RNG, but the game's do: the x-ray breath variant A draws `Rand_FRand(2*pi)` from the shared stream [M Env_Breath 0x17be00, disasm 0x17bf8c-0x17bfa0] and `Gas_Init` 0x1aacc8 draws its spin and sign. Leaving them out shifts every later shared draw (AI, bullets) vs the game. Port the draws in the game's order with the addresses, check determinism (cross_process_determinism, fps_determinism_live) and re-pin any test whose seed-dependent outcome moves (say which, and why the new value is the game's). Run the gas/breath/smoke binaries, ai_live and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/gas-rng-draws`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-01T21:53Z

---
## Report (done, 2026-11-01T22:34Z)

Closed by the driver: the report is on `s/gas-rng-draws` (docs/reports/gas-rng-draws.md); the worker didn't run finish.sh.
