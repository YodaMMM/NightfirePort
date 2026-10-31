## Your task: `level-16-finale-shortcuts`, 16: the last test shortcuts after the finale cinema

origin/s/fix-16-finale made `walk_the_chasers_defeat_completes_the_mission` pass on real input (the pit blast-room shelter). Two older tests, `fix_finale_live` and `kiko_button_real_walk_live`, still move Bond to the upper floor after the second cinema (0x06000632) as a shortcut, and the latter's header wrongly says nothing on the disc clears the button lock (the clear is at t1056 in the 07900016 pack). Replace the moves with what the cinema does (it drops Bond beside the button at t1057 [D]) or real input, fix the header, and keep all 16 tests green. Continue from origin/s/fix-16-finale (merge origin/master first). Run walkreal_07000016_live, interactables_07000016_live, fix_finale_live, kiko_button_real_walk_live, blast_room_shelter_16_live. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-16-finale-shortcuts`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

