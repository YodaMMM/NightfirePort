## Your task: `guard-castle-chat-gate`, 07000006's castle chat pair spawns and stands chatting as in the game

Branch from origin/s/guard-castle-chat-states, merge current origin/master, and read its docs/reports/guard-castle-chat-states.md, "What is left" items 1 and 2. First find, in the game, what spawns the 0x06000124/0x06000126 guard pair on 07000006: census the level's creators, events and scripts keyed on those ids or their map, and write the gate down with addresses [M]/[D]. Then census who raises channel 0x11 on 07000006 (the chat's listen channel, M 0x14c600:147-158). If the port's spawn or channel differs, fix it in the game's way and test it red on the old code. Then un-ignore walkreal_07000006_live::the_castle_chat_pair_stands_chatting_and_breaks_off_when_alerted (its legs, state-0x36 asserts and burst break-off are written). Real input only, no top-ups, never lower damage or health. If it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000006_live binary and interactables_07000006_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-castle-chat-gate`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-12T02:11Z
