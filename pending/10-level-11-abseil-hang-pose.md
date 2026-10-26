## Your task: `level-11-abseil-hang-pose`, the hanging abseiler plays the hang pose

On 07000011 a roping guard in AbseilHang (0x90) still plays danim 11 — the AbseilInit rope-grab (`SetDAnimNow(0xb)` [M 0x169610 case 1]) — because the Hang ENTER's `CallAnim(2)` (the hang pose [M AbseilHang 0x169940 case 1]) never starts in the port (level-11-rope-guard-hitbox's probe: danim 11 / script 100665490 persists while state = 0x90). Find what the game's CallAnim path does that the port's `call_anim` fold drops (the anim-set lookup, the queued-vs-now rule, a gate), fix it so every abseil state plays its own animation (Init 0xb, Slide CallAnim(3), Hang CallAnim(2), the death 0x92) [M], and finish that branch's live kept-casing pin (`a_kept_casing_from_the_rope_guard_settles_on_the_floor_below`, ignored: the guard's weapon's casing mesh census). Tests: per abseil state, the playing anim [M]; the hanging guard's pose differs from the slide's.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-abseil-hang-pose`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

