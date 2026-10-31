## Your task: `capsule-cull-and-los-masks-20`, land the capsule rules: the last red

Continue `origin/s/capsule-cull-and-los-masks-19` (merge `origin/master` first; read its report in `blocked/20-capsule-cull-and-los-masks-19.md`). 12 of the 13 reds are fixed. Its live check 1 is settled: the f-clamp in Intersect_CylGeom is [0, 1] — f23 = 0.0 (`mtc1 r0,f23` at 0x10a01c), f20 = 1.0 (0x10a024/0x10a028), untouched before the clamp at 0x10a22c-0x10a274 [M disasm] — so the flags-99 query point p = A + AB*f clamped [0,1] is the game's and needs no re-keying. A fresh-boot live run [L 2026-10-31] puts the STANDING body's stop in the 0c shaft pocket approach at z -111.39 (from (5.6, 12.9, -111.2) facing south); the crouch stop is not measured. So `walkreal_0700000c_live`'s :308 pin (the crouch stop at -111.813, derived from the old port behaviour, not a live measurement) is not game truth: re-derive that leg from the faithful rule (check the standing stop matches -111.39 in the port as a sister), and settle :775 (the #283 wire ride never releases at the ring SE corner) from the wire class [M Player_CollWire 0x197130 / the wire end test] with evidence. Land with all 13 and every walkreal_* binary green; push after each step; quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-20`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-31T14:21Z
