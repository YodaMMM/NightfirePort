## Your task: `level-07-guest-faces`, 07000007: guests' lips and faces move when they talk to Bond

On master the party guests now speak their BondTalk lines through the facial path [M NDrone2_SpeechSFX 0x141b00, NDrone2_BondTalk 0x1446b0], but in real play their faces do not move. Find the game's facial animation for these lines [M — AnimFacialAppend, the +0x53c attach, the facial script's anim ops, the morph/bone channels the facial anim drives] and what the port does with it (the GPU facial morph path exists: `upload_morph_weights`, the facial tests facial2/facial3/facial_gpu). Port what is missing so the talking guest's face animates with the line; a test on the morph weights / bone channels during a guest line, plus headless renders of a talking guest named in the report.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-guest-faces`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
