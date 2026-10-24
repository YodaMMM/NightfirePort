## Your task: `level-4a-lift-walk-5`, 0700004a: the spawn-to-lobby lift walk, un-ignored

`walkreal_0700004a_live::ride_the_lift_from_the_spawn_and_walk_into_the_lobby` is ignored on the post-enable hand-back: Bond was re-enabled at the base-composed marker (1.799, -107.662, -6.830), off the platform, and fell into the void. Master now re-enables him at the landing scene's raw marker key (the hash-0 kind-0x19 marker rule in `crates/nf-game/src/nis.rs`, pinned by `fix_landing_matrix_live`), and `walkreal_0700004a_satchel_live` rides the same lift to the lobby with the satchels burned.

1. Re-run the ignored walk on master. If it passes, un-ignore it and remove its fall exemption if it is no longer needed (check that the walk would fail if Bond fell — no vacuous pass).
2. If it still fails, find where and why with real input and fix it [M]; the satchels now drop on this ride (the shooters at ch 0x22 / ch 0x4f), so the walk must burn them with the laser watch or survive them the way the satchel test does.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-lift-walk-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox5 at 2026-10-24T15:37Z
