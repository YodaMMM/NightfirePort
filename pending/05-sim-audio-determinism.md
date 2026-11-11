## Your task: `sim-audio-determinism`, the game's sim never reads the audio device's clock (FIRST PRIORITY)

Branch from origin/master. Measured: the same recorded inputs play differently in the windowed app than in any headless test, because sim state depends on when the OUTPUT DEVICE finishes a sound. crates/nf-audio/src/sfx.rs `Sound::batch` takes the voices' end flags from `o.take_ended()` (the mixer thread, wall clock) when a device exists, and from `self.driver.clock.advance()` (VoiceClock, 800 samples per 60 Hz frame, the SPU's lockstep) only without one. The end flags feed `Driver::is_playing` -> `Sound::update`'s `playing(h)` -> `Sound_IsFinnished` -> `Drone::voice_finished` (drone.rs, `dr.voice_finished = ...self.audio.finished(..)`) -> `voice_busy()` -> `attack_talk`'s Rand draws on the shared stream. On the 07000005 door-band walk's inputs the app's RNG split from the headless replay at tick 1187 (two extra draws at attack_talk), and the fights after it ran differently.
Do:
1. In `Sound::batch` always run `self.driver.clock.apply(&out)` and take `ended` from `self.driver.clock.advance()`; drain and drop the device's `take_ended()`; still `send` the commands to the device (same in `set_paused`). The game's SPU end flags are frame-locked, which the clock models.
2. Find every other device-timed value that reaches the sim and make it frame-clocked the same way: `music_played()` -> the music player -> `Drones::iop_section` / UpdateMusicalEvents, streamed effects, NIS sound timing. After step 1 a second split remained at tick 10262 (attack_talk drew headless, not in the app): find it.
3. Test: a committed check that a sim stepped with a real-time device stand-in (an Output whose end list runs early/late) gives the same per-tick RNG and drone state as the headless clock. Also confirm by ear-free means that sounds still start/stop: the mixer must not cut lines early (compare each voice's clock end against its sample length).
Then the full suite.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/05-sim-audio-determinism`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
