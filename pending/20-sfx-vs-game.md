## Your task: `sfx-vs-game`, sound effects play when and how the game plays them

Enumerate every sound-play call site in the decompile (the SFX play functions and callers, the SfxHearing table, music cue/stream functions) and map each to the port's sound events (nf-audio, nf-game SoundEvent emitters, nf-view audio): present / missing / wrong id / wrong params, with citations. Fix the ones a player hears most first: footsteps per surface, gunfire/reload/empty clicks, impacts per surface, doors/lifts/switches/keypads, pickups, gadget sounds, guard barks and pain/death, explosions, menu blips, music transitions (stealth/alert/combat). Pin with headless scenario tests asserting the emitted events (ids, positions, loop flags), each with a sister. Always run with NF_SILENT=1. Run every audio/sfx/sound/music/voice binary and nf-game. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/sfx-vs-game`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

