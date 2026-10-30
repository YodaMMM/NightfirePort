## Your task: `capsule-cull-and-los-masks-15`, the second cull divergence and the last reds

Continue `origin/s/capsule-cull-and-los-masks-14` (merge `origin/master` first; read its report, "What's left" items 1-5). The bisect proved the head-on cull still moves the 07000003 captor-kill red (NF_NO_CULL=1 greens it; NF_NO_MASKS=1 does not). Root-cause that second divergence [M Intersect_CylGeom 0x109da0, Collide_Intersect 0x1ebaa8] and fix it; bisect walkreal_07000008's fusebox red the same way; capture and fix the mp_perf_16_live and npc_doors_live reds (both new on that tree). Then REMOVE the NF_NO_CULL / NF_NO_MASKS gates. Land only with every touched binary green and the walkreal binaries for 02, 03, 08, 14 green; quote each total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-15`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-30T11:18Z

---
## Report (done, 2026-10-30T12:25Z)


---
## Report from the worker (done, 2026-10-30T12:25Z)

Closed by the driver: the report is on `s/capsule-cull-and-los-masks-15` (docs/reports/capsule-cull-and-los-masks-15.md); the worker didn't run finish.sh.
