## Your task: `decryptor-digit-sounds`, the decryptor's digit and unlock sounds; the camera beam through doors

1. The decryptor: `Lock_Update` 0x1ae3d8 plays sound 0x5db each time a digit solves (:346-369) and 0x5da on the unlock (:370-374) [M] — the port plays neither (no 0x5db/0x5da anywhere in crates/). Play them through the port's 3-D sound path at the lock's position, with a test that a held scan emits one 0x5db per solved digit and one 0x5da at the unlock. Check the scan's own sounds too (the scanning beep/hum, if the decompile plays one) and port them.
2. The security-camera sight beam (posed slot 3 per camera, scene.rs) reads as blending into doors: check the beam's draw state against the game's (the Camera_Beam glist flags 0x241, the blend/z-write/z-test the game's draw applies, the draw order) [M + D] so the beam is drawn as the game draws it — not over or through closed doors if the game's isn't.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/decryptor-digit-sounds`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
