## Your task: `session-skins-only`, the session skin merge takes skins only

Start from `origin/s/985-session-skin-0600005b` (NOT merged). It makes the session-skin collection follow sub-scripts, so 07000001's script 0x0600005b stream 7 finds skin 0x05000037, carried only by 07000005/06 [D]. With it, level 01 changes far beyond that skin: `door_portal_live::doors_link_to_their_portal_quads_on_07000001` fails (the linked edges become [(7,144),(7,155),(11,76)] instead of the measured [(7,20),(7,40),(11,76)]), and the 01 walks shift with it. The merge brings in more than the skin.

1. Find what the merge adds besides the skin (maps, objects, cels, portals, collision) — the game's session gate registers the container's SKINS in the skin hashtable [M `AnimProcessSkinData` 0x1bd500 via `AnimLoadFile` 0x1bdf40 / `LoaderProcess` 0x1d4880 case 3]; it adds no world geometry to the level.
2. Make the merge take the skin (and what the skin itself needs to draw) and nothing else, so every 01 test that passes on master still passes, and 0x05000037 still resolves. Tests: the door portal pins unchanged; the skin present after load.
3. Render-relevant: say in the report what 0x0600005b stream 7 shows and where, so it can be looked at.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/session-skins-only`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


