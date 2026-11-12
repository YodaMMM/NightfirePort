## Your task: `objective-banner-part-load`, a mission's later part opens with its own NEW OBJECTIVE, not a stray OBJECTIVE COMPLETE

Loading a mission's later part directly (the same load in both: PCSX2 tools/trace/goto_level.py <level>, the port's load_with_carry_difficulty(.., carry None, 1)) the game opens play with "NEW OBJECTIVE / <the part's objective>" while the port flashes "OBJECTIVE COMPLETE" first [L PCSX2 frames vs port frames, same moments]:
- 07000004: game "NEW OBJECTIVE / Defeat the assassin"; port "OBJECTIVE COMPLETE" for 3 s.
- 0700000d: game "NEW OBJECTIVE"; port "OBJECTIVE COMPLETE".
- 07000013: game nothing then "NEW OBJECTIVE / Escape from the tower"; port "OBJECTIVE COMPLETE" then "NEW OBJECTIVE".
- 07000001 (a mission's FIRST part): both "NEW OBJECTIVE / Escort Mayhew to bunker" - correct.

Port state on 07000004 at load (Mission.states per LEVELS record): [[0],[0],[0,0],[0,0],[4],[4,4,4],[3,3,3],[1],...]; on the first play tick (t 1330) record 6 objective 2 goes 3 -> 4 ("OBJECTIVE COMPLETE") and record 7 objective 0 goes 1 -> 3 ("Defeat the assassin"). So the port leaves the PREVIOUS part's objectives announced-but-open and completes one on entry. Find what the game does with the earlier records at a part's load (Mission init / Mission_Update 0x1827d8's record walk, the `ThisOrderNum <= record.part` gate, the save's carried states - read the LEVELS record states live in PCSX2 right after goto_level 07000004), port it, and check the real path too (walk 07000003 out into 07000004 with carry: which banners show). Pin with tests for 04/0d/13 (first banner after the load) and 01 (unchanged).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-objective-banner-part-load`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
