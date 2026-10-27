## Your task: `level-1b-space-laser`, the 0700001b space laser fires

On 0700001b (the space station) the kind-0x38 space laser never threatens the player: master creates it (`Create_SpaceLaser` 0x1aa8b8, space_laser.rs) and the level-objects-uncited round added a tested `SpaceLaser::tick`/`register`, but nothing in the sim ticks `Drones::space_lasers`. Port `SpaceLaser_Update` 0x1aa3f8 fully and wire it at the game's object-update position (dispatcher case 0x38 @0x1d06fc -> control_funcs row 0x50): states 0-4, the 10 s re-fire timer (data+0x38), the charge script + sound 0x5de, the player raycast + `Bullet_init` with the weapon def @0x2c6154 (the damage reaches Bond), the beam object (control idx 0x10) + `Emitter_Create` + 3D sound 0x46a, the ch-99 (data+0x40) state-3 write, the ch-14 disable -> obj+0xfe kill (Drake's death). BigLaserBeamHURT (0xe4 #259, gate ch 18) must be able to arm through this path. Expose the beam's endpoints per tick for the renderer (the draw is done separately). Tests: the laser arms, charges and fires at Bond on the game's timings [M]; ch 14 kills it; ch 99 rises in state 3.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-1b-space-laser`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-26T22:23Z

---
## Report (done, 2026-10-26T23:36Z)

