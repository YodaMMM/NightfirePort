## Your task: `level-13-lobby-rifles`, the 07000013 lobby rifle guards can be answered from the walked lane

`crates/nf-view/tests/walkreal_07000013_live.rs` `walk_the_warm_chain_to_the_front_entrance_and_out_real_damage` (~line 326) is #[ignore]d: on real damage the walk dies in the north lane at z ≈ 39 under two rifle guards (mezzanine #4 at (15.4, −362.4, 42.3) and spawner #6 at (4.1, −368.4, 37.2); weapon 0x18, 7.0 a hit). Their rounds reach Bond but the walker's `exposed_part` world trace never sees their hit boxes from the lane. First decide whether that asymmetry is a port bug: trace a guard's round path and Bond's return ray against the same geometry — if the guards shoot through something the game's bullets stop at, or their hit volume is wrong, fix the port [M the bullet/LOS tests; D the geometry]. If both are right, find cover in the lane (the desk block at x −14.6..−12.9 / z 31.3..32.9 is a lead) and give the walker that stand. Un-ignore the test. Run walkreal_07000013_live and quote the totals.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-lobby-rifles`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-11-02T03:46Z

---
## Report (done, 2026-11-02T05:21Z)

Closed by the driver: the report is on `s/level-13-lobby-rifles` (docs/reports/level-13-lobby-rifles.md); the worker didn't run finish.sh.
