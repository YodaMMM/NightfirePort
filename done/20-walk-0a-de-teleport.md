## Your task: `walk-0a-de-teleport`, 0700000a's walk reaches mission end on real input with no position writes

Branch from current origin/master (walks-replay-proof is merged) and read docs/reports/walks-replay-proof.md on origin/s/walks-replay-proof, "What's left" item 2. The 0700000a walk still has 3 position writes (teleports). Replace each with real input: route the leg through the nav web, crouch/jump/use as a player would, and take the level's own doors and lifts. List each write you remove with the leg it served. Then record the walk and add the replay assert the same way the 0700000b walk does (record -> headless replay -> the recorded end state), with the recording committed. Helpers stay gated to 0700000a and the difficulty, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. If a write cannot be replaced yet, keep it, and write down why and the next lever in the test's doc comment. Run the whole walkreal_0700000a_live binary and interactables_0700000a_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/walk-0a-de-teleport`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: main agent at 2026-11-12T10:13:10Z

---
## Report (done by the main agent)
On master bb5e63e94: the 0a Operative real-damage walk is pure recorded input (Q-Worm shot + use presses through the walker step, keypad codes typed on the pad, the #648 stand walked); tests/replays/0700000a_operative.bin + replay_parity_0700000a (no divergence over 14038 ticks, hatch exit 0700000b).
