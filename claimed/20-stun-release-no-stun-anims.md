## Your task: `stun-release-no-stun-anims`, stun-grenade guards whose anim set lacks the stun clips never get up

`crates/nf-view/tests/fix_grenade_types_live.rs` (~line 204) is #[ignore]d (107a). The stun arm works: a placed guard goes to state 0x4d with weapon 1 and stun_until = frame + 3600, and the TIMER_C pass fires. But the release chain stalls in 0x4d for guards whose anim set has no 0x2d/0x2e stun anims. Guard 0 reaches the Loop; the chosen guard never leaves Impact. Read DroneAnim_CallAnim's no-datum fallback in the decompile: what the game does when the requested anim is missing from the set (skip, substitute, or end the clip at once) [M]. Port it so these guards finish the stun and recover like the game. Check the other states that call stun or dart anims for the same missing-anim stall (stun dart, stunner), and fix the whole class. Un-ignore the test. Run fix_grenade_types_live, anim_events_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/stun-release-no-stun-anims`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-03T02:53Z
