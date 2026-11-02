## Your task: `door-scan-skip-bit-2`, the door scan skip bit without breaking the escort

Continue `origin/s/door-scan-skip-bit` (merge `origin/master` first). On master it turns `fix_npc_doors_live::the_escort_opens_the_interior_doors_on_his_route` and `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage` red (the escort's interior shoji doors no longer open for him). Find what the game's door scan skips and what it still lets an ally open [M the door scan / Door_Update 0x1363c8 / the ally door-open path], fix the port so both the skip bit and the escort's doors behave as the game does, and keep both tests green without weakening an assertion. Run fix_npc_doors_live, escort_*, every *07000001* binary, every interactables_* and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/door-scan-skip-bit-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-02T13:44Z
