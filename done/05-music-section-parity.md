## Your task: `music-section-parity`, the music the mission scripts read is the same in tests and in the game

Branch from origin/master (after s/sim-audio-determinism lands: voices end on the frame clock, no catch-up sound batches while the sim runs). What is left [L real app vs headless replay of a 07000005 walk, sound attached, auto aim on both]: the two runs still split at tick 10262 (a guard's attack_talk draws in one run only; a sound node freed in one run only). The music is the difference: `attach_headless_sound` builds the Sound with `music_files: None`, so headless runs never start the music player and `Drones::iop_section` (the now-playing section the sim's UpdateMusicalEvents reads) never moves, while the windowed game streams the music and its section advances on the device's playback. Making the APP's music playhead frame-clocked hid the split but is not faithful (the PS2 streams music in real time on the IOP) and risks starving the device after dropped frames.
Do: load the level's music files in the headless Sound too and advance a music playhead there at 32 000/60 samples per game frame (no device), so headless runs see the same sections the game does at full speed. Leave the windowed game's device-timed music alone. Check: the 07000005 recorded walk replays in the app and headless with the same per-tick random state to the end (no split at 10262); then run the full suite: walks that depended on the music never advancing will move, re-pin or ignore with a task each.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/05-music-section-parity`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: main agent at 2026-11-11 14:42 MST

---
## Done (main agent, 2026-11-11 15:35 MST)
Pushed 4ea2e3338: headless runs play the level music on the frame clock; iop_section follows the windowed game within one tick on the 07000005 replay; suite 4914/0. The app keeps real-time music. Not fixed: the per-tick split at 10262 (no section change near it) -> 05-stream-slot-parity.
