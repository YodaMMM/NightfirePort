## Your task: `markers-drone-b`, guard behaviour the port guessed is settled against the game (drone.rs, middle third)

Every `[S]` (unverified) marker in crates/nf-game/src/drone.rs from line 12000 to line 18000 (21 markers today) on current master is behaviour the port guessed. For each one, read the game code it stands in for, with addresses, and end it one of three ways: port the game's behaviour and remove the marker; prove the port already equals the game, with the evidence ([M], [D] or a fresh-boot [L]) in place of the marker; or prove the path is never reached on the shipped disc. List every marker in the report with its line, what it guessed, the game's answer and which of the three it became. Where a marker cannot be settled without a live read, keep it and list the exact read under the report's live checks for the main agent. Rand draws must keep the game's count. Where a change alters play, pin it through real input on a level where it happens, and repair any walk it shifts, gated to that level, never lowering damage or health. Run the binaries that touch the changed code (grep tests for the functions), every walkreal_*_live one level per tool call if guard or player behaviour changes, and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/markers-drone-b`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-08T13:55Z

---
## Report (done, 2026-11-08T14:50Z)

