## Your task: `level-13-security-room`, 07000013: the security room door and the panel

Two `interactables_07000013_live` tests are ignored:
- the security-room approach: the walk to door #355 at (-26.65, -368.36, 37.44) wedges west of the desk block;
- the panel: the aim-fire loop connects (the panel spans y -369..-354 at (6.46, 36.72) [D bounds]) but no break lands in 900 ticks.

1. The approach: find what wedges the walk (the triangle owners at the wedge, the capsule contacts) and whether the game's player passes there [M — the capsule push and pick rules], or the route needs the game's real path; fix and walk it.
2. The panel: find how the game damages/breaks this panel [M — its creator by placement kind, its damage handler, the bullet contact against its bounds] and why the port's rounds do not land; fix it.
3. Un-ignore both tests as they go green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-security-room`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
