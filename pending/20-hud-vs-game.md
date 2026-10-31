## Your task: `hud-vs-game`, the in-game HUD like the original

Continue `origin/s/wip-fix-hud` (merge `origin/master` first). Verify every HUD element against the decompile and fix divergences: the iris health/armour ring, ammo counter and fire-mode text, weapon/gadget icons and switch popup, the crosshair and its on-target change, the radar, the NEW OBJECTIVE box (font, timing, fade), the hand-over-hand/ladder/zip icons and grip bar, damage direction and the red overlay, zoom overlays, decryptor/Q-worm progress, pickup messages — positions in the 512x448 frame, sizes, colours, timings, show/hide rules [M the HUD draw functions; D HUD textures/fonts]. Render before/after headless shots; pin the element list/geometry with tests (sisters). Run every hud/menu/render binary. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/hud-vs-game`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

