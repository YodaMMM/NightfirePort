## Your task: `level-0a-rain-indoors`, 0700000a: rain falls inside the office

In real play on 0700000a, rain streaks fall indoors — in the dark office lounge at Bond (-37.50, 1.11, -16.95, yaw 2.827, pitch -0.092), frame 2719. Rain belongs outside the windows only.

1. Find how the game confines its rain [M — the weather/rain emitter's creator and update, its volume/box or room gate, any indoor/outdoor flag on the room or the camera's room, the particle cull], and why the port draws it in this room.
2. Fix it [M]; a test that no rain particle draws with the camera in this room, and a sister that rain still draws outside (0b's rain stays as it is — the user already checked 0b's rain as right). Headless render of the spot named in the report.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-rain-indoors`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-10-25T04:00Z
