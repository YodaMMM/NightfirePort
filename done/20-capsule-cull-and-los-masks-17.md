## Your task: `capsule-cull-and-los-masks-17`, the feet-band port that the faithful cull needs

Continue `origin/s/capsule-cull-and-los-masks-16` (merge `origin/master` first; read its report). The faithful cull (raw-denom compare, disasm 0x10a1f0-0x10a2e0) landed and greens 02/03/08/fix_npc_doors/npc_doors/mp_perf_16, but walkreal_07000014 is 3/3 and walkreal_07000001 2/3: under the faithful rule the walk spine's cull is inert and the game's underfoot support is the feet ray [M Player_FeetOnPoint 0x194188], which the port only partly models. Port the feet-band from the decompile (cite lines), then make walkreal 01 and 14 green with 02/03/08/0a and the rest unchanged. Run every walkreal_* binary; quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-17`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-31T01:10Z

---
## Report (done, 2026-10-31T02:17Z)

