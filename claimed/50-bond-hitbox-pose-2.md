## Your task: `bond-hitbox-pose-2`, crouched hit boxes without changing the view height

Start from `origin/s/bond-hitbox-pose` (merge it, then origin/master). Its hit boxes on every level and the crouch pose are right, but the crouch toggle now moves `bond.pos[1]` down by ROOT_HEIGHT - CROUCH_ROOT_HEIGHT (0.4847) instantly, on top of the existing eased `crouch_drop` (0..0.45) that `Bond::head()` already subtracts. The camera reads `head()`, so a crouch would lower the view about 0.93 in one tick instead of easing 0.45: the on-screen crouch height and feel change. Keep the player's view exactly as on origin/master (same head() height every tick of a crouch and a stand-up; add a test that pins head() per tick against master's values), and still pose the hit boxes crouched (pass the crouch root offset to `bond_boxes` instead of moving `pos`, or fold the drop into one eased quantity so head() is unchanged). Keep the fix_bond_damage_live, anim_events_live and bond_boxes_every_level_live results, and the mp_perf_16_live digest pin must not need a re-pin from a view-only change.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/bond-hitbox-pose-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-29T13:50Z
