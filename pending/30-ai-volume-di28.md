## Your task: `ai-volume-di28`, guards use the AI volumes' second parameter as the game does

`AIVol` create at 0x1376c0:85-87 stores the placement's param 2 at `DI+0x28`, and the two words after it at +0x2c and +0x26. docs/reports/unread-params-drones.md on origin/s/unread-params-drones found the port never reads param 2, across 203 records on 12 levels, and did not trace the reader. Trace every reader of +0x28, +0x2c and +0x26 on that record in the decompile. The values look like script hashes (the sample reads 0x06000000), so check what runs them and when: a guard entering the volume, a state, or a message. List in the report each reader and what the port does there, plus a census of the values per level [D]. If a reader changes behaviour, port it from the placement read through to the behaviour, keeping the count of Rand draws the same. Pin through a real level with a guard entering such a volume, red on the old code. If no reader exists, say so with the search you ran and close it. Also confirm whether drone param 22 is consumed by the stats tail, as that report left open. Run the ai binaries, walkreal for the levels you pin and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/ai-volume-di28`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

