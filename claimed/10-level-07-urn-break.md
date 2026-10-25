## Your task: `level-07-urn-break`, shooting the urn starts its break script (and the other bullet-hit script players)

Placement #312 on 07000007 (`OBJECT_UrnBreak`, script 0x0600007A, mode 64, mask 2) is a script player that should start when shot. `SP_Hit` 0x1dc470 returns a flag byte to `SP_Update` 0x1dbda8; its bit 2 is set when a class-5 (bullet) HITDATA record sits on one of the current script's stream objects' +0xD0 chains — the `SP+0x37 == 0` arm filters with `Collide_FilterBullets(rec+0xD0, 0x106)` first [M 0x1dc470:30-34, :62-71]. The port's `Scripts::held_hit_targets` only returns players with hit_points != 0 and a held script, so the urn (hit_points 0) never enters the bullet pass's target set; bit 2 is raised only by the blast proxy. A gunshot cannot start the urn's break script.

1. Port the bullet arm of SP_Hit for every mask-bit-2 script player [M]: the current stream objects as bullet targets (primed or running), the hit recorded by the player-bullet pass and consumed by the SP update with the game's one-frame order (Collide after the script walk), the +0x37 filter arm if any placement uses it. Census the disc for mask-bit-2 SPs [D] (expected: 07000007 #312, 07000012 #660 'Sky bridge' with 120 hit points, 07000006 #807 'Object_Radio') and check each reacts as `SP_Update`'s state machine says.
2. Real-input tests: shooting the urn starts its script (state 1 -> 2); a sister where an unshot urn stays primed; the 12 sky bridge and the 06 radio react per [M]. Every walk on those levels stays green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-urn-break`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-10-25T08:23Z
