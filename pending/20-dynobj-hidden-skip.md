## Your task: `dynobj-hidden-skip`, placed dynamic objects the game hides from sight rays stop blocking them

Branch from origin/master. DynamicObject_Create 0x18dbf8:25-31 ORs 0x30 into obj+0xF0 when the placement's +0x2c short is non-zero; 0x20 is in every Collide_PickObj skip set [M 0x1eae80:70-82], so the game never picks those objects for any masked ray (guard sight, sensors, auto-aim, the beam reach). The port's world.rs Moving::pick_pair carries only the glist word plus the hide bit 0x10. Find which placements have +0x2c != 0 per level [D], carry the 0x30 into the dynamic object's flags, and pin it with a unit test (red on the old code) plus one real-level check where such an object sits on a guard's sight line, if any exists [D]. Run nf-game's unit tests, ai_live, and every walkreal_*_live binary one level per tool call.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-dynobj-hidden-skip`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
