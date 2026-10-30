## Your task: `level-0d-crane-transit`, 0700000d the crane route into the ST-3 yard

`crates/nf-view/tests/walkreal_0700000d_live.rs` reaches the ST-3 yard through a documented position stand-in: the game's way in is the crane sequence (level hint "Use the cranes to get into building ST-3" [D USATxt 0x4000064]; the cranes are 0xE2 SimpleScripts - `shipping_handle` #112, the script players #115/#492, the crane buttons #113/#114 animating on ch 0x2b), which the port does not simulate [docs/levels/0700000d.md §6]. Find what the game does: which input starts each crane, how the 0xE2 SimpleScript moves its load, and how the player gets carried or reaches the yard [M the 0xE2 create/update functions, D the scripts]. Port it so the player can do the crane route on real input, then replace the stand-in in the walk with that route (and remove the `set_channel` of 0x1b/0x1c at line ~38 if the crane path is what raises them in the game). The same 0xE2 family carries 0c's vent/pulley stand-ins: say in the report whether your port covers them too. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-crane-transit`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

