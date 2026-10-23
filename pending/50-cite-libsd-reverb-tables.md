## Your task: `cite-libsd-reverb-tables`, the reverb preset tables from the disc, not from a reimplementation

`crates/nf-audio/src/reverb.rs:416` (STUDIO_3 / mode 4: apf sizes, volumes, the 20 delay registers, IN_COEF 0x8000), `:459` and `:492` (ROOM spot-checks, the per-mode invariants, `EFFECT_SIZES[4] == 0x6FE`) take their values from ps2sdk's `effect.c`, a reimplementation.

1. Find the effect parameter table and the effect size table in the disc's LIBSD module, the one SFX.IRX's `sceSdSetEffectAttr` uses, and cite the bytes [D] (module, offset).
2. Compare every value the port uses. Where they differ, the disc wins: fix the port and the test together.
3. `PS2_SetupReverb` 0xc944:23 shows one stack word of 0x6fe0. Say whether the table agrees with it.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cite-libsd-reverb-tables`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

