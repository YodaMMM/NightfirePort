## Your task: `level-07-guest-collision-2`, guests solid to Bond without breaking three walks

Start from `origin/s/level-07-guest-collision` (merge it first; docs/reports/level-07-guest-collision.md has the [M] chain: `Collide_Jointy` 0x1ec180's 0x800 arm pushes Bond off every capsule-ON bot in every stance, the bot not pushed in SP; the guests' capsule from `NDrone2_DoCollision` 0x149ec8 + `NDrone2_Collision` 0x140e90). Merged with origin/master, these go red (each green on master):
- `walkreal_07000007_live::walk_the_party_start_to_finish`
- `interactables_0700000a_live::armour_pickup_taken_by_walking_over_it`
- `speech_facial_fallback_live::a_present_script_talker_never_plays_nor_advances`

1. For each, find what the push changes on its path (which bot's capsule meets Bond, where, with which flags) and whether the game pushes Bond there [M — the DoCollision gate per bot: dead/disabled/+0x538/near-player/obj+0xfc/+0x570&2, the class 0xc radius]. Fix the port where it departs from the game; where the game really pushes Bond, re-route the walk with real input (never weaken an assert).
2. All three green plus fix_guest_solid_live; run the full suite on the final tree merged with origin/master and report the total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-guest-collision-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-10-25T03:41Z

---
## Report (done, 2026-10-25T05:19Z)

Closed by the driver: the report is on `s/level-07-guest-collision-2` (docs/reports/level-07-guest-collision-2.md); the worker didn't run finish.sh.
