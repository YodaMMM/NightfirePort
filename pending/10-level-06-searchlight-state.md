## Your task: `level-06-searchlight-state`, The Exchange: the few searchlights that still look lit after the fusebox cut

On 07000006, after the fusebox is cut with the laser (green wire), MOST searchlights go dark correctly, but a couple still show a bright lens/housing although they are off (no beam). Wanted: those couple look off exactly like the others. NOT wanted: hiding the searchlight - an earlier attempt set the head `drawn = false` on the off-kill and every searchlight vanished entirely, which is wrong (the housing stays in the game).
Find: (1) which searchlight placements stay bright after the cut and what distinguishes them from the ones that go dark correctly [D their placements, off channels, which channel the cut writes, their linked parts; whether the cut reaches them at all]; (2) exactly what the game's off path changes on the head object and its linked parts [M Searchlight_Update's off-channel kill: which fields - obj+0xFE bit, colour bytes, glist, light removal, the lens/corona parts], so the head stays drawn but unlit. Port the sim/data side so those couple end in the same state as the others, and expose the per-light on/off state; list exactly what must DRAW differently (object, part, state) for the main agent - drawing changes are the main agent's. Test through the real fusebox laser path that every searchlight ends in the same off state. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-06-searchlight-state`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

