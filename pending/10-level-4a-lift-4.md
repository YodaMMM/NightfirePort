## Your task: `level-4a-lift-4`, 0700004a: cross from the stopped cab onto the landing, and the shaft kill's real trigger

On master the lift walk runs green to the cab's north edge (z -1.85) at the stop (root -54.1971, the 575-tick landing-scene disable, the raw-key hand-back), and staying on ends in the death at root -95.80. Two parts are open:
- The crossing: the landing's south edge is at z ~-1.2 (floor -56), a ~0.65 gap. The port's walk speed is ~0.037 units/tick and a jump's airborne momentum decays within ~20 ticks, so the walker cannot cross before the resumed cab drops. The real game's walk-off works.
- The kill at -95.80 is a level-specific [S] zone in drone.rs (no disc writer for ch 0x62, no static collision there, no script kill found).
The probe tests (walkreal_probe_l3*, ignored) are assertion-free dumps.

1. The crossing: compare the port's on-foot walk speed and jump arc with the game's [M Player_Update / the walk and jump handlers in player.rs's citations]; fix whichever differs, then extend the walk across onto the landing and into the lobby, real input only.
2. The kill: find what ends the player in the shaft [M: Player_HandlePain's landing-damage formula 0x1949a8:113-124 on a catch below, Mission_Update's ch-0x62 arm 0x182c60:38-41, any collide/trigger class that kills]. Replace the [S] zone with it, or keep it [S] and name the PCSX2 read (fresh boot, watch BLData+0x894 and ch 0x62 through y -95.80..-96).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-lift-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

