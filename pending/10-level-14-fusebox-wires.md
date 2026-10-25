## Your task: `level-14-fusebox-wires`, 07000014: the opened fusebox panel shows its wires

Start from `origin/s/level-14-empty-panels` (merge it first; docs/reports/level-14-empty-panels.md). The wall panel at (-88.58, -1.10, 46.15) is door #386 beside the real fusebox #385. The fusebox's script 0x06000A1B [D placement params] spawns the wire sub-objects 0x02000A62 / 0x02000A61 / 0x02000A73 (type 0x31) in streams 0/1/3 [D decode, key-1 Entities]; the port's fusebox.rs stands in with one wire per group, so the opened panel shows an empty recess.

1. Run the fusebox script's Entity streams through the same SP-entity path the route-lid replay uses (SP_Update 0x1dbda8, SP_Activate 0x1dc338, the entity create by type [M]), replacing the stand-in: the wires exist, draw, and take the game's damage/sabotage path (the fusebox objective "Sabotage base security") [M FuseBox_Update 0x1adfe8].
2. Un-ignore `panels_07000014_live::the_fusebox_use_spawns_the_wire_sub_objects_behind_the_panel` and make it green with real input; keep the 14 walks and the fusebox/objective tests green. Name headless renders of the opened panel before/after in the report.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-fusebox-wires`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
