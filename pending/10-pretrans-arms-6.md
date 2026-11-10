## Your task: `pretrans-arms-6`, 07000001 walks with the contact to the exit with the game's guard aim

Branch from origin/s/pretrans-arms-5, merge current origin/master, and read its docs/reports/pretrans-arms-5.md, item 3 of "Done". The aiming-guard cone fix is the game's and stays [M 0x178ae0:34-36]. With it, three 07000001 tests that share the descent walk are red: walk_from_the_spawn_to_the_exit_with_the_contact, its _real_damage sister, and the_exit_fade_starts_on_the_exit_tick_runs_the_games_300_frames_then_the_level_loads. The contact stalls at route 29/37 with two armed guards parked on him, and the escort dies during the descent follow. Take the ledger's lever: the descent recovery's forward clear band, clearing the guards ahead of the contact before he reaches them. The plate-side clear loop is a measured dead end; do not repeat it. Helpers stay gated to 07000001 and the difficulty, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000001_live binary, interactables_07000001_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/pretrans-arms-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

