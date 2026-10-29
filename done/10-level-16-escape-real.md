## Your task: `level-16-escape-real`, Countdown Part 3: how the game really gets Bond out of the launch pit

The level-16-jetpack-flight branch (`origin/s/level-16-jetpack-flight`, NOT merged) ported `Player_SSScanMode` 0x193098's fly arm - but that is the game's DEBUG scan mode (`debug_scanmode`, toggled by action 39 that no shipped controller sends [M]): a free-fly no player can reach. It is not the escape and must not ship as a player control. Find what the shipped game actually does after the cinema re-enables Bond at the pit floor (3.57, -23.18, -179.55) inside HurtPoly-ShuttleLaunch #309's stack: the jetpack/escape mechanism a player uses [M the substates, scripts and triggers that move the player on 07000016; D the level's placements around the pit - ladders, lifts, wires, script players; L a fresh-boot PCSX2 run of the escape if needed]. Port that, and walk the escape on real input in `walkreal_07000016_live` / `fix_finale_live` / `kiko_button_real_walk_live` (reuse the -jetpack-flight branch's test legs where they still apply, but no scan-mode toggle). Do not add a debug fly control. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-16-escape-real`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-29T23:03Z

---
## Report (done, 2026-10-29T23:42Z)

