## Your task: `pickup-medal-gate`, the kind-6 pickups' gold-medal gate

Pickup kind 6 (the '007' bonus pickups, 07000011's two rows and 0700000a's) is gated by `PlrStats_HasGoldMedal` 0x199a40 in the game; the port ignores the gate. Port the gate and the medal state it reads (the per-level medal record the save keeps, `PlrStats` [M]), so the pickups appear/take exactly when the game allows. Tests: with no gold medal for the level the kind-6 pickup behaves as the game's no-medal arm [M]; with the medal recorded, the other arm [M]; cite the create/take addresses.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/pickup-medal-gate`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-26T16:08Z
