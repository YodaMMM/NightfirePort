## Your task: `level-0a-office-hatch-11`, the game's wall-stop, then the 0a hall, corner and office walk

Read `docs/reports/level-0a-office-hatch-10.md` on `origin/s/level-0a-office-hatch-10` (merged; the report itself is not on master). It settled that HITTEST+0x50 is a dead accumulator, so the cylinder tail's overlap gate is not what lets the game glide the 0a doorway line — and that removing the port's wall push breaks the 01/02/03 walks: the port's push is load-bearing as the wall-stop. The requirement: a stopper that keeps every walk green but never fires on the 0a doorway/corner lines ([L] the game glides the z 16.29 line 0.011 inside the z 16.279 plane; corner exit 1.127 -> 1.120).

1. Find the game's wall-stop [M]. Candidates from that report, in its order: the obj+0xdc flag word's velocity-side use (`Player_CollisionHandler` 0x1949a8:154-165 — bits 0x10/0x20 from the first hit's hitpoint vs BL+0x760, bit 0x8 zeroing BL+0x50 when grounded); the `Collide_Jointy` 0x1ec180 path; the tail's radius argument (`param_1[8]+0xc`). Port the one the decompile shows, with tests for both requirements (the 01/02/03 walks green, the 0a glide/corner pins).
2. Un-ignore `the_west_hall_glide_height_matches_the_game`, `the_corner_crosses_like_the_game`, then `walk_office_floor_qworm_keyboard_hatch` as they go green. The full suite on the final tree.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-office-hatch-11`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


