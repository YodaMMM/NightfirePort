## Your task: `gas-breath-cel`, breath puffs stop under ceilings as the game's do

Branch from origin/s/gas-cel-host, merge current origin/master, and read its docs/reports/gas-cel-host.md, item 3 of "What's left". The breath sites pass `HostCel::default()`, but the game passes the caller's cel (`Env_Breath` 0x17be00's arg 3), so the ceiling clamp never applies to breath puffs. Thread the caller's cel into `env_breath` from every breath site, with addresses. Also take the real-play pin that report left: fire inside 07000004's wind cel 1 and check the muzzle puff drifts with the wind, and a flashbang's kind-0xb puff holds under a low ceiling. Rand draws must keep the game's count. Run the gas binaries, walkreal_07000004_live, the walkreal binaries for the wind-census levels one per tool call, and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/gas-breath-cel`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-09T05:06Z
