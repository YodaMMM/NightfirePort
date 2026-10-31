## Your task: `weapon-handling`, every player weapon handles like the game

For every player weapon and the firing gadgets, compare the port to the game and fix every divergence: fire interval, burst/auto, clip and reserve, reload time and rules, damage per body part, spread (standing, moving, crouched, zoomed), zoom levels and speed, aim-assist, projectile speed, switching time [M Player_WeaponFiring, the reload/switch handlers, Check_AutoAim 0x1a73f8, the zoom handler; D the weapon data table — work/weapon_data_live.bin is a live dump]. Do NOT change shell-casing ejection at all. Produce a game-vs-port table per weapon with citations; pin each weapon's effective behaviour (fire interval in ticks, clip, reload ticks, zoom steps) with a test reading the disc data, with a sister. Run nf-game, every *weapon*/*gun*/*fire*/*reload*/*zoom*/*aim* binary and the fighting walkreal_* binaries. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/weapon-handling`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

