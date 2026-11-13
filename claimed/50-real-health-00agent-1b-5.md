## Your task: `real-health-00agent-1b-5`, 0700001b's console phase can be survived on 00 Agent with real health

Branch from origin/s/real-health-00agent-16-1b-4, merge current origin/master, and read its docs/reports/real-health-00agent-16-1b-4.md, item 1 of "What's left". First re-derive the real branch-read cadence from the decode it found, and fix the determinism divergence it names: test it red on the old code, and keep the Rand count. Then re-balance the per-console caps against the real read times, keeping the lane kill shape at the 300 cycle with the fights fenced out of the hold chains. Then the Drake phase's spawn-band re-weight. The 16 half of that report is not this task. Helpers stay gated to 0700001b and difficulty 3, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_0700001b_live binary, interactables_0700001b_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-1b-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
Main agent, fresh deaths on master 729f9694e (each ignored walk of the level, --ignored --test-threads=1; test names not captured):
walkreal_0700001b_live: Bond died at tick 11385 (frame 11385) at (46.01,-14.08,-3.00) armour 0.0
walkreal_0700001b_live: Bond died at tick 5619 (frame 5619) at (31.92,-1.68,-0.59) armour 0.0

---
claimed-by: sandbox4 at 2026-11-13T04:33Z
