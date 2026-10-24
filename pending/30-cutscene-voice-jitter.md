## Your task: `cutscene-voice-jitter`, cutscene dialogue sometimes jitters rapidly

Symptom: during cutscenes, spoken lines sometimes jitter rapidly (a fast stutter/warble in the voice), not every time.

1. Reproduce through the real path first: play the cutscenes with dialogue (07000001's opening, 07000005's, 07000008's) through the sim and the audio mixer with NF_SILENT, and record per mixer block the voice channel's source position, gain, pan and pitch, and the stream's read position / buffer fill. Find a measurable jitter: per-tick swings of pan/gain/pitch (a speaker attached to a bone or a camera-relative position that wobbles between ticks, the doppler/pitch from a moving emitter), or stream underruns and repeats.
2. Compare each with the game's rule: how the NIS speech is played (2D or 3D, attached or fixed) and how often its volume/pan are updated [M the Sound_Play3D / stream calls in the NIS event handler and the IOP SFX path], and fix the port to it.
3. Test: over a whole dialogue cutscene the voice's pan/gain/pitch change no faster than the game's update rule allows, and the stream never underruns or repeats a block. Sister: a moving 3D source outside cutscenes still pans.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cutscene-voice-jitter`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

