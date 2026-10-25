## Your task: `level-07-guest-collision-4`, guests solid to Bond — the walks that still break

Start from `origin/s/level-07-guest-collision-3` (merge it first; it carries rounds 1-2 and the undrawn-slide fix [M Drone_Control 0x139268:75-96]). Merged with origin/master (8de76e31a or later) these go red, each green on master, each re-run alone:
- `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact`
- `npc_doors_live::the_01_escort_driven_through_the_shoji_opens_it_and_crosses`
- `elevator_buttons_09_live::walk_operate_the_button_and_ride_to_0700000a`
- `interactables_0700000a_live::armour_pickup_taken_by_walking_over_it`
- `speech_facial_fallback_live::a_present_script_talker_plays_through_the_facial_and_advances`
- `probe_07_route::probe_the_route` (STUCK at the hall's north-west corner: bond (94.15, 44.13, -46.98), target (92.60, 43.60, -47.50))

Run the FULL suite on your merged tree before reporting (earlier rounds ran slices and missed these).

1. For each: which bot pushes Bond, where, with which flags, and does the game push Bond there [M — `NDrone2_DoCollision` 0x149ec8 gate per bot (dead/disabled/+0x538/near-player/obj+0xfc/+0x570&2), `Collide_Jointy` 0x1ec180:229-266]? Pay attention to escorts/allies (team 2, the 01 contact) and to drones the game has disabled or undrawn. Fix the port where it departs; where the game really pushes Bond, the walk takes the player's way around with real input.
2. Every test above green plus fix_guest_solid_live; the full suite total line in the report.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-guest-collision-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-10-25T10:24Z
