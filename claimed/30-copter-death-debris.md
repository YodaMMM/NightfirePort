## Your task: `copter-death-debris`, the copter's death blast throws its metal debris

explode-debris-spawn ported `Explode_Update` 0x17c350's debris arm but noted that the car/copter death blasts (debris kind 6 [M 0x131e70, 0x17e128]) are not plumbed — the port has no copter death blast call site. On The Exchange (07000005 Part 1 / 07000008 Part 4) the copter can be destroyed. Port the copter's death blast as the game creates it (`Explode_Create` with the copter's kind-6 debris arg, its script, radius, position [M 0x17e128 and its caller]), so `Explode_Update` throws the MetalHash_132 debris. Tests: destroying the copter creates the blast with kind 6 [M]; debris objects follow on the game's cadence [M 0x17c350:34-67].

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/copter-death-debris`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-10-28T16:12Z
