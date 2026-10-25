## Your task: `level-11-stairwell-bullet-wall-2`, bullets pass the stairwell fudge without breaking other shots

Start from `origin/s/level-11-stairwell-bullet-wall` (merge it first; its report has the chain: Bullet_update 0x12e7b0 re-arms the HITTEST masks per tick, Collide_Update 0x1ea398:216-229 skips pass-flagged contacts, the 07000011 fudge map43#126 surfaces 0x40). Merged with origin/master (34ac31a56) these go red, each green on master, each re-run alone:
- `gondola_windows_live::firing_at_a_ride_window_breaks_it_dead_bit_glass_sound_entity_gone` (08)
- `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact`
- `escort_follow_07000001_live::escort_at_walking_speed_keeps_the_games_distance_and_the_exit_opens`

Run the FULL suite on your merged tree before reporting (the last round ran slices and missed these).

1. For each, find which contact the new pass rule now skips (or newly stops on): the gondola window (glass class 0x10 + the surface mask 0x20 arm — does the round now pass the glass it should break? [M Collide_Update's glass arm, the window's damage path]); on 01, which shot now reaches or misses whom (the contact, the corridor guards). Fix the port where it departs from [M]; where a walk depended on the old wrong blocking, play it the player's way with real input.
2. All three green plus fix_foliage_bullets_live and the stairwell test; the full suite total line in the report.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-stairwell-bullet-wall-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox6 at 2026-10-25T10:10Z
