## Your task: `cite-take-ray-tolerance`, the pickup take ray's 1e-3 end band needs its decompile line

`crates/nf-game/src/pickup.rs:810` widens the take ray by 1e-3 at its end and says the game's VU0 fixed-point triangle test absorbs a sub-quantum 2^-14 contact (0x18e148's fall rest). No line is cited and the tolerance is not shown.

1. Find the Collide_LineOfSight / triangle-test lines that give the tolerance (or the fixed-point quantisation) and cite them [M address:lines].
2. If the decompile shows a different band, or none, fix the code and its test to it.
3. If it cannot be shown, mark the band `// [S] unverified` and name the PCSX2 read that would settle it (a resting drop at that contact depth on a fresh boot).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cite-take-ray-tolerance`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-24T13:24Z
