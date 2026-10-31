## Your task: `goggles`, the night-vision and x-ray modes

The goggles' view modes are missing: the guard body-glow sprites (x-ray mode, viewer+0x236 == 2, within 10 units [M BodyGlow_Update 0x1ac2c8:140-200]) and the night-sight mode (viewer+0x236 == 1; sprite 0x2000131) are not drawn because the goggles are not ported. Implement them like the game: which levels carry them [D the level kits, Player_InitWeapon 0x1a7ef8], how the player switches them on/off and cycles modes, the viewer+0x236 state, each mode's full-screen look (tint/brightness/noise; what x-ray draws and how), the HUD overlay and sounds, and the draws gated on the mode. Add a render aid to set the mode headless if none exists; render before/after per mode; tests (mode switching, gated draws, post-effect parameters) with sisters. Run every render/gadget/hud/weapon binary and load_scene_equality/digest. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/goggles`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

