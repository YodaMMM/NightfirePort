## Your task: `stuck-patrol-0c-0d`, two patrol guards that never move on 0700000c and 0700000d

The stuck-guard sweep (`crates/nf-view/tests/guard_stuck_sweep_live.rs`) finds two patrol movers frozen from the level start:
- 0700000c drone 15, PATROL (0x07), stops at (9.30, 12.24, -220.82), 4.7 units from its home (6.55, 12.24, -224.56); nearest door 32.4 units away.
- 0700000d drone 33, PATROL (0x07), parked at its home (191.08, 13.66, -232.19) with a ~0.09 z jitter for the whole run; nearest door 44.0 units away.

1. Trace each drone's patrol: its path record and node chain [D], the next node it is heading for, and the query that stops it (route follow BLOCKED / NO_PATH, the mover's collision response, an unreachable node) [M NDrone2_ReachedDestNode 0x153358 and the patrol functions].
2. Fix the cause in the shared code (never a per-level patch) and turn the two sweep rows into assertions: each guard walks its patrol. Sister: a guard whose patrol path is genuinely blocked in the game still stops.
3. If a live number is needed, say exactly which drone field to read in PCSX2 on a fresh boot.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/stuck-patrol-0c-0d`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

