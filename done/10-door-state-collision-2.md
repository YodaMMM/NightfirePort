## Your task: `door-state-collision-2`, open doors stop blocking without breaking the walks

Start from `origin/s/door-state-collision` (the `Door_IsClosed` 0x1367b0 filter on the doors' collision parts; merge origin/master). On the full suite it (carried in the capsule-cull branch) turned these red: walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact`, probe_07_route `probe_the_route` (Bond STUCK), walkreal_07000014_live `the_east_duct_route_reaches_the_stairwell_slot`, ai_live `guard_takes_cover_like_the_game`. For each, find what the walker/guard now runs into or through (a door that is open in the port but the game's route still treats as an obstacle, a CLOSING leaf the game keeps solid for a few ticks, the doors' group states, cover points that used the door leaf as cover) and decide from the game's rule [M Door_IsClosed 0x1367b0, the zone consumer 0x135738/0x1336f8, the drone cover pick] — fix the port, or re-derive a test's pin from the game's behaviour with the cause cited. All four green plus walkreal_07000008_live 4/4 and every other walkreal / door / mp test on the full suite.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/door-state-collision-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-10-27T04:39Z

---
## Report (done, 2026-10-27T06:19Z)

