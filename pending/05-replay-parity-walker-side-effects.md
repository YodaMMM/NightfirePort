## Your task: `replay-parity-walker-side-effects`, a walk's own inputs replay to the same result (FIRST PRIORITY)

Branch from origin/master. A recording rig showed that the 07000004 full-mission walk finishes at tick 9300 with Bond on 44 health, but replaying exactly the same 9300 inputs (identical in memory and on disk) from a fresh load kills Bond at frame 5976, both headless and in the real app. The sim is deterministic (two replays agree). So the test walker changes the game in a way that is not input. Rig: wrap every step_bond in crates/nf-view/tests/common/walker.rs so it records nf_view::replay::TickInput::from_parts(...) and feeds to_parts() to the sim; record walkreal_07000004's full-mission route (difficulty 1, top_up_health false, skip_cutscenes false), save the .nfr, replay it from nf_view::load, compare Bond position/health per tick. First divergence is between frames 1200 and 2000. Find the side effect: suspect read-only world queries the walker makes (segment, line_of_sight*, nav probes, find_cel) mutating CollisionWorld or sim caches (RwLock/Mutex fields such as last_objects, hurt_hits, scope or aim caches), or any helper that writes state. Fix it so queries have no side effect (a port bug if the GAME path can trigger it too), and add a committed test: record a walk, replay its inputs, same end state. Then run the full suite.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/05-replay-parity-walker-side-effects`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
