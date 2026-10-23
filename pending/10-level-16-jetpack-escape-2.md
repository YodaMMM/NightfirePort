## Your task: `level-16-jetpack-escape-2`, 07000016: the real escape from the finale pit

The previous attempt made `walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission` pass by turning on the game's DEBUG fly mode (`debug_scanmode`, substate 5, toggled from a debug controller slot) through a new port-only F key. That is a debug cheat, not what the shipped game gives the player, and it is rejected. Find how a player actually survives the hurt-volume window (0x0600090a stream 6 ticks 1611..2170 [D]) in the shipped game: the script, the item or vehicle, the player state, the channel that moves Bond out [M][D]. Remove the F key and the debug fly path from the walk.

The previous attempt's findings are in the test's own `#[ignore = "..."]` message on master — read it first; it names the blocker exactly. Real input only (`step_bond`: movement/aim/fire/use; no teleports past the blocker, no channel or state pokes, no debug or cheat paths). Port what the game does at the blocker from the decompile [M] / the disc [D]. If a live number is needed, say exactly which address to read in PCSX2 on a fresh boot, and stop there rather than guess. Un-ignore the test when it passes, with a sister that can fail.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-16-jetpack-escape-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

