## Your task: `submap-placements`, objects placed on a level's sub-maps exist in play

The control-funcs-update audit says `Drones::create` builds `WorldObjects` from the main map's placements only, so a placement on a sub-map never gets its record. Its example is 07000041's one-sided wall (class 0xd5, object 01000058, on sub-map 99). The game's `parsemap_create_dynamic_objects` 0x1d04f0 takes one placements array per parsed map [M its signature]. First confirm on master which map index `WorldObjects::create` reads (drone.rs, search `WorldObjects::create(`) and whether any other path creates sub-map objects. Then census every level: which sub-maps carry placements, of which classes, and whether each is created today [D]. Read how the game parses and creates sub-maps (when each is loaded and whose placements it creates) and port that, so every class the game creates from a sub-map gets its record, in the game's order, without double-creating anything the port already makes another way. Rand draws must keep the game's count. Pin through real input on 07000041 (Bond meets the one-sided wall's collision from both sides as the game's rule says) and on one more level from the census. The one-sided draw itself is a visual and stays out of scope; list its frames under the report's live checks for the main agent. Run the worldobj binaries, every walkreal_*_live and interactables_*_live for the census levels, and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/submap-placements`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

