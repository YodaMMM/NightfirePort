## Your task: `mp-bot-backoff-2`, multiplayer bots probe 1.8 behind them before backing off

Branch from origin/s/mp-bot-backoff and merge origin/master; read docs/reports/mp-bot-backoff.md. A review found one mismatch: the CanBackoff probe is (0, 0, -1.8), not -0.75 [M NDrone2_CanBackoff 0x1501a0; disasm 0x1501f4 'lui r1,0xbfe6' = 0xbfe66666 = -1.8]. The branch's other findings stand: the bot speed-mul arm (+0xc5 == 0x1e) does not fire for bots, so the probe is UNSCALED; the anim-state-4 gate as data; the NDrone2_MoveTest port. Set the probe to 1.8 unscaled, fix can_backoff_probe_is_075_and_unscaled into a 1.8 pin citing the lui, re-pin mp_perf_16_live's digest once at the end, and run every mp_* and net_* binary one per tool call plus nf-game --lib.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/30-mp-bot-backoff-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox7 at 2026-11-12T05:29Z
