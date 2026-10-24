## Your task: `level-0a-office-hatch-8`, the spawn settle's last red test, then the 0a hall and corner

Start from `origin/s/level-0a-office-hatch-7` (NOT merged; read its `docs/reports/level-0a-office-hatch-7.md`). The level-start spawn settle now leaves four of the five tests it broke green; one is still red on the branch and passes on master: `bathhouse_shout_mixer_live::the_bunker_trio_shout_reaches_the_mixer_with_the_ambients_live` ("shout id 115 reached the mixer silent").

1. Find why the bunker trio's shout reaches the mixer at zero volume with the settle and audible without it: the shout's 3-D position vs the listener (Bond's root/eye) on the first ticks, the distance/pan/volume rule the game applies [M — the 3-D sound volume path `Sound_Play3D` → the SFX driver's attenuation], and whether the settle's held ticks change the listener the sound is mixed against. Fix the cause [M]; never widen the settle or special-case the test.
2. The branch must have NO test red that is green on master. Then continue the hall glide, the corner and the office walk (`walkreal_0700000a_live`) as far as they go.
3. The full suite on the final tree.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-office-hatch-8`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


