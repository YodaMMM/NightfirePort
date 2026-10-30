## Your task: `level-14-cascade-servers-and-door-guards`, Countdown Part 1: the generator-hall blast aftermath

In play on 07000014, after the fusebox sabotage and the E on the console, the generator hall blows up and the server racks simply disappear; and guards stand behind the double door repeatedly pressing its open button. (1) The servers: for each Destroy / breakable the cascade smashes, find what the game shows after the smash [M Destroy_Smash 0x12bb60's Control_SetGList swap to the destroyed glist (block-0x27 LOD / the parsemap fallback slot); D each object's destroyed model] and draw it - a leaf whose destroyed glist is a real mesh must show the wreck, only an undrawable slot vanishes. (2) The guards at the door: find what state they are in and what the game's guards do at a locked/unlocking door [M the door-guard / door-use AI states, Door_Activate 0x135be0 for NPCs]; fix the loop so they behave like the game's. Pin both with real-input tests. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-cascade-servers-and-door-guards`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-30T01:52Z
