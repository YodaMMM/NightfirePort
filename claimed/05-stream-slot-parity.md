## Your task: `stream-slot-parity`, the last app-vs-headless split (tick 10262 of the 07000005 replay)

Branch from origin/master (voice ends on the frame clock, no catch-up sound batches while the sim runs, headless music on the frame clock: all merged). Replaying the same recorded 07000005 walk in the windowed app (NF_NULL_MIXER) and headless (attach_headless_sound, auto aim on in both) gives the same per-tick random state up to tick 10261, then splits at 10262: a guard's attack_talk (drone.rs ~4448/4464) draws in the headless run only, because one sound node (sfx id 123) is freed at tick 10208 headless but still live in the app. Music sections now match within a tick and none change near 10262. When the app's music playhead was ALSO put on the frame clock (an experiment, not faithful), the two runs matched all 10440 ticks - so the app's real-time music reaches the sound driver's lifetimes some other way: suspect the stream slots (`STREAM_SLOTS`, slot 0 = the music) or the music queue state the driver reads from the device. Find the exact path, make the driver's sim-visible state independent of the device's music timing (the music itself may keep playing in real time), and add a test.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/05-stream-slot-parity`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: main agent at 2026-11-11 15:35 MST
