## Your task: `level-0b-every-objective-3`, 0700000b exec floor walked on real input

`crates/nf-view/tests/walkreal_0700000b_live.rs` still carries position stand-ins on the exec floor after the lift: the two `bond.pos` "transit" writes (the Q-Worm C desk and the terminal room), the terminal-room fallback placement into plate #163 when the walk wedges, and the keycard lock whose ch 9 is driven with `set_channel(9)` when the walked E approach fails (the boardroom table blocks the eye ray). Replace each with a real `step_bond` walk and a real E press. The previous session found the exec-floor nav routes drop Bond into the dome hole's fail web when the walker wanders: first add a probe test that walks every exec-floor nav route and dumps where each one dies, then re-attach the walks with short per-leg caps. Where a walk cannot pass, find what the game does there [M collision / Switch_Activate 0x1b02a8 use reach / D the furniture placements] and fix the port; if the game's own geometry really blocks it, pin that with evidence and keep the stand-in documented. Assertions never rest on a missing writer. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0b-every-objective-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-30T02:52Z

---
## Report (done, 2026-10-30T03:35Z)

Closed by the driver: the report is on `s/level-0b-every-objective-3` (docs/reports/level-0b-every-objective-3.md); the worker didn't run finish.sh.
