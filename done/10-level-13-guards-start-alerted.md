## Your task: `level-13-guards-start-alerted`, Phoenix Fire Part 4's guards start unaware

On Phoenix Fire Part 4 (07000013) the lobby guards are already alerted when the level starts, before Bond has been seen. Find each guard's start state on the game's path [M Drone_Create / the drone's initial AI state and alert fields, the level's start scripts and channels that may alert them, any SP/script that sets the alarm at load] [D the placements' AI params] and port what the game does, so they start in their idle/patrol state and only alert through the game's own detection (sight, sound, alarms). Test from a fresh level load on real input: at tick 0 and after N ticks with Bond hidden, every guard's alert state matches the game's rule [M]; then the first sighting alerts them as the game does.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-guards-start-alerted`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-27T14:35Z

---
## Report (done, 2026-10-27T16:31Z)

