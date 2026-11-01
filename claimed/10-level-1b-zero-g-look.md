## Your task: `level-1b-zero-g-look`, 1b: zero-g look that pitches at every heading AND keeps the aim with the view

The body-frame zero-g look (`obj := R·obj`, [M Concat 0x1e4a90:13-49, Player_ZeroG 0x192ab0:162-164]; commit c3ed2d764, reverted on master) let the view pitch at every heading, but in play the aim no longer followed the camera/mouse as expected. The reverted world-frame look barely pitches facing east/west, so `walkreal_0700001b_live::walk_arrival_to_drakes_defeat` (ignored) cannot shoot the consoles. Find exactly how the game maps the look inputs to the view AND the aim in zero-g [M Player_ZeroG 0x192ab0 :74-164, the view/camera build for substates 8/9, the shot direction Player_WeaponInitBullet 0x1a4bf0 / auto-aim in zero-g, the auto-stabiliser], including how mouse/pad deltas feed it, and port it so view and aim stay one direction and pitch works at any heading. Pin: facing east/west a look-up pitches the view; the shot lands on the crosshair at several headings and pitches; un-ignore the walk. Run every *0700001b* binary, mouse_look_rate_live and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-1b-zero-g-look`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-11-01T04:28Z
