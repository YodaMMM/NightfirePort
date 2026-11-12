## Your task: `level-0b-files-plate-hazard`, the 0b real-damage walks on real input to the end, then a real-app replay (FIRST PRIORITY: blocks the 0.1.38 release)

Start from origin/s/walks-0b-real-wip merged with origin/master (master now has replay format v3: skip + options; walkers record through one step site). On that branch the 0700000b walks have NO position/keypad/gun writes: the duct route for every walk, the keypad typed on the page (keypad_nav/keypad_confirm), weapon swaps by switch_to, the files plate faced in yaw+pitch and pressed from the sprint's stop. The plain walk is green; the three real-damage walks (Operative, Agent, 00 Agent) die at the files plate to "world" damage: -2.40 every 7 frames, owner usize::MAX, from tick ~13308 at (-1.1, 47.9, -3.4) entering the terminal room (or ~13682 at the plate itself when the old 0x4e keypad fire is put back - so not that), fusebox_hurts 0, no live explosions, alarm ch 0x61 off, Bond holding the Q-Worm 0x58 with clip 0. The usize::MAX callers left: `run_beam_explodes` (pending_beam_blasts, drone.rs ~18475), `spawn_blast`, `blast_mines`. The old walk (master) placed Bond at the plate stances and never took this damage.
Do: an instrumented run at tick 13300-13320 dumping the pending blast/beam queues, mines and script blast spawners near the plate (-1.15, 48.36, -13.89); name the object and what arms it; check against the decompile whether a player standing there takes the same hits. If it is the game's, route the walk the way a player avoids it (on real input). Then all four 0b walks green, record the Operative real-damage walk (the walker's recorder), replay it headless (replay_headless, same per-tick state, same mission end) and in the app (`nf-view <iso> 0700000b.bin --difficulty 1 --replay <file>`) to mission complete; commit the headless replay check as a test.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-level-0b-files-plate-hazard`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox5 at 2026-11-12T04:33Z
