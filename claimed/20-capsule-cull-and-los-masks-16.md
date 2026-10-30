## Your task: `capsule-cull-and-los-masks-16`, one cylinder query, faithfully

Continue `origin/s/capsule-cull-and-los-masks-15` (merge `origin/master` first; read its report, "What's left"). The cull fixes two walks and breaks two others in every variant tried: the port's push arm and the game's single Intersect_CylGeom 0x109da0 query diverge structurally. Port the game's ONE cylinder query faithfully [M 0x109da0 whole fn; Collide_Intersect 0x1ebaa8] for the paths that use it, so a single cull rule serves all of them; then remove NF_CULL_UP / NF_NO_CULL. Land only with walkreal 02, 03, 08, 14, fix_npc_doors, npc_doors, mp_perf_16 and every touched binary green; quote each total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-16`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-30T14:03Z
