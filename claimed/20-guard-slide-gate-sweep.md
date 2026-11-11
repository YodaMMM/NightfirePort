## Your task: `guard-slide-gate-sweep`, every walk stays green with the guard slide rule on every level

Branch from origin/s/guard-slide-gate-all-levels, merge current origin/master, and read its docs/reports/guard-slide-gate-all-levels.md, "What is left", items 1 to 3. The slide law is the game's on every level and stays. First the 07000001 Agent repair it ledgered, gated to 07000001 and difficulty 2. Then the rest of the sweep, one binary per tool call: walkreal_07000002, 03, 04, 09, 0a, 0b, 0d, 13, 14, 15, 16, 1b and 41_live, the 0c eastroute and barrelrow sisters, and the 07000011 route sister. Repair any walk that shifts, gated to its level and difficulty. Then the binaries the drone.rs change obligates: the nf-game lib tests, ai_live, guards_0c_live, npc_doors_live, fix_guard_misc_live, and the ally and guard stuck sweeps. Real input only, no top-ups, never lower damage or health.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-slide-gate-sweep`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-11T09:32Z
