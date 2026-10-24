## Your task: `cite-14-skinless-drones`, a [D] census for the 07000014 skinless drones

`crates/nf-view/tests/missing_models_census_live.rs:224` pins `skinless_drones == 5` on 07000014 as "the shipped game's own skinless renders". Its only evidence is a doc name.

1. Census the disc [D]: which archives the 14 load path pulls in, and which of them hold or lack skin 0x050000b1 (archive, entry, offsets). Name the five placements.
2. If the game would find the skin somewhere the port does not look (a session table, a shared archive), say so with the [M] address. The test then changes to the game's rule.
3. If the claim cannot be settled from the disc and the decompile, mark the pin `// [S] unverified` and keep it out of pass/fail. Name the PCSX2 read that would settle it (fresh boot, which address).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cite-14-skinless-drones`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-24T12:28Z
